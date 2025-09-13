//! Main Banner API client implementation.

use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
    time::Instant,
};

use crate::banner::{
    BannerSession, SessionPool, models::*, nonce, query::SearchQuery, util::user_agent,
};
use anyhow::{Context, Result, anyhow};
use cookie::Cookie;
use dashmap::DashMap;
use http::{Extensions, HeaderValue};
use reqwest::{Client, Request, Response};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, Middleware, Next};
use serde_json;
use tl;
use tracing::{Level, Metadata, Span, debug, error, field::ValueSet, info, span, trace, warn};

#[derive(Debug, thiserror::Error)]
pub enum BannerApiError {
    #[error("Banner session is invalid or expired: {0}")]
    InvalidSession(String),
    #[error(transparent)]
    RequestFailed(#[from] anyhow::Error),
}

/// Main Banner API client.
pub struct BannerApi {
    pub sessions: SessionPool,
    http: ClientWithMiddleware,
    base_url: String,
}

pub struct TransparentMiddleware;

#[async_trait::async_trait]
impl Middleware for TransparentMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> std::result::Result<Response, reqwest_middleware::Error> {
        trace!(
            domain = req.url().domain(),
            headers = ?req.headers(),
            "{method} {path}",
            method = req.method().to_string(),
            path = req.url().path(),
        );
        let response_result = next.run(req, extensions).await;

        match response_result {
            Ok(response) => {
                if response.status().is_success() {
                    trace!(
                        "{code} {reason} {path}",
                        code = response.status().as_u16(),
                        reason = response.status().canonical_reason().unwrap_or("??"),
                        path = response.url().path(),
                    );
                    Ok(response)
                } else {
                    let e = response.error_for_status_ref().unwrap_err();
                    warn!(error = ?e, "Request failed (server)");
                    Ok(response)
                }
            }
            Err(error) => {
                warn!(?error, "Request failed (middleware)");
                Err(error)
            }
        }
    }
}

impl BannerApi {
    /// Creates a new Banner API client.
    pub fn new(base_url: String) -> Result<Self> {
        let http = ClientBuilder::new(
            Client::builder()
                .cookie_store(false)
                .user_agent(user_agent())
                .tcp_keepalive(Some(std::time::Duration::from_secs(60 * 5)))
                .read_timeout(std::time::Duration::from_secs(10))
                .connect_timeout(std::time::Duration::from_secs(10))
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .context("Failed to create HTTP client")?,
        )
        .with(TransparentMiddleware)
        .build();

        Ok(Self {
            sessions: SessionPool::new(http.clone(), base_url.clone()),
            http,
            base_url,
        })
    }

    /// Retrieves a list of subjects from the Banner API.
    pub async fn get_subjects(
        &self,
        search: &str,
        term: &str,
        offset: i32,
        max_results: i32,
    ) -> Result<Vec<Pair>> {
        if offset <= 0 {
            return Err(anyhow::anyhow!("Offset must be greater than 0"));
        }

        let session = self.sessions.acquire(term.parse()?).await?;
        let url = format!("{}/classSearch/get_subject", self.base_url);
        let params = [
            ("searchTerm", search),
            ("term", term),
            ("offset", &offset.to_string()),
            ("max", &max_results.to_string()),
            ("uniqueSessionId", &session.id()),
            ("_", &nonce()),
        ];

        let response = self
            .http
            .get(&url)
            .query(&params)
            .send()
            .await
            .context("Failed to get subjects")?;

        let subjects: Vec<Pair> = response
            .json()
            .await
            .context("Failed to parse subjects response")?;

        Ok(subjects)
    }

    /// Retrieves a list of instructors from the Banner API.
    pub async fn get_instructors(
        &self,
        search: &str,
        term: &str,
        offset: i32,
        max_results: i32,
    ) -> Result<Vec<Instructor>> {
        if offset <= 0 {
            return Err(anyhow::anyhow!("Offset must be greater than 0"));
        }

        let session = self.sessions.acquire(term.parse()?).await?;
        let url = format!("{}/classSearch/get_instructor", self.base_url);
        let params = [
            ("searchTerm", search),
            ("term", term),
            ("offset", &offset.to_string()),
            ("max", &max_results.to_string()),
            ("uniqueSessionId", &session.id()),
            ("_", &nonce()),
        ];

        let response = self
            .http
            .get(&url)
            .query(&params)
            .send()
            .await
            .context("Failed to get instructors")?;

        let instructors: Vec<Instructor> = response
            .json()
            .await
            .context("Failed to parse instructors response")?;

        Ok(instructors)
    }

