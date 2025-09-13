//! Session management for Banner API.

use crate::banner::BannerTerm;
use crate::banner::models::Term;
use anyhow::{Context, Result};
use cookie::Cookie;
use dashmap::DashMap;
use rand::distr::{Alphanumeric, SampleString};
use reqwest::Client;
use reqwest_middleware::ClientWithMiddleware;
use std::collections::{HashMap, VecDeque};
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, info};
use url::Url;

const SESSION_EXPIRY: Duration = Duration::from_secs(25 * 60); // 25 minutes

/// Represents an active anonymous session within the Banner API.
/// Identified by multiple persistent cookies, as well as a client-generated "unique session ID".
#[derive(Debug, Clone)]
pub struct BannerSession {
    // Randomly generated
    pub unique_session_id: String,
    // Timestamp of creation
    created_at: Instant,
    // Timestamp of last activity
    last_activity: Option<Instant>,
    // Cookie values from initial registration page
    jsessionid: String,
    ssb_cookie: String,
}

/// Generates a new session ID mimicking Banner's format
fn generate_session_id() -> String {
    let random_part = Alphanumeric.sample_string(&mut rand::rng(), 5);
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("{}{}", random_part, timestamp)
}

/// Generates a timestamp-based nonce
pub fn nonce() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string()
}

impl BannerSession {
    /// Creates a new session
    pub async fn new(unique_session_id: &str, jsessionid: &str, ssb_cookie: &str) -> Result<Self> {
        let now = Instant::now();

        Ok(Self {
            created_at: now,
            last_activity: None,
            unique_session_id: unique_session_id.to_string(),
            jsessionid: jsessionid.to_string(),
            ssb_cookie: ssb_cookie.to_string(),
        })
    }

    /// Returns the unique session ID
    pub fn id(&self) -> String {
        self.unique_session_id.clone()
    }

    /// Updates the last activity timestamp
    pub fn touch(&mut self) {
        debug!(id = self.unique_session_id, "Session was used");
        self.last_activity = Some(Instant::now());
    }

    /// Returns true if the session is expired
    pub fn is_expired(&self) -> bool {
        self.last_activity.unwrap_or(self.created_at).elapsed() > SESSION_EXPIRY
    }

    /// Returns a string used to for the "Cookie" header
    pub fn cookie(&self) -> String {
        format!(
            "JSESSIONID={}; SSB_COOKIE={}",
            self.jsessionid, self.ssb_cookie
        )
    }

    pub fn been_used(&self) -> bool {
        self.last_activity.is_some()
    }
}

/// A smart pointer that returns a BannerSession to the pool when dropped.
pub struct PooledSession {
    session: Option<BannerSession>,
    // This Arc points directly to the queue the session belongs to.
    pool: Arc<Mutex<VecDeque<BannerSession>>>,
}

impl PooledSession {
    pub fn been_used(&self) -> bool {
        self.session.as_ref().unwrap().been_used()
    }
}

impl Deref for PooledSession {
    type Target = BannerSession;
    fn deref(&self) -> &Self::Target {
        // The option is only ever None after drop is called, so this is safe.
        self.session.as_ref().unwrap()
    }
}

impl DerefMut for PooledSession {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.session.as_mut().unwrap()
    }
}

/// The magic happens here: when the guard goes out of scope, this is called.
impl Drop for PooledSession {
    fn drop(&mut self) {
        if let Some(session) = self.session.take() {
            // Don't return expired sessions to the pool.
            if session.is_expired() {
                debug!(
                    id = session.unique_session_id,
                    "Session is now expired, dropping."
                );
                return;
            }

            // This is a synchronous lock, so it's allowed in drop().
            // It blocks the current thread briefly to return the session.
            let mut queue = self.pool.lock().unwrap();

            let id = session.unique_session_id.clone();
            queue.push_back(session);

            debug!(
                id = id,
                "Session returned to pool. Queue size is now {queue_size}.",
                queue_size = queue.len(),
            );
        }
    }
}

pub struct SessionPool {
    sessions: DashMap<Term, Arc<Mutex<VecDeque<BannerSession>>>>,
    http: ClientWithMiddleware,
    base_url: String,
}

impl SessionPool {
    pub fn new(http: ClientWithMiddleware, base_url: String) -> Self {
        Self {
            sessions: DashMap::new(),
            http,
            base_url,
        }
    }

    /// Acquires a session from the pool.
    /// If no sessions are available, a new one is created on demand.
    pub async fn acquire(&self, term: Term) -> Result<PooledSession> {
        // Get the queue for the given term, or insert a new empty one.
        let pool_entry = self
            .sessions
            .entry(term.clone())
            .or_insert_with(|| Arc::new(Mutex::new(VecDeque::new())))
            .clone();

        loop {
            // Lock the specific queue for this term
            let session_option = {
                let mut queue = pool_entry.lock().unwrap();
                queue.pop_front() // Try to get a session
            };

            if let Some(mut session) = session_option {
                // We got a session, check if it's expired.
                if !session.is_expired() {
                    debug!(id = session.unique_session_id, "Reusing session");

                    session.touch();
                    return Ok(PooledSession {
                        session: Some(session),
                        pool: pool_entry,
                    });
                } else {
                    debug!(
                        id = session.unique_session_id,
                        "Popped an expired session, discarding.",
                    );
                    // The session is expired, so we loop again to try and get another one.
                }
            } else {
                // Queue was empty, so we create a new session.
                let mut new_session = self.create_session(&term).await?;
                new_session.touch();

                return Ok(PooledSession {
                    session: Some(new_session),
                    pool: pool_entry,
                });
            }
        }
    }

