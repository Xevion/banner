//! Web API endpoints for Banner bot monitoring and metrics.

use axum::{
    Extension, Router,
    body::Body,
    extract::{Path, Query, Request, State},
    response::{Json, Response},
    routing::{get, post, put},
};

use crate::web::admin_scraper;
use crate::web::auth::{self, AuthConfig};
use crate::web::calendar;
use crate::web::error::{ApiError, db_error};
use crate::web::timeline;
use crate::web::ws;
use crate::{data, web::admin};
use crate::{data::models, web::admin_rmp};
#[cfg(feature = "embed-assets")]
use axum::{
    http::{HeaderMap, StatusCode, Uri},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::BTreeMap, time::Duration};
use ts_rs::TS;

use crate::state::AppState;
use crate::status::ServiceStatus;
#[cfg(not(feature = "embed-assets"))]
use tower_http::cors::{Any, CorsLayer};
use tower_http::{
    classify::ServerErrorsFailureClass, compression::CompressionLayer, timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::{Span, debug, trace, warn};

#[cfg(feature = "embed-assets")]
use crate::web::assets::try_serve_asset_with_encoding;

/// Creates the web server router
pub fn create_router(app_state: AppState, auth_config: AuthConfig) -> Router {
    let api_router = Router::new()
        .route("/health", get(health))
        .route("/status", get(status))
        .route("/metrics", get(metrics))
        .route("/courses/search", get(search_courses))
        .route("/courses/{term}/{crn}", get(get_course))
        .route(
            "/courses/{term}/{crn}/calendar.ics",
            get(calendar::course_ics),
        )
        .route("/courses/{term}/{crn}/gcal", get(calendar::course_gcal))
        .route("/terms", get(get_terms))
        .route("/subjects", get(get_subjects))
        .route("/reference/{category}", get(get_reference))
        .route("/timeline", post(timeline::timeline))
        .with_state(app_state.clone());

    let auth_router = Router::new()
        .route("/auth/login", get(auth::auth_login))
        .route("/auth/callback", get(auth::auth_callback))
        .route("/auth/logout", post(auth::auth_logout))
        .route("/auth/me", get(auth::auth_me))
        .layer(Extension(auth_config))
        .with_state(app_state.clone());

    let admin_router = Router::new()
        .route("/admin/status", get(admin::admin_status))
        .route("/admin/users", get(admin::list_users))
        .route(
            "/admin/users/{discord_id}/admin",
            put(admin::set_user_admin),
        )
        .route("/admin/scrape-jobs", get(admin::list_scrape_jobs))
        .route("/admin/scrape-jobs/ws", get(ws::scrape_jobs_ws))
        .route("/admin/audit-log", get(admin::list_audit_log))
        .route("/admin/instructors", get(admin_rmp::list_instructors))
        .route("/admin/instructors/{id}", get(admin_rmp::get_instructor))
        .route(
            "/admin/instructors/{id}/match",
            post(admin_rmp::match_instructor),
        )
        .route(
            "/admin/instructors/{id}/reject-candidate",
            post(admin_rmp::reject_candidate),
        )
        .route(
            "/admin/instructors/{id}/reject-all",
            post(admin_rmp::reject_all),
        )
        .route(
            "/admin/instructors/{id}/unmatch",
            post(admin_rmp::unmatch_instructor),
        )
        .route("/admin/rmp/rescore", post(admin_rmp::rescore))
        .route("/admin/scraper/stats", get(admin_scraper::scraper_stats))
        .route(
            "/admin/scraper/timeseries",
            get(admin_scraper::scraper_timeseries),
        )
        .route(
            "/admin/scraper/subjects",
            get(admin_scraper::scraper_subjects),
        )
        .route(
            "/admin/scraper/subjects/{subject}",
            get(admin_scraper::scraper_subject_detail),
        )
        .with_state(app_state);

    let mut router = Router::new()
        .nest("/api", api_router)
        .nest("/api", auth_router)
        .nest("/api", admin_router);

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
        // Compress API responses (gzip/brotli/zstd). Pre-compressed static
        // assets already have Content-Encoding set, so tower-http skips them.
        CompressionLayer::new()
            .zstd(true)
            .br(true)
            .gzip(true)
            .quality(tower_http::CompressionLevel::Fastest),
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

/// SPA fallback handler with content encoding negotiation.
///
/// Serves embedded static assets with pre-compressed variants when available,
/// falling back to `index.html` for SPA client-side routing.
#[cfg(feature = "embed-assets")]
async fn fallback(request: Request) -> axum::response::Response {
    let uri = request.uri().clone();
    let headers = request.headers().clone();
    handle_spa_fallback(uri, headers).await
}

#[cfg(feature = "embed-assets")]
async fn handle_spa_fallback(uri: Uri, request_headers: HeaderMap) -> axum::response::Response {
    let path = uri.path();

    // Try serving the exact asset (with encoding negotiation)
    if let Some(response) = try_serve_asset_with_encoding(path, &request_headers) {
        return response;
    }

    // SvelteKit assets under _app/ that don't exist are a hard 404
    let trimmed = path.trim_start_matches('/');
    if trimmed.starts_with("_app/") || trimmed.starts_with("assets/") {
        return (StatusCode::NOT_FOUND, "Asset not found").into_response();
    }

    // SPA fallback: serve index.html with encoding negotiation
    match try_serve_asset_with_encoding("/index.html", &request_headers) {
        Some(response) => response,
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

#[derive(Serialize, TS)]
#[ts(export)]
pub struct ServiceInfo {
    name: String,
    status: ServiceStatus,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct StatusResponse {
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
async fn metrics(
    State(state): State<AppState>,
    Query(params): Query<MetricsParams>,
) -> Result<Json<MetricsResponse>, ApiError> {
    let limit = params.limit.clamp(1, 5000);

    // Parse range shorthand, defaulting to 24h
    let range_str = params.range.as_deref().unwrap_or("24h");
    let duration = match range_str {
        "1h" => chrono::Duration::hours(1),
        "6h" => chrono::Duration::hours(6),
        "24h" => chrono::Duration::hours(24),
        "7d" => chrono::Duration::days(7),
        "30d" => chrono::Duration::days(30),
        _ => {
            return Err(ApiError::new(
                "INVALID_RANGE",
                format!("Invalid range '{range_str}'. Valid: 1h, 6h, 24h, 7d, 30d"),
            ));
        }
    };
    let since = chrono::Utc::now() - duration;

    // Resolve course_id: explicit param takes priority, then term+crn lookup
    let course_id = if let Some(id) = params.course_id {
        Some(id)
    } else if let (Some(term), Some(crn)) = (params.term.as_deref(), params.crn.as_deref()) {
        let row: Option<(i32,)> =
            sqlx::query_as("SELECT id FROM courses WHERE term_code = $1 AND crn = $2")
                .bind(term)
                .bind(crn)
                .fetch_optional(&state.db_pool)
                .await
                .map_err(|e| db_error("Course lookup for metrics", e.into()))?;
        row.map(|(id,)| id)
    } else {
        None
    };

    // Build query dynamically based on filters
    let metrics: Vec<(i32, i32, chrono::DateTime<chrono::Utc>, i32, i32, i32)> =
        if let Some(cid) = course_id {
            sqlx::query_as(
                "SELECT id, course_id, timestamp, enrollment, wait_count, seats_available \
             FROM course_metrics \
             WHERE course_id = $1 AND timestamp >= $2 \
             ORDER BY timestamp DESC \
             LIMIT $3",
            )
            .bind(cid)
            .bind(since)
            .bind(limit)
            .fetch_all(&state.db_pool)
            .await
        } else {
            sqlx::query_as(
                "SELECT id, course_id, timestamp, enrollment, wait_count, seats_available \
             FROM course_metrics \
             WHERE timestamp >= $1 \
             ORDER BY timestamp DESC \
             LIMIT $2",
            )
            .bind(since)
            .bind(limit)
            .fetch_all(&state.db_pool)
            .await
        }
        .map_err(|e| db_error("Metrics query", e.into()))?;

    let count = metrics.len();
    let metrics_entries: Vec<MetricEntry> = metrics
        .into_iter()
        .map(
            |(id, course_id, timestamp, enrollment, wait_count, seats_available)| MetricEntry {
                id,
                course_id,
                timestamp: timestamp.to_rfc3339(),
                enrollment,
                wait_count,
                seats_available,
            },
        )
        .collect();

    Ok(Json(MetricsResponse {
        metrics: metrics_entries,
        count,
        timestamp: chrono::Utc::now().to_rfc3339(),
    }))
}

// ============================================================
// Course search & detail API
// ============================================================

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MetricEntry {
    pub id: i32,
    pub course_id: i32,
    pub timestamp: String,
    pub enrollment: i32,
    pub wait_count: i32,
    pub seats_available: i32,
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MetricsResponse {
    pub metrics: Vec<MetricEntry>,
    pub count: usize,
    pub timestamp: String,
}

#[derive(Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MetricsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub course_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub term: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<String>,
    #[serde(default = "default_metrics_limit")]
    pub limit: i32,
}

fn default_metrics_limit() -> i32 {
    500
}

#[derive(Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SubjectsParams {
    pub term: String,
}

#[derive(Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SearchParams {
    pub term: String,
    #[serde(default)]
    pub subject: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub q: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub course_number_low: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub course_number_high: Option<i32>,
    #[serde(default)]
    pub open_only: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructional_method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub campus: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i32,
    #[serde(default)]
    pub offset: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<SortColumn>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_dir: Option<SortDirection>,
}

use crate::data::courses::{SortColumn, SortDirection};

fn default_limit() -> i32 {
    25
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CourseResponse {
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
    meeting_times: Vec<models::DbMeetingTime>,
    attributes: Vec<String>,
    instructors: Vec<InstructorResponse>,
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct InstructorResponse {
    instructor_id: i32,
    banner_id: String,
    display_name: String,
    email: String,
    is_primary: bool,
    rmp_rating: Option<f32>,
    rmp_num_ratings: Option<i32>,
    rmp_legacy_id: Option<i32>,
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SearchResponse {
    courses: Vec<CourseResponse>,
    total_count: i32,
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CodeDescription {
    code: String,
    description: String,
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TermResponse {
    code: String,
    slug: String,
    description: String,
}

/// Build a `CourseResponse` from a DB course with pre-fetched instructor details.
fn build_course_response(
    course: &models::Course,
    instructors: Vec<models::CourseInstructorDetail>,
) -> CourseResponse {
    let instructors = instructors
        .into_iter()
        .map(|i| InstructorResponse {
            instructor_id: i.instructor_id,
            banner_id: i.banner_id,
            display_name: i.display_name,
            email: i.email,
            is_primary: i.is_primary,
            rmp_rating: i.avg_rating,
            rmp_num_ratings: i.num_ratings,
            rmp_legacy_id: i.rmp_legacy_id,
        })
        .collect();

    let meeting_times = serde_json::from_value(course.meeting_times.clone())
        .map_err(|e| {
            tracing::error!(
                course_id = course.id,
                crn = %course.crn,
                term = %course.term_code,
                error = %e,
                "Failed to deserialize meeting_times JSONB"
            );
            e
        })
        .unwrap_or_default();

    let attributes = serde_json::from_value(course.attributes.clone())
        .map_err(|e| {
            tracing::error!(
                course_id = course.id,
                crn = %course.crn,
                term = %course.term_code,
                error = %e,
                "Failed to deserialize attributes JSONB"
            );
            e
        })
        .unwrap_or_default();

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
        meeting_times,
        attributes,
        instructors,
    }
}

/// `GET /api/courses/search`
async fn search_courses(
    State(state): State<AppState>,
    axum_extra::extract::Query(params): axum_extra::extract::Query<SearchParams>,
) -> Result<Json<SearchResponse>, ApiError> {
    use crate::banner::models::terms::Term;

    let term_code =
        Term::resolve_to_code(&params.term).ok_or_else(|| ApiError::invalid_term(&params.term))?;
    let limit = params.limit.clamp(1, 100);
    let offset = params.offset.max(0);

    let (courses, total_count) = data::courses::search_courses(
        &state.db_pool,
        &term_code,
        if params.subject.is_empty() {
            None
        } else {
            Some(&params.subject)
        },
        params.q.as_deref(),
        params.course_number_low,
        params.course_number_high,
        params.open_only,
        params.instructional_method.as_deref(),
        params.campus.as_deref(),
        limit,
        offset,
        params.sort_by,
        params.sort_dir,
    )
    .await
    .map_err(|e| db_error("Course search", e))?;

    // Batch-fetch all instructors in a single query instead of N+1
    let course_ids: Vec<i32> = courses.iter().map(|c| c.id).collect();
    let mut instructor_map =
        data::courses::get_instructors_for_courses(&state.db_pool, &course_ids)
            .await
            .unwrap_or_default();

    let course_responses: Vec<CourseResponse> = courses
        .iter()
        .map(|course| {
            let instructors = instructor_map.remove(&course.id).unwrap_or_default();
            build_course_response(course, instructors)
        })
        .collect();

    Ok(Json(SearchResponse {
        courses: course_responses,
        total_count: total_count as i32,
    }))
}

/// `GET /api/courses/:term/:crn`
async fn get_course(
    State(state): State<AppState>,
    Path((term, crn)): Path<(String, String)>,
) -> Result<Json<CourseResponse>, ApiError> {
    let course = data::courses::get_course_by_crn(&state.db_pool, &crn, &term)
        .await
        .map_err(|e| db_error("Course lookup", e))?
        .ok_or_else(|| ApiError::not_found("Course not found"))?;

    let instructors = data::courses::get_course_instructors(&state.db_pool, course.id)
        .await
        .unwrap_or_default();
    Ok(Json(build_course_response(&course, instructors)))
}

/// `GET /api/terms`
async fn get_terms(State(state): State<AppState>) -> Result<Json<Vec<TermResponse>>, ApiError> {
    use crate::banner::models::terms::Term;

    let term_codes = data::courses::get_available_terms(&state.db_pool)
        .await
        .map_err(|e| db_error("Get terms", e))?;

    let terms: Vec<TermResponse> = term_codes
        .into_iter()
        .filter_map(|code| {
            let term: Term = code.parse().ok()?;
            Some(TermResponse {
                code,
                slug: term.slug(),
                description: term.description(),
            })
        })
        .collect();

    Ok(Json(terms))
}

/// `GET /api/subjects?term=202620`
async fn get_subjects(
    State(state): State<AppState>,
    Query(params): Query<SubjectsParams>,
) -> Result<Json<Vec<CodeDescription>>, ApiError> {
    use crate::banner::models::terms::Term;

    let term_code =
        Term::resolve_to_code(&params.term).ok_or_else(|| ApiError::invalid_term(&params.term))?;
    let rows = data::courses::get_subjects_by_enrollment(&state.db_pool, &term_code)
        .await
        .map_err(|e| db_error("Get subjects", e))?;

    let subjects: Vec<CodeDescription> = rows
        .into_iter()
        .map(|(code, description, _enrollment)| CodeDescription { code, description })
        .collect();

    Ok(Json(subjects))
}

/// `GET /api/reference/:category`
async fn get_reference(
    State(state): State<AppState>,
    Path(category): Path<String>,
) -> Result<Json<Vec<CodeDescription>>, ApiError> {
    let cache = state.reference_cache.read().await;
    let entries = cache.entries_for_category(&category);

    if entries.is_empty() {
        // Fall back to DB query in case cache doesn't have this category
        drop(cache);
        let rows = data::reference::get_by_category(&category, &state.db_pool)
            .await
            .map_err(|e| db_error(&format!("Reference lookup for {}", category), e))?;

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
