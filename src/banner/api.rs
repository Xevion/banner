//! Main Banner API client implementation.

use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
    time::Instant,
};

use crate::banner::{
    BannerSession, SessionPool, errors::BannerApiError, json::parse_json_with_context,
    middleware::TransparentMiddleware, models::*, nonce, query::SearchQuery, util::user_agent,
};
use anyhow::{Context, Result, anyhow};
use cookie::Cookie;
use dashmap::DashMap;
use http::HeaderValue;
use reqwest::{Client, Request, Response};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use serde_json;
use tl;
use tracing::{Level, Metadata, Span, debug, error, field::ValueSet, info, span, trace, warn};

/// Main Banner API client.
pub struct BannerApi {
    pub sessions: SessionPool,
    http: ClientWithMiddleware,
    base_url: String,
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

    /// Validates offset parameter for search methods.
    fn validate_offset(offset: i32) -> Result<()> {
        if offset <= 0 {
            Err(anyhow::anyhow!("Offset must be greater than 0"))
        } else {
            Ok(())
        }
    }

    /// Builds common search parameters for list endpoints.
    fn build_list_params(
        &self,
        search: &str,
        term: &str,
        offset: i32,
        max_results: i32,
        session_id: &str,
    ) -> Vec<(&str, String)> {
        vec![
            ("searchTerm", search.to_string()),
            ("term", term.to_string()),
            ("offset", offset.to_string()),
            ("max", max_results.to_string()),
            ("uniqueSessionId", session_id.to_string()),
            ("_", nonce()),
        ]
    }

    /// Makes a GET request to a list endpoint and parses JSON response.
    async fn get_list_endpoint<T>(
        &self,
        endpoint: &str,
        search: &str,
        term: &str,
        offset: i32,
        max_results: i32,
    ) -> Result<Vec<T>>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        Self::validate_offset(offset)?;

        let session = self.sessions.acquire(term.parse()?).await?;
        let url = format!("{}/classSearch/{}", self.base_url, endpoint);
        let params = self.build_list_params(search, term, offset, max_results, &session.id());

        let response = self
            .http
            .get(&url)
            .query(&params)
            .send()
            .await
            .with_context(|| format!("Failed to get {}", endpoint))?;

        let data: Vec<T> = response
            .json()
            .await
            .with_context(|| format!("Failed to parse {} response", endpoint))?;

        Ok(data)
    }

    /// Builds search parameters for course search methods.
    fn build_search_params(
        &self,
        query: &SearchQuery,
        term: &str,
        session_id: &str,
        sort: &str,
        sort_descending: bool,
    ) -> HashMap<String, String> {
        let mut params = query.to_params();
        params.insert("txt_term".to_string(), term.to_string());
        params.insert("uniqueSessionId".to_string(), session_id.to_string());
        params.insert("sortColumn".to_string(), sort.to_string());
        params.insert(
            "sortDirection".to_string(),
            if sort_descending { "desc" } else { "asc" }.to_string(),
        );
        params.insert("startDatepicker".to_string(), String::new());
        params.insert("endDatepicker".to_string(), String::new());
        params
    }

    /// Performs a course search and handles common response processing.
    async fn perform_search(
        &self,
        term: &str,
        query: &SearchQuery,
        sort: &str,
        sort_descending: bool,
    ) -> Result<SearchResult, BannerApiError> {
        let mut session = self.sessions.acquire(term.parse()?).await?;

        if session.been_used() {
            self.http
                .post(format!("{}/classSearch/resetDataForm", self.base_url))
                .header("Cookie", session.cookie())
                .send()
                .await
                .map_err(|e| BannerApiError::RequestFailed(e.into()))?;
        }

        session.touch();

        let params = self.build_search_params(query, term, &session.id(), sort, sort_descending);

        debug!(
            term = term,
            query = ?query,
            sort = sort,
            sort_descending = sort_descending,
            "Searching for courses with params: {:?}", params
        );

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

        // Check for signs of an invalid session
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

    /// Retrieves a list of subjects from the Banner API.
    pub async fn get_subjects(
        &self,
        search: &str,
        term: &str,
        offset: i32,
        max_results: i32,
    ) -> Result<Vec<Pair>> {
        self.get_list_endpoint("get_subject", search, term, offset, max_results)
            .await
    }

    /// Retrieves a list of instructors from the Banner API.
    pub async fn get_instructors(
        &self,
        search: &str,
        term: &str,
        offset: i32,
        max_results: i32,
    ) -> Result<Vec<Instructor>> {
        self.get_list_endpoint("get_instructor", search, term, offset, max_results)
            .await
    }

    /// Retrieves a list of campuses from the Banner API.
    pub async fn get_campuses(
        &self,
        search: &str,
        term: &str,
        offset: i32,
        max_results: i32,
    ) -> Result<Vec<Pair>> {
        self.get_list_endpoint("get_campus", search, term, offset, max_results)
            .await
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
        self.perform_search(term, query, sort, sort_descending)
            .await
    }

    /// Retrieves a single course by CRN by issuing a minimal search
    pub async fn get_course_by_crn(
        &self,
        term: &str,
        crn: &str,
    ) -> Result<Option<Course>, BannerApiError> {
        let query = SearchQuery::new()
            .course_reference_number(crn)
            .max_results(1);

        let search_result = self
            .perform_search(term, &query, "subjectDescription", false)
            .await?;

        // Additional validation for CRN search
        if search_result.path_mode == Some("registration".to_string())
            && search_result.data.is_none()
        {
            return Err(BannerApiError::InvalidSession(
                "Search result path mode is registration and data is none".to_string(),
            ));
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
