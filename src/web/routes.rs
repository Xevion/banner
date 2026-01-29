//! Web API endpoints for Banner bot monitoring and metrics.

use axum::{
    Router,
    body::Body,
    extract::{Path, Query, Request, State},
    http::StatusCode as AxumStatusCode,
    response::{Json, Response},
    routing::get,
};
#[cfg(feature = "embed-assets")]
use axum::{
    http::{HeaderMap, HeaderValue, StatusCode, Uri},
    response::{Html, IntoResponse},
};
#[cfg(feature = "embed-assets")]
use http::header;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::BTreeMap, time::Duration};

use crate::state::AppState;
use crate::status::ServiceStatus;
#[cfg(not(feature = "embed-assets"))]
use tower_http::cors::{Any, CorsLayer};
use tower_http::{classify::ServerErrorsFailureClass, timeout::TimeoutLayer, trace::TraceLayer};
use tracing::{Span, debug, trace, warn};

#[cfg(feature = "embed-assets")]
use crate::web::assets::{WebAssets, get_asset_metadata_cached};

/// Set appropriate caching headers based on asset type
#[cfg(feature = "embed-assets")]
fn set_caching_headers(response: &mut Response, path: &str, etag: &str) {
    let headers = response.headers_mut();

    // Set ETag
    if let Ok(etag_value) = HeaderValue::from_str(etag) {
        headers.insert(header::ETAG, etag_value);
    }

    // Set Cache-Control based on asset type
    let cache_control = if path.starts_with("assets/") {
        // Static assets with hashed filenames - long-term cache
        "public, max-age=31536000, immutable"
    } else if path == "index.html" {
        // HTML files - short-term cache
        "public, max-age=300"
    } else {
        match path.split_once('.').map(|(_, extension)| extension) {
            Some(ext) => match ext {
                // CSS/JS files - medium-term cache
                "css" | "js" => "public, max-age=86400",
                // Images - long-term cache
                "png" | "jpg" | "jpeg" | "gif" | "svg" | "ico" => "public, max-age=2592000",
                // Default for other files
                _ => "public, max-age=3600",
            },
            // Default for files without an extension
            None => "public, max-age=3600",
        }
    };

    if let Ok(cache_control_value) = HeaderValue::from_str(cache_control) {
        headers.insert(header::CACHE_CONTROL, cache_control_value);
    }
}

/// Creates the web server router
pub fn create_router(app_state: AppState) -> Router {
    let api_router = Router::new()
        .route("/health", get(health))
        .route("/status", get(status))
        .route("/metrics", get(metrics))
        .route("/courses/search", get(search_courses))
        .route("/courses/{term}/{crn}", get(get_course))
        .route("/terms", get(get_terms))
        .route("/subjects", get(get_subjects))
        .route("/reference/{category}", get(get_reference))
        .with_state(app_state);

    let mut router = Router::new().nest("/api", api_router);

    // When embed-assets feature is enabled, serve embedded static assets
    #[cfg(feature = "embed-assets")]
    {
        router = router.fallback(fallback);
    }

    // Without embed-assets, enable CORS for dev proxy to Vite
    #[cfg(not(feature = "embed-assets"))]
    {
        router = router.layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );
    }

    router.layer((
        TraceLayer::new_for_http()
            .make_span_with(|request: &Request<Body>| {
                tracing::debug_span!("request", path = request.uri().path())
            })
            .on_request(())
            .on_body_chunk(())
            .on_eos(())
            .on_response(
                |response: &Response<Body>, latency: Duration, _span: &Span| {
                    let latency_threshold = if cfg!(debug_assertions) {
                        Duration::from_millis(100)
                    } else {
                        Duration::from_millis(1000)
                    };

                    // Format latency, status, and code
                    let (latency_str, status) = (
                        format!("{latency:.2?}"),
                        format!(
                            "{} {}",
                            response.status().as_u16(),
                            response.status().canonical_reason().unwrap_or("??")
                        ),
                    );

                    // Log in warn if latency is above threshold, otherwise debug
                    if latency > latency_threshold {
                        warn!(latency = latency_str, status = status, "Response");
                    } else {
                        debug!(latency = latency_str, status = status, "Response");
                    }
                },
            )
            .on_failure(
                |error: ServerErrorsFailureClass, latency: Duration, _span: &Span| {
                    warn!(
                        error = ?error,
                        latency = format!("{latency:.2?}"),
                        "Request failed"
                    );
                },
            ),
        TimeoutLayer::new(Duration::from_secs(10)),
    ))
}

