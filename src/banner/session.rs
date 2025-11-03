//! Session management for Banner API.

use crate::banner::BannerTerm;
use crate::banner::models::Term;
use anyhow::{Context, Result};
use cookie::Cookie;
use dashmap::DashMap;
use governor::state::InMemoryState;
use governor::{Quota, RateLimiter};
use once_cell::sync::Lazy;
use rand::distr::{Alphanumeric, SampleString};
use reqwest_middleware::ClientWithMiddleware;
use std::collections::{HashMap, VecDeque};
use std::num::NonZeroU32;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Notify};
use tracing::{debug, info, trace};
use url::Url;

const SESSION_EXPIRY: Duration = Duration::from_secs(25 * 60); // 25 minutes

// A global rate limiter to ensure we only try to create one new session every 10 seconds,
// preventing us from overwhelming the server with session creation requests.
static SESSION_CREATION_RATE_LIMITER: Lazy<
    RateLimiter<governor::state::direct::NotKeyed, InMemoryState, governor::clock::DefaultClock>,
> = Lazy::new(|| RateLimiter::direct(Quota::with_period(Duration::from_secs(10)).unwrap()));

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
    // This Arc points directly to the term-specific pool.
    pool: Arc<TermPool>,
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
            let pool = self.pool.clone();
            // Since drop() cannot be async, we spawn a task to return the session.
            tokio::spawn(async move {
                pool.release(session).await;
            });
        }
    }
}

pub struct TermPool {
    sessions: Mutex<VecDeque<BannerSession>>,
    notifier: Notify,
    is_creating: Mutex<bool>,
}

impl TermPool {
    fn new() -> Self {
        Self {
            sessions: Mutex::new(VecDeque::new()),
            notifier: Notify::new(),
            is_creating: Mutex::new(false),
        }
    }

    async fn release(&self, session: BannerSession) {
        let id = session.unique_session_id.clone();
        if session.is_expired() {
            debug!(id = id, "Session expired, dropping");
            // Wake up a waiter, as it might need to create a new session
            // if this was the last one.
            self.notifier.notify_one();
            return;
        }

        let mut queue = self.sessions.lock().await;
        queue.push_back(session);
        drop(queue); // Release lock before notifying

        self.notifier.notify_one();
    }
}

pub struct SessionPool {
    sessions: DashMap<Term, Arc<TermPool>>,
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
    /// If no sessions are available, a new one is created on demand,
    /// respecting the global rate limit.
    pub async fn acquire(&self, term: Term) -> Result<PooledSession> {
        let term_pool = self
            .sessions
            .entry(term)
            .or_insert_with(|| Arc::new(TermPool::new()))
            .clone();

        let start = Instant::now();
        let mut waited_for_creation = false;

        loop {
            // Fast path: Try to get an existing, non-expired session.
            {
                let mut queue = term_pool.sessions.lock().await;
                if let Some(session) = queue.pop_front() {
                    if !session.is_expired() {
                        return Ok(PooledSession {
                            session: Some(session),
                            pool: Arc::clone(&term_pool),
                        });
                    } else {
                        debug!(id = session.unique_session_id, "Discarded expired session");
                    }
                }
            } // MutexGuard is dropped, lock is released.

            // Slow path: No sessions available. We must either wait or become the creator.
            let mut is_creating_guard = term_pool.is_creating.lock().await;
            if *is_creating_guard {
                // Another task is already creating a session. Release the lock and wait.
                drop(is_creating_guard);
                if !waited_for_creation {
                    trace!("Waiting for another task to create session");
                    waited_for_creation = true;
                }
                term_pool.notifier.notified().await;
                // Loop back to the top to try the fast path again.
                continue;
            }

            // This task is now the designated creator.
            *is_creating_guard = true;
            drop(is_creating_guard);

            // Race: wait for a session to be returned OR for the rate limiter to allow a new one.
            trace!("Pool empty, creating new session");
            tokio::select! {
                _ = term_pool.notifier.notified() => {
                    // A session was returned while we were waiting!
                    // We are no longer the creator. Reset the flag and loop to race for the new session.
                    let mut guard = term_pool.is_creating.lock().await;
                    *guard = false;
                    drop(guard);
                    continue;
                }
                _ = SESSION_CREATION_RATE_LIMITER.until_ready() => {
                    // The rate limit has elapsed. It's our job to create the session.
                    let new_session_result = self.create_session(&term).await;

                    // After creation, we are no longer the creator. Reset the flag
                    // and notify all other waiting tasks.
                    let mut guard = term_pool.is_creating.lock().await;
                    *guard = false;
                    drop(guard);
                    term_pool.notifier.notify_waiters();

                    match new_session_result {
                        Ok(new_session) => {
                            let elapsed = start.elapsed();
                            debug!(
                                id = new_session.unique_session_id,
                                elapsed_ms = elapsed.as_millis(),
                                "Created new session"
                            );
                            return Ok(PooledSession {
                                session: Some(new_session),
                                pool: term_pool,
                            });
                        }
                        Err(e) => {
                            // Propagate the error if session creation failed.
                            return Err(e.context("Failed to create new session in pool"));
                        }
                    }
                }
            }
        }
    }

    /// Sets up initial session cookies by making required Banner API requests
    pub async fn create_session(&self, term: &Term) -> Result<BannerSession> {
        info!(term = %term, "setting up banner session");

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
                if let Ok(cookie_str) = header_value.to_str() {
                    if let Ok(cookie) = Cookie::parse(cookie_str) {
                        Some((cookie.name().to_string(), cookie.value().to_string()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect::<HashMap<String, String>>();

        if !cookies.contains_key("JSESSIONID") || !cookies.contains_key("SSB_COOKIE") {
            return Err(anyhow::anyhow!("Failed to get cookies"));
        }

        let jsessionid = cookies.get("JSESSIONID")
            .ok_or_else(|| anyhow::anyhow!("JSESSIONID cookie missing after validation"))?;
        let ssb_cookie = cookies.get("SSB_COOKIE")
            .ok_or_else(|| anyhow::anyhow!("SSB_COOKIE cookie missing after validation"))?;
        let cookie_header = format!("JSESSIONID={}; SSB_COOKIE={}", jsessionid, ssb_cookie);

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

        let terms = self.get_terms("", 1, 10).await?;
        if !terms.iter().any(|t| t.code == term.to_string()) {
            return Err(anyhow::anyhow!("Failed to get term search response"));
        }

        let specific_term_search_response = self.get_terms(&term.to_string(), 1, 10).await?;
        if !specific_term_search_response
            .iter()
            .any(|t| t.code == term.to_string())
        {
            return Err(anyhow::anyhow!("Failed to get term search response"));
        }

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
            .with_context(|| "Failed to get terms".to_string())?;

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

        let base_url_path = self.base_url.parse::<Url>()
            .context("Failed to parse base URL")?
            .path()
            .to_string();
        let non_overlap_redirect = redirect.fwd_url.strip_prefix(&base_url_path)
            .ok_or_else(|| anyhow::anyhow!(
                "Redirect URL '{}' does not start with expected prefix '{}'",
                redirect.fwd_url, base_url_path
            ))?;

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

        Ok(())
    }
}