    /// Retrieves a list of campuses from the Banner API.
    pub async fn get_campuses(
        &self,
        search: &str,
        term: &str,
        offset: i32,
        max_results: i32,
    ) -> Result<Vec<Pair>> {
        if offset <= 0 {
            return Err(anyhow::anyhow!("Offset must be greater than 0"));
        }

        let session = self.sessions.acquire(term.parse()?).await?;
        let url = format!("{}/classSearch/get_campus", self.base_url);
        let params = [
            ("searchTerm", search),
            ("term", term),
            ("offset", &offset.to_string()),
            ("max", &max_results.to_string()),
            ("uniqueSessionId", &session.id()),
            ("_", &nonce()),
        ];

        let response = self
            .http
            .get(&url)
            .query(&params)
            .send()
            .await
            .context("Failed to get campuses")?;

        let campuses: Vec<Pair> = response
            .json()
            .await
            .context("Failed to parse campuses response")?;

        Ok(campuses)
    }

    /// Retrieves meeting time information for a course.
    pub async fn get_course_meeting_time(
        &self,
        term: &str,
        crn: &str,
    ) -> Result<Vec<MeetingScheduleInfo>> {
        let url = format!("{}/searchResults/getFacultyMeetingTimes", self.base_url);
        let params = [("term", term), ("courseReferenceNumber", crn)];

        let response = self
            .http
            .get(&url)
            .query(&params)
            .send()
            .await
            .context("Failed to get meeting times")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to get meeting times: {}",
                response.status()
            ));
        } else if !response
            .headers()
            .get("Content-Type")
            .unwrap_or(&HeaderValue::from_static(""))
            .to_str()
            .unwrap_or("")
            .starts_with("application/json")
        {
            return Err(anyhow::anyhow!(
                "Unexpected content type: {:?}",
                response
                    .headers()
                    .get("Content-Type")
                    .unwrap_or(&HeaderValue::from_static("(empty)"))
                    .to_str()
                    .unwrap_or("(non-ascii)")
            ));
        }

        let response: MeetingTimesApiResponse =
            response.json().await.context("Failed to parse response")?;

        Ok(response
            .fmt
            .into_iter()
            .map(|m| m.schedule_info())
            .collect())
    }

    /// Performs a search for courses.
    pub async fn search(
        &self,
        term: &str,
        query: &SearchQuery,
        sort: &str,
        sort_descending: bool,
    ) -> Result<SearchResult, BannerApiError> {
        // self.sessions.reset_data_form().await?;

        let session = self.sessions.acquire(term.parse()?).await?;
        let mut params = query.to_params();

        // Add additional parameters
        params.insert("txt_term".to_string(), term.to_string());
        params.insert("uniqueSessionId".to_string(), session.id());
        params.insert("sortColumn".to_string(), sort.to_string());
        params.insert(
            "sortDirection".to_string(),
            if sort_descending { "desc" } else { "asc" }.to_string(),
        );
        params.insert("startDatepicker".to_string(), String::new());
        params.insert("endDatepicker".to_string(), String::new());

        if session.been_used() {
            self.http
                .post(&format!("{}/classSearch/resetDataForm", self.base_url))
                .send()
                .await
                .map_err(|e| BannerApiError::RequestFailed(e.into()))?;
        }

        debug!(
            term = term,
            query = ?query,
            sort = sort,
            sort_descending = sort_descending,
            "Searching for courses with params: {:?}", params);

        let response = self
            .http
            .get(format!("{}/searchResults/searchResults", self.base_url))
            .header("Cookie", session.cookie())
            .query(&params)
            .send()
            .await
            .context("Failed to search courses")?;

        let status = response.status();
        let url = response.url().clone();
        let body = response
            .text()
            .await
            .with_context(|| format!("Failed to read body (status={status})"))?;

        let search_result: SearchResult = parse_json_with_context(&body).map_err(|e| {
            BannerApiError::RequestFailed(anyhow!(
                "Failed to parse search response (status={status}, url={url}): {e}\nBody: {body}"
            ))
        })?;

        // Check for signs of an invalid session, based on docs/Sessions.md
        if search_result.path_mode.is_none() {
            return Err(BannerApiError::InvalidSession(
                "Search result path mode is none".to_string(),
            ));
        } else if search_result.data.is_none() {
            return Err(BannerApiError::InvalidSession(
                "Search result data is none".to_string(),
            ));
        }

        if !search_result.success {
            return Err(BannerApiError::RequestFailed(anyhow!(
                "Search marked as unsuccessful by Banner API"
            )));
        }

        Ok(search_result)
    }

    /// Retrieves a single course by CRN by issuing a minimal search
    pub async fn get_course_by_crn(
        &self,
        term: &str,
        crn: &str,
    ) -> Result<Option<Course>, BannerApiError> {
        // self.sessions.reset_data_form().await?;
        // Ensure session is configured for this term
        // self.select_term(term).await?;

        let session = self.sessions.acquire(term.parse()?).await?;

        let query = SearchQuery::new()
            .course_reference_number(crn)
            .max_results(1);

        let mut params = query.to_params();
        params.insert("txt_term".to_string(), term.to_string());
        params.insert("uniqueSessionId".to_string(), session.id());
        params.insert("sortColumn".to_string(), "subjectDescription".to_string());
        params.insert("sortDirection".to_string(), "asc".to_string());
        params.insert("startDatepicker".to_string(), String::new());
        params.insert("endDatepicker".to_string(), String::new());

        let url = format!("{}/searchResults/searchResults", self.base_url);
        let response = self
            .http
            .get(&url)
            .header("Cookie", session.cookie())
            .query(&params)
            .send()
            .await
            .context("Failed to search course by CRN")?;

        let status = response.status();
        let url = response.url().clone();
        let body = response
            .text()
            .await
            .with_context(|| format!("Failed to read body (status={status})"))?;

        let search_result: SearchResult = parse_json_with_context(&body).map_err(|e| {
            BannerApiError::RequestFailed(anyhow!(
                "Failed to parse search response for CRN (status={status}, url={url}): {e}"
            ))
        })?;

        // Check for signs of an invalid session, based on docs/Sessions.md
        if search_result.path_mode == Some("registration".to_string())
            && search_result.data.is_none()
        {
            return Err(BannerApiError::InvalidSession(
                "Search result path mode is registration and data is none".to_string(),
            ));
        }

        if !search_result.success {
            return Err(BannerApiError::RequestFailed(anyhow!(
                "Search marked as unsuccessful by Banner API"
            )));
        }

        Ok(search_result
            .data
            .and_then(|courses| courses.into_iter().next()))
    }

    /// Gets course details (placeholder - needs implementation).
    pub async fn get_course_details(&self, term: &str, crn: &str) -> Result<ClassDetails> {
        let body = serde_json::json!({
            "term": term,
            "courseReferenceNumber": crn,
            "first": "first"
        });

        let url = format!("{}/searchResults/getClassDetails", self.base_url);
        let response = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await
            .context("Failed to get course details")?;

        let details: ClassDetails = response
            .json()
            .await
            .context("Failed to parse course details response")?;

        Ok(details)
    }
}