/// Handler that extracts request information for caching
#[cfg(feature = "embed-assets")]
async fn fallback(request: Request) -> Response {
    let uri = request.uri().clone();
    let headers = request.headers().clone();
    handle_spa_fallback_with_headers(uri, headers).await
}

/// Handles SPA routing by serving index.html for non-API, non-asset requests
/// This version includes HTTP caching headers and ETag support
#[cfg(feature = "embed-assets")]
async fn handle_spa_fallback_with_headers(uri: Uri, request_headers: HeaderMap) -> Response {
    let path = uri.path().trim_start_matches('/');

    if let Some(content) = WebAssets::get(path) {
        // Get asset metadata (MIME type and hash) with caching
        let metadata = get_asset_metadata_cached(path, &content.data);

        // Check if client has a matching ETag (conditional request)
        if let Some(etag) = request_headers.get(header::IF_NONE_MATCH)
            && etag.to_str().is_ok_and(|s| metadata.etag_matches(s))
        {
            return StatusCode::NOT_MODIFIED.into_response();
        }

        // Use cached MIME type, only set Content-Type if we have a valid MIME type
        let mut response = (
            [(
                header::CONTENT_TYPE,
                // For unknown types, set to application/octet-stream
                metadata
                    .mime_type
                    .unwrap_or("application/octet-stream".to_string()),
            )],
            content.data,
        )
            .into_response();

        // Set caching headers
        set_caching_headers(&mut response, path, &metadata.hash.quoted());

        return response;
    } else {
        // Any assets that are not found should be treated as a 404, not falling back to the SPA index.html
        if path.starts_with("assets/") {
            return (StatusCode::NOT_FOUND, "Asset not found").into_response();
        }
    }

    // Fall back to the SPA index.html
    match WebAssets::get("index.html") {
        Some(content) => {
            let metadata = get_asset_metadata_cached("index.html", &content.data);

            // Check if client has a matching ETag for index.html
            if let Some(etag) = request_headers.get(header::IF_NONE_MATCH)
                && etag.to_str().is_ok_and(|s| metadata.etag_matches(s))
            {
                return StatusCode::NOT_MODIFIED.into_response();
            }

            let mut response = Html(content.data).into_response();
            set_caching_headers(&mut response, "index.html", &metadata.hash.quoted());
            response
        }
        None => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to load index.html",
        )
            .into_response(),
    }
}