    /// Sets up initial session cookies by making required Banner API requests
    pub async fn create_session(&self, term: &Term) -> Result<BannerSession> {
        info!("setting up banner session for term {term}");

        // The 'register' or 'search' registration page
        let initial_registration = self
            .http
            .get(format!("{}/registration", self.base_url))
            .send()
            .await?;
        // TODO: Validate success

        let cookies = initial_registration
            .headers()
            .get_all("Set-Cookie")
            .iter()
            .filter_map(|header_value| {
                if let Ok(cookie) = Cookie::parse(header_value.to_str().unwrap()) {
                    Some((cookie.name().to_string(), cookie.value().to_string()))
                } else {
                    None
                }
            })
            .collect::<HashMap<String, String>>();

        if cookies.get("JSESSIONID").is_none() || cookies.get("SSB_COOKIE").is_none() {
            return Err(anyhow::anyhow!("Failed to get cookies"));
        }

        let jsessionid = cookies.get("JSESSIONID").unwrap();
        let ssb_cookie = cookies.get("SSB_COOKIE").unwrap();
        let cookie_header = format!("JSESSIONID={}; SSB_COOKIE={}", jsessionid, ssb_cookie);

        debug!(
            jsessionid = jsessionid,
            ssb_cookie = ssb_cookie,
            "New session cookies acquired"
        );

        self.http
            .get(format!("{}/selfServiceMenu/data", self.base_url))
            .header("Cookie", &cookie_header)
            .send()
            .await?
            .error_for_status()
            .context("Failed to get data page")?;

        self.http
            .get(format!("{}/term/termSelection", self.base_url))
            .header("Cookie", &cookie_header)
            .query(&[("mode", "search")])
            .send()
            .await?
            .error_for_status()
            .context("Failed to get term selection page")?;
        // TOOD: Validate success

        /*let terms = self.get_terms("", 1, 10).await?;
        if !terms.iter().any(|t| t.code == term.to_string()) {
            return Err(anyhow::anyhow!("Failed to get term search response"));
        }

        let specific_term_search_response = self.get_terms(&term.to_string(), 1, 10).await?;
        if !specific_term_search_response
            .iter()
            .any(|t| t.code == term.to_string())
        {
            return Err(anyhow::anyhow!("Failed to get term search response"));
        }*/

        let unique_session_id = generate_session_id();
        self.select_term(&term.to_string(), &unique_session_id, &cookie_header)
            .await?;

        BannerSession::new(&unique_session_id, jsessionid, ssb_cookie).await
    }

    /// Retrieves a list of terms from the Banner API.
    pub async fn get_terms(
        &self,
        search: &str,
        page: i32,
        max_results: i32,
    ) -> Result<Vec<BannerTerm>> {
        if page <= 0 {
            return Err(anyhow::anyhow!("Page must be greater than 0"));
        }

        let url = format!("{}/classSearch/getTerms", self.base_url);
        let params = [
            ("searchTerm", search),
            ("offset", &page.to_string()),
            ("max", &max_results.to_string()),
            ("_", &nonce()),
        ];

        let response = self
            .http
            .get(&url)
            .query(&params)
            .send()
            .await
            .with_context(|| format!("Failed to get terms"))?;

        let terms: Vec<BannerTerm> = response
            .json()
            .await
            .context("Failed to parse terms response")?;

        Ok(terms)
    }

    /// Selects a term for the current session
    pub async fn select_term(
        &self,
        term: &str,
        unique_session_id: &str,
        cookie_header: &str,
    ) -> Result<()> {
        let form_data = [
            ("term", term),
            ("studyPath", ""),
            ("studyPathText", ""),
            ("startDatepicker", ""),
            ("endDatepicker", ""),
            ("uniqueSessionId", unique_session_id),
        ];

        let url = format!("{}/term/search", self.base_url);
        let response = self
            .http
            .post(&url)
            .header("Cookie", cookie_header)
            .query(&[("mode", "search")])
            .form(&form_data)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to select term {}: {}",
                term,
                response.status()
            ));
        }

        #[derive(serde::Deserialize)]
        struct RedirectResponse {
            #[serde(rename = "fwdURL")]
            fwd_url: String,
        }

        let redirect: RedirectResponse = response.json().await?;

        let base_url_path = self.base_url.parse::<Url>().unwrap().path().to_string();
        let non_overlap_redirect = redirect.fwd_url.strip_prefix(&base_url_path).unwrap();

        // Follow the redirect
        let redirect_url = format!("{}{}", self.base_url, non_overlap_redirect);
        let redirect_response = self
            .http
            .get(&redirect_url)
            .header("Cookie", cookie_header)
            .send()
            .await?;

        if !redirect_response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to follow redirect: {}",
                redirect_response.status()
            ));
        }

        debug!(term = term, "successfully selected term");
        Ok(())
    }
}
