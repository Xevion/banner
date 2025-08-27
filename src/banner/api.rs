//! Main Banner API client implementation.

use crate::banner::{SessionManager, models::*, query::SearchQuery};
use anyhow::{Context, Result};
use axum::http::HeaderValue;
use reqwest::Client;
use serde_json;

// use tracing::debug;

/// Main Banner API client.
#[derive(Debug)]
pub struct BannerApi {
    session_manager: SessionManager,
    client: Client,
    base_url: String,
}

impl BannerApi {
    /// Creates a new Banner API client.
    pub fn new(base_url: String) -> Result<Self> {
        let client = Client::builder()
            .cookie_store(true)
            .user_agent(user_agent())
            .tcp_keepalive(Some(std::time::Duration::from_secs(60 * 5)))
            .read_timeout(std::time::Duration::from_secs(10))
            .connect_timeout(std::time::Duration::from_secs(10))
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        let session_manager = SessionManager::new(base_url.clone(), client.clone());

        Ok(Self {
            session_manager,
            client,
            base_url,
        })
    }

    /// Sets up the API client by initializing session cookies.
    pub async fn setup(&self) -> Result<()> {
        self.session_manager.setup().await
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
            ("_", &timestamp_nonce()),
        ];

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .context("Failed to get terms")?;

        let terms: Vec<BannerTerm> = response
            .json()
            .await
            .context("Failed to parse terms response")?;

        Ok(terms)
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

        let session_id = self.session_manager.ensure_session()?;
        let url = format!("{}/classSearch/get_subject", self.base_url);
        let params = [
            ("searchTerm", search),
            ("term", term),
            ("offset", &offset.to_string()),
            ("max", &max_results.to_string()),
            ("uniqueSessionId", &session_id),
            ("_", &timestamp_nonce()),
        ];

        let response = self
            .client
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

        let session_id = self.session_manager.ensure_session()?;
        let url = format!("{}/classSearch/get_instructor", self.base_url);
        let params = [
            ("searchTerm", search),
            ("term", term),
            ("offset", &offset.to_string()),
            ("max", &max_results.to_string()),
            ("uniqueSessionId", &session_id),
            ("_", &timestamp_nonce()),
        ];