/// Attempt to parse JSON and, on failure, include a contextual snippet of the
/// line where the error occurred. This prevents dumping huge JSON bodies to logs.
fn parse_json_with_context<T: serde::de::DeserializeOwned>(body: &str) -> Result<T> {
    match serde_json::from_str::<T>(body) {
        Ok(value) => Ok(value),
        Err(err) => {
            let (line, column) = (err.line(), err.column());
            let snippet = build_error_snippet(body, line, column, 80);
            Err(anyhow::anyhow!(
                "{err} at line {line}, column {column}\nSnippet:\n{snippet}",
            ))
        }
    }
}

fn build_error_snippet(body: &str, line: usize, column: usize, context_len: usize) -> String {
    let target_line = body.lines().nth(line.saturating_sub(1)).unwrap_or("");
    if target_line.is_empty() {
        return "(empty line)".to_string();
    }

    // column is 1-based, convert to 0-based for slicing
    let error_idx = column.saturating_sub(1);

    let half_len = context_len / 2;
    let start = error_idx.saturating_sub(half_len);
    let end = (error_idx + half_len).min(target_line.len());

    let slice = &target_line[start..end];
    let indicator_pos = error_idx - start;

    let indicator = " ".repeat(indicator_pos) + "^";

    format!("...{slice}...\n   {indicator}")
}
