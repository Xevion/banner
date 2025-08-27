//! Session management for Banner API.

use crate::banner::util::user_agent;
use anyhow::Result;
use rand::distributions::{Alphanumeric, DistString};
use reqwest::Client;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tracing::{debug, info};

/// Session manager for Banner API interactions
#[derive(Debug)]
pub struct SessionManager {
    current_session: Mutex<Option<SessionData>>,
    base_url: String,
    client: Client,
}

#[derive(Debug, Clone)]
struct SessionData {
    session_id: String,
    created_at: Instant,
}

impl SessionManager {
    const SESSION_EXPIRY: Duration = Duration::from_secs(25 * 60); // 25 minutes

    /// Creates a new session manager
    pub fn new(base_url: String, client: Client) -> Self {
        Self {
            current_session: Mutex::new(None),
            base_url,
            client,
        }
    }

    /// Ensures a valid session is available, creating one if necessary
    pub fn ensure_session(&self) -> Result<String> {
        let start_time = std::time::Instant::now();
        let mut session_guard = self.current_session.lock().unwrap();

        if let Some(ref session) = *session_guard
            && session.created_at.elapsed() < Self::SESSION_EXPIRY
        {
            let elapsed = start_time.elapsed();
            debug!(
                session_id = session.session_id,
                elapsed = format!("{:.2?}", elapsed),
                "reusing existing banner session"
            );
            return Ok(session.session_id.clone());
        }

        // Generate new session
        let session_id = self.generate_session_id();
        *session_guard = Some(SessionData {
            session_id: session_id.clone(),
            created_at: Instant::now(),
        });

        let elapsed = start_time.elapsed();
        debug!(
            session_id = session_id,
            elapsed = format!("{:.2?}", elapsed),
            "generated new banner session"
        );
        Ok(session_id)
    }

    /// Generates a new session ID mimicking Banner's format
    fn generate_session_id(&self) -> String {
        let random_part = Alphanumeric.sample_string(&mut rand::thread_rng(), 5);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        format!("{}{}", random_part, timestamp)
    }

    /// Sets up initial session cookies by making required Banner API requests
    pub async fn setup(&self) -> Result<()> {
        info!("setting up banner session...");

        let request_paths = ["/registration/registration", "/selfServiceMenu/data"];

        for path in &request_paths {
            let url = format!("{}{}", self.base_url, path);
            let response = self
                .client
                .get(&url)
                .query(&[("_", Self::nonce())])
                .header("User-Agent", user_agent())
                .send()
                .await?;

            if !response.status().is_success() {
                return Err(anyhow::anyhow!(
                    "Failed to setup session, request to {} returned {}",
                    path,
                    response.status()
                ));
            }
        }

        // Note: Cookie validation would require additional setup in a real implementation
        debug!("session setup complete");
        Ok(())
    }

    /// Selects a term for the current session
    pub async fn select_term(&self, term: &str) -> Result<()> {
        let session_id = self.ensure_session()?;

        let form_data = [
            ("term", term),
            ("studyPath", ""),
            ("studyPathText", ""),
            ("startDatepicker", ""),
            ("endDatepicker", ""),
            ("uniqueSessionId", &session_id),
        ];

        let url = format!("{}/term/search", self.base_url);
        let response = self
            .client
            .post(&url)
            .query(&[("mode", "search")])
            .form(&form_data)
            .header("User-Agent", user_agent())
            .header("Content-Type", "application/x-www-form-urlencoded")
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
            #[serde(rename = "fwdUrl")]
            fwd_url: String,
        }

        let redirect: RedirectResponse = response.json().await?;

        // Follow the redirect
        let redirect_url = format!("{}{}", self.base_url, redirect.fwd_url);
        let redirect_response = self
            .client
            .get(&redirect_url)
            .header("User-Agent", user_agent())
            .send()
            .await?;

        if !redirect_response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to follow redirect: {}",
                redirect_response.status()
            ));
        }

        debug!("successfully selected term: {}", term);
        Ok(())
    }

    /// Resets the data form (required before new searches)
    pub async fn reset_data_form(&self) -> Result<()> {
        let url = format!("{}/classSearch/resetDataForm", self.base_url);
        let response = self
            .client
            .post(&url)
            .header("User-Agent", user_agent())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to reset data form: {}",
                response.status()
            ));
        }

        Ok(())
    }

    /// Generates a timestamp-based nonce
    pub fn nonce() -> String {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .to_string()
    }
}