        let response = self
            .client
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
        term: i32,
        offset: i32,
        max_results: i32,
    ) -> Result<Vec<Pair>> {
        if offset <= 0 {
            return Err(anyhow::anyhow!("Offset must be greater than 0"));
        }

        let session_id = self.session_manager.ensure_session()?;
        let url = format!("{}/classSearch/get_campus", self.base_url);
        let params = [
            ("searchTerm", search),
            ("term", &term.to_string()),
            ("offset", &offset.to_string()),
            ("max", &max_results.to_string()),
            ("uniqueSessionId", &session_id),
            ("_", &timestamp_nonce()),
        ];

        let response = self
            .client
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
        crn: i32,
    ) -> Result<Vec<MeetingScheduleInfo>> {
        let url = format!("{}/searchResults/getFacultyMeetingTimes", self.base_url);
        let params = [("term", term), ("courseReferenceNumber", &crn.to_string())];

        let response = self
            .client
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

        #[derive(serde::Deserialize)]
        struct ResponseWrapper {
            fmt: Vec<MeetingTimeResponse>,
        }

        let wrapper: ResponseWrapper = response.json().await.context("Failed to parse response")?;

        Ok(wrapper.fmt.into_iter().map(|m| m.schedule_info()).collect())
    }

    /// Performs a search for courses.
    pub async fn search(
        &self,
        term: &str,
        query: &SearchQuery,
        sort: &str,
        sort_descending: bool,
    ) -> Result<SearchResult> {
        self.session_manager.reset_data_form().await?;

        let session_id = self.session_manager.ensure_session()?;
        let mut params = query.to_params();

        // Add additional parameters
        params.insert("txt_term".to_string(), term.to_string());
        params.insert("uniqueSessionId".to_string(), session_id);
        params.insert("sortColumn".to_string(), sort.to_string());
        params.insert(
            "sortDirection".to_string(),
            if sort_descending { "desc" } else { "asc" }.to_string(),
        );
        params.insert("startDatepicker".to_string(), String::new());
        params.insert("endDatepicker".to_string(), String::new());

        let url = format!("{}/searchResults/searchResults", self.base_url);
        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .context("Failed to search courses")?;

        let search_result: SearchResult = response
            .json()
            .await
            .context("Failed to parse search response")?;

        if !search_result.success {
            return Err(anyhow::anyhow!(
                "Search marked as unsuccessful by Banner API"
            ));
        }

        Ok(search_result)
    }

    /// Selects a term for the current session.
    pub async fn select_term(&self, term: &str) -> Result<()> {
        self.session_manager.select_term(term).await
    }

    /// Retrieves a single course by CRN by issuing a minimal search
    pub async fn get_course_by_crn(&self, term: &str, crn: &str) -> Result<Option<Course>> {
        self.session_manager.reset_data_form().await?;
        // Ensure session is configured for this term
        self.select_term(term).await?;

        let session_id = self.session_manager.ensure_session()?;

        let query = SearchQuery::new()
            .course_reference_number(crn)
            .max_results(1);

        let mut params = query.to_params();
        params.insert("txt_term".to_string(), term.to_string());
        params.insert("uniqueSessionId".to_string(), session_id);
        params.insert("sortColumn".to_string(), "subjectDescription".to_string());
        params.insert("sortDirection".to_string(), "asc".to_string());
        params.insert("startDatepicker".to_string(), String::new());
        params.insert("endDatepicker".to_string(), String::new());

        let url = format!("{}/searchResults/searchResults", self.base_url);
        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .context("Failed to search course by CRN")?;

        let status = response.status();
        let body = response
            .text()
            .await
            .with_context(|| format!("Failed to read body (status={status})"))?;

        let search_result: SearchResult = parse_json_with_context(&body).map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse search response for CRN (status={status}, url={url}): {e}",
            )
        })?;

        if !search_result.success {
            return Err(anyhow::anyhow!(
                "Search marked as unsuccessful by Banner API"
            ));
        }

        Ok(search_result
            .data
            .and_then(|courses| courses.into_iter().next()))
    }

    /// Gets course details (placeholder - needs implementation).
    pub async fn get_course_details(&self, term: i32, crn: i32) -> Result<ClassDetails> {
        let body = serde_json::json!({
            "term": term.to_string(),
            "courseReferenceNumber": crn.to_string(),
            "first": "first"
        });

        let url = format!("{}/searchResults/getClassDetails", self.base_url);
        let response = self
            .client
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

/// Generates a timestamp-based nonce.
fn timestamp_nonce() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string()
}

/// Returns a browser-like user agent string.
fn user_agent() -> &'static str {
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.36"
}

/// Attempt to parse JSON and, on failure, include a contextual snippet around the error location
fn parse_json_with_context<T: serde::de::DeserializeOwned>(body: &str) -> Result<T> {
    match serde_json::from_str::<T>(body) {
        Ok(value) => Ok(value),
        Err(err) => {
            let (line, column) = (err.line(), err.column());
            let snippet = build_error_snippet(body, line as usize, column as usize, 120);
            Err(anyhow::anyhow!(
                "{} at line {}, column {}\nSnippet:\n{}",
                err,
                line,
                column,
                snippet
            ))
        }
    }
}

fn build_error_snippet(body: &str, line: usize, column: usize, max_len: usize) -> String {
    let target_line = body.lines().nth(line.saturating_sub(1)).unwrap_or("");
    if target_line.is_empty() {
        return String::new();
    }

    let start = column.saturating_sub(max_len.min(column));
    let end = (column + max_len).min(target_line.len());
    let slice = &target_line[start..end];

    let mut indicator = String::new();
    if column > start {
        indicator.push_str(&" ".repeat(column - start - 1));
        indicator.push('^');
    }

    format!("{}\n{}", slice, indicator)
}