/// Health check endpoint
async fn health() -> Json<Value> {
    trace!("health check requested");
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

#[derive(Serialize)]
struct ServiceInfo {
    name: String,
    status: ServiceStatus,
}

#[derive(Serialize)]
struct StatusResponse {
    status: ServiceStatus,
    version: String,
    commit: String,
    services: BTreeMap<String, ServiceInfo>,
}

/// Status endpoint showing bot and system status
async fn status(State(state): State<AppState>) -> Json<StatusResponse> {
    let mut services = BTreeMap::new();

    for (name, svc_status) in state.service_statuses.all() {
        services.insert(
            name.clone(),
            ServiceInfo {
                name,
                status: svc_status,
            },
        );
    }

    let overall_status = if services
        .values()
        .any(|s| matches!(s.status, ServiceStatus::Error))
    {
        ServiceStatus::Error
    } else if !services.is_empty()
        && services
            .values()
            .all(|s| matches!(s.status, ServiceStatus::Active | ServiceStatus::Connected))
    {
        ServiceStatus::Active
    } else if services.is_empty() {
        ServiceStatus::Disabled
    } else {
        ServiceStatus::Active
    };

    Json(StatusResponse {
        status: overall_status,
        version: env!("CARGO_PKG_VERSION").to_string(),
        commit: env!("GIT_COMMIT_HASH").to_string(),
        services,
    })
}

/// Metrics endpoint for monitoring
async fn metrics() -> Json<Value> {
    // For now, return basic metrics structure
    Json(json!({
        "banner_api": {
            "status": "connected"
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

// ============================================================
// Course search & detail API
// ============================================================

#[derive(Deserialize)]
struct SearchParams {
    term: String,
    subject: Option<String>,
    q: Option<String>,
    course_number_low: Option<i32>,
    course_number_high: Option<i32>,
    #[serde(default)]
    open_only: bool,
    instructional_method: Option<String>,
    campus: Option<String>,
    #[serde(default = "default_limit")]
    limit: i32,
    #[serde(default)]
    offset: i32,
}

fn default_limit() -> i32 {
    25
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CourseResponse {
    crn: String,
    subject: String,
    course_number: String,
    title: String,
    term_code: String,
    sequence_number: Option<String>,
    instructional_method: Option<String>,
    campus: Option<String>,
    enrollment: i32,
    max_enrollment: i32,
    wait_count: i32,
    wait_capacity: i32,
    credit_hours: Option<i32>,
    credit_hour_low: Option<i32>,
    credit_hour_high: Option<i32>,
    cross_list: Option<String>,
    cross_list_capacity: Option<i32>,
    cross_list_count: Option<i32>,
    link_identifier: Option<String>,
    is_section_linked: Option<bool>,
    part_of_term: Option<String>,
    meeting_times: Value,
    attributes: Value,
    instructors: Vec<InstructorResponse>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InstructorResponse {
    banner_id: String,
    display_name: String,
    email: Option<String>,
    is_primary: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchResponse {
    courses: Vec<CourseResponse>,
    total_count: i64,
    offset: i32,
    limit: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CodeDescription {
    code: String,
    description: String,
}

/// Build a `CourseResponse` from a DB course, fetching its instructors.
async fn build_course_response(
    course: &crate::data::models::Course,
    db_pool: &sqlx::PgPool,
) -> CourseResponse {
    let instructors = crate::data::courses::get_course_instructors(db_pool, course.id)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(
            |(banner_id, display_name, email, is_primary)| InstructorResponse {
                banner_id,
                display_name,
                email,
                is_primary,
            },
        )
        .collect();

    CourseResponse {
        crn: course.crn.clone(),
        subject: course.subject.clone(),
        course_number: course.course_number.clone(),
        title: course.title.clone(),
        term_code: course.term_code.clone(),
        sequence_number: course.sequence_number.clone(),
        instructional_method: course.instructional_method.clone(),
        campus: course.campus.clone(),
        enrollment: course.enrollment,
        max_enrollment: course.max_enrollment,
        wait_count: course.wait_count,
        wait_capacity: course.wait_capacity,
        credit_hours: course.credit_hours,
        credit_hour_low: course.credit_hour_low,
        credit_hour_high: course.credit_hour_high,
        cross_list: course.cross_list.clone(),
        cross_list_capacity: course.cross_list_capacity,
        cross_list_count: course.cross_list_count,
        link_identifier: course.link_identifier.clone(),
        is_section_linked: course.is_section_linked,
        part_of_term: course.part_of_term.clone(),
        meeting_times: course.meeting_times.clone(),
        attributes: course.attributes.clone(),
        instructors,
    }
}

/// `GET /api/courses/search`
async fn search_courses(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<SearchResponse>, (AxumStatusCode, String)> {
    let limit = params.limit.clamp(1, 100);
    let offset = params.offset.max(0);

    let (courses, total_count) = crate::data::courses::search_courses(
        &state.db_pool,
        &params.term,
        params.subject.as_deref(),
        params.q.as_deref(),
        params.course_number_low,
        params.course_number_high,
        params.open_only,
        params.instructional_method.as_deref(),
        params.campus.as_deref(),
        limit,
        offset,
    )
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "Course search failed");
        (
            AxumStatusCode::INTERNAL_SERVER_ERROR,
            "Search failed".to_string(),
        )
    })?;

    let mut course_responses = Vec::with_capacity(courses.len());
    for course in &courses {
        course_responses.push(build_course_response(course, &state.db_pool).await);
    }

    Ok(Json(SearchResponse {
        courses: course_responses,
        total_count,
        offset,
        limit,
    }))
}

/// `GET /api/courses/:term/:crn`
async fn get_course(
    State(state): State<AppState>,
    Path((term, crn)): Path<(String, String)>,
) -> Result<Json<CourseResponse>, (AxumStatusCode, String)> {
    let course = crate::data::courses::get_course_by_crn(&state.db_pool, &crn, &term)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Course lookup failed");
            (
                AxumStatusCode::INTERNAL_SERVER_ERROR,
                "Lookup failed".to_string(),
            )
        })?
        .ok_or_else(|| (AxumStatusCode::NOT_FOUND, "Course not found".to_string()))?;

    Ok(Json(build_course_response(&course, &state.db_pool).await))
}

/// `GET /api/terms`
async fn get_terms(
    State(state): State<AppState>,
) -> Result<Json<Vec<CodeDescription>>, (AxumStatusCode, String)> {
    let cache = state.reference_cache.read().await;
    let term_codes = crate::data::courses::get_available_terms(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to get terms");
            (
                AxumStatusCode::INTERNAL_SERVER_ERROR,
                "Failed to get terms".to_string(),
            )
        })?;

    let terms: Vec<CodeDescription> = term_codes
        .into_iter()
        .map(|code| {
            let description = cache
                .lookup("term", &code)
                .unwrap_or("Unknown Term")
                .to_string();
            CodeDescription { code, description }
        })
        .collect();

    Ok(Json(terms))
}

/// `GET /api/subjects?term=202420`
async fn get_subjects(
    State(state): State<AppState>,
) -> Result<Json<Vec<CodeDescription>>, (AxumStatusCode, String)> {
    let cache = state.reference_cache.read().await;
    let entries = cache.entries_for_category("subject");

    let subjects: Vec<CodeDescription> = entries
        .into_iter()
        .map(|(code, description)| CodeDescription {
            code: code.to_string(),
            description: description.to_string(),
        })
        .collect();

    Ok(Json(subjects))
}

/// `GET /api/reference/:category`
async fn get_reference(
    State(state): State<AppState>,
    Path(category): Path<String>,
) -> Result<Json<Vec<CodeDescription>>, (AxumStatusCode, String)> {
    let cache = state.reference_cache.read().await;
    let entries = cache.entries_for_category(&category);

    if entries.is_empty() {
        // Fall back to DB query in case cache doesn't have this category
        drop(cache);
        let rows = crate::data::reference::get_by_category(&category, &state.db_pool)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, category = %category, "Reference lookup failed");
                (
                    AxumStatusCode::INTERNAL_SERVER_ERROR,
                    "Lookup failed".to_string(),
                )
            })?;

        return Ok(Json(
            rows.into_iter()
                .map(|r| CodeDescription {
                    code: r.code,
                    description: r.description,
                })
                .collect(),
        ));
    }

    Ok(Json(
        entries
            .into_iter()
            .map(|(code, desc)| CodeDescription {
                code: code.to_string(),
                description: desc.to_string(),
            })
            .collect(),
    ))
}
