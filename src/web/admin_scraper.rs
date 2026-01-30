//! Admin API handlers for scraper observability.
//!
//! All endpoints require the `AdminUser` extractor, returning 401/403 as needed.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;

use crate::data::scrape_jobs;
use crate::scraper::adaptive::{self, SubjectSchedule, SubjectStats};
use crate::state::AppState;
use crate::web::extractors::AdminUser;

type ApiError = (StatusCode, Json<serde_json::Value>);

fn parse_period(period: &str) -> Result<chrono::Duration, ApiError> {
    match period {
        "1h" => Ok(chrono::Duration::hours(1)),
        "6h" => Ok(chrono::Duration::hours(6)),
        "24h" => Ok(chrono::Duration::hours(24)),
        "7d" => Ok(chrono::Duration::days(7)),
        "30d" => Ok(chrono::Duration::days(30)),
        _ => Err((
            StatusCode::BAD_REQUEST,
            Json(
                json!({"error": format!("Invalid period '{period}'. Valid: 1h, 6h, 24h, 7d, 30d")}),
            ),
        )),
    }
}

fn period_to_interval_str(period: &str) -> &'static str {
    match period {
        "1h" => "1 hour",
        "6h" => "6 hours",
        "24h" => "24 hours",
        "7d" => "7 days",
        "30d" => "30 days",
        _ => "24 hours",
    }
}

fn parse_bucket(bucket: &str) -> Result<&'static str, ApiError> {
    match bucket {
        "1m" => Ok("1 minute"),
        "5m" => Ok("5 minutes"),
        "15m" => Ok("15 minutes"),
        "1h" => Ok("1 hour"),
        "6h" => Ok("6 hours"),
        _ => Err((
            StatusCode::BAD_REQUEST,
            Json(
                json!({"error": format!("Invalid bucket '{bucket}'. Valid: 1m, 5m, 15m, 1h, 6h")}),
            ),
        )),
    }
}

fn default_bucket_for_period(period: &str) -> &'static str {
    match period {
        "1h" => "1m",
        "6h" => "5m",
        "24h" => "15m",
        "7d" => "1h",
        "30d" => "6h",
        _ => "15m",
    }
}

// ---------------------------------------------------------------------------
// Endpoint 1: GET /api/admin/scraper/stats
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct StatsParams {
    #[serde(default = "default_period")]
    period: String,
}

fn default_period() -> String {
    "24h".to_string()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScraperStatsResponse {
    period: String,
    total_scrapes: i64,
    successful_scrapes: i64,
    failed_scrapes: i64,
    success_rate: f64,
    avg_duration_ms: f64,
    total_courses_changed: i64,
    total_courses_fetched: i64,
    total_audits_generated: i64,
    pending_jobs: i64,
    locked_jobs: i64,
}

pub async fn scraper_stats(
    _admin: AdminUser,
    State(state): State<AppState>,
    Query(params): Query<StatsParams>,
) -> Result<Json<ScraperStatsResponse>, ApiError> {
    let _duration = parse_period(&params.period)?;
    let interval_str = period_to_interval_str(&params.period);

    let row = sqlx::query(
        "SELECT \
            COUNT(*) AS total_scrapes, \
            COUNT(*) FILTER (WHERE success) AS successful_scrapes, \
            COUNT(*) FILTER (WHERE NOT success) AS failed_scrapes, \
            COALESCE(AVG(duration_ms) FILTER (WHERE success), 0) AS avg_duration_ms, \
            COALESCE(SUM(courses_changed) FILTER (WHERE success), 0) AS total_courses_changed, \
            COALESCE(SUM(courses_fetched) FILTER (WHERE success), 0) AS total_courses_fetched, \
            COALESCE(SUM(audits_generated) FILTER (WHERE success), 0) AS total_audits_generated \
         FROM scrape_job_results \
         WHERE completed_at > NOW() - $1::interval",
    )
    .bind(interval_str)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "Failed to fetch scraper stats");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to fetch scraper stats"})),
        )
    })?;

    let total_scrapes: i64 = row.get("total_scrapes");
    let successful_scrapes: i64 = row.get("successful_scrapes");
    let failed_scrapes: i64 = row.get("failed_scrapes");
    let avg_duration_ms: f64 = row.get("avg_duration_ms");
    let total_courses_changed: i64 = row.get("total_courses_changed");
    let total_courses_fetched: i64 = row.get("total_courses_fetched");
    let total_audits_generated: i64 = row.get("total_audits_generated");

    let queue_row = sqlx::query(
        "SELECT \
            COUNT(*) FILTER (WHERE locked_at IS NULL) AS pending_jobs, \
            COUNT(*) FILTER (WHERE locked_at IS NOT NULL) AS locked_jobs \
         FROM scrape_jobs",
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "Failed to fetch queue stats");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to fetch queue stats"})),
        )
    })?;

    let pending_jobs: i64 = queue_row.get("pending_jobs");
    let locked_jobs: i64 = queue_row.get("locked_jobs");

    let success_rate = if total_scrapes > 0 {
        successful_scrapes as f64 / total_scrapes as f64
    } else {
        0.0
    };

    Ok(Json(ScraperStatsResponse {
        period: params.period,
        total_scrapes,
        successful_scrapes,
        failed_scrapes,
        success_rate,
        avg_duration_ms,
        total_courses_changed,
        total_courses_fetched,
        total_audits_generated,
        pending_jobs,
        locked_jobs,
    }))
}

// ---------------------------------------------------------------------------
// Endpoint 2: GET /api/admin/scraper/timeseries
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct TimeseriesParams {
    #[serde(default = "default_period")]
    period: String,
    bucket: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeseriesResponse {
    period: String,
    bucket: String,
    points: Vec<TimeseriesPoint>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeseriesPoint {
    timestamp: DateTime<Utc>,
    scrape_count: i64,
    success_count: i64,
    error_count: i64,
    courses_changed: i64,
    avg_duration_ms: f64,
}

pub async fn scraper_timeseries(
    _admin: AdminUser,
    State(state): State<AppState>,
    Query(params): Query<TimeseriesParams>,
) -> Result<Json<TimeseriesResponse>, ApiError> {
    let _duration = parse_period(&params.period)?;
    let period_interval = period_to_interval_str(&params.period);

    let bucket_code = match &params.bucket {
        Some(b) => {
            // Validate the bucket
            parse_bucket(b)?;
            b.as_str()
        }
        None => default_bucket_for_period(&params.period),
    };
    let bucket_interval = parse_bucket(bucket_code)?;

    let rows = sqlx::query(
        "SELECT \
            date_bin($1::interval, completed_at, '2020-01-01'::timestamptz) AS bucket_start, \
            COUNT(*)::BIGINT AS scrape_count, \
            COUNT(*) FILTER (WHERE success)::BIGINT AS success_count, \
            COUNT(*) FILTER (WHERE NOT success)::BIGINT AS error_count, \
            COALESCE(SUM(courses_changed) FILTER (WHERE success), 0)::BIGINT AS courses_changed, \
            COALESCE(AVG(duration_ms) FILTER (WHERE success), 0)::FLOAT8 AS avg_duration_ms \
         FROM scrape_job_results \
         WHERE completed_at > NOW() - $2::interval \
         GROUP BY bucket_start \
         ORDER BY bucket_start",
    )
    .bind(bucket_interval)
    .bind(period_interval)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "Failed to fetch scraper timeseries");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to fetch scraper timeseries"})),
        )
    })?;

    let points = rows
        .iter()
        .map(|row| TimeseriesPoint {
            timestamp: row.get("bucket_start"),
            scrape_count: row.get("scrape_count"),
            success_count: row.get("success_count"),
            error_count: row.get("error_count"),
            courses_changed: row.get("courses_changed"),
            avg_duration_ms: row.get("avg_duration_ms"),
        })
        .collect();

    Ok(Json(TimeseriesResponse {
        period: params.period,
        bucket: bucket_code.to_string(),
        points,
    }))
}

// ---------------------------------------------------------------------------
// Endpoint 3: GET /api/admin/scraper/subjects
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubjectsResponse {
    subjects: Vec<SubjectSummary>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubjectSummary {
    subject: String,
    schedule_state: String,
    current_interval_secs: u64,
    time_multiplier: u32,
    last_scraped: DateTime<Utc>,
    avg_change_ratio: f64,
    consecutive_zero_changes: i64,
    recent_runs: i64,
    recent_failures: i64,
}

pub async fn scraper_subjects(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> Result<Json<SubjectsResponse>, ApiError> {
    let raw_stats = scrape_jobs::fetch_subject_stats(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to fetch subject stats");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fetch subject stats"})),
            )
        })?;

    let now = Utc::now();
    let multiplier = adaptive::time_of_day_multiplier(now);

    let subjects = raw_stats
        .into_iter()
        .map(|row| {
            let stats: SubjectStats = row.into();
            let schedule = adaptive::evaluate_subject(&stats, now, false);
            let base_interval = adaptive::compute_base_interval(&stats);

            let schedule_state = match &schedule {
                SubjectSchedule::Eligible(_) => "eligible",
                SubjectSchedule::Cooldown(_) => "cooldown",
                SubjectSchedule::Paused => "paused",
                SubjectSchedule::ReadOnly => "read_only",
            };

            let current_interval_secs = base_interval.as_secs() * multiplier as u64;

            SubjectSummary {
                subject: stats.subject,
                schedule_state: schedule_state.to_string(),
                current_interval_secs,
                time_multiplier: multiplier,
                last_scraped: stats.last_completed,
                avg_change_ratio: stats.avg_change_ratio,
                consecutive_zero_changes: stats.consecutive_zero_changes,
                recent_runs: stats.recent_runs,
                recent_failures: stats.recent_failure_count,
            }
        })
        .collect();

    Ok(Json(SubjectsResponse { subjects }))
}

// ---------------------------------------------------------------------------
// Endpoint 4: GET /api/admin/scraper/subjects/{subject}
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct SubjectDetailParams {
    #[serde(default = "default_detail_limit")]
    limit: i32,
}

fn default_detail_limit() -> i32 {
    50
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubjectDetailResponse {
    subject: String,
    results: Vec<SubjectResultEntry>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubjectResultEntry {
    id: i64,
    completed_at: DateTime<Utc>,
    duration_ms: i32,
    success: bool,
    error_message: Option<String>,
    courses_fetched: Option<i32>,
    courses_changed: Option<i32>,
    courses_unchanged: Option<i32>,
    audits_generated: Option<i32>,
    metrics_generated: Option<i32>,
}

pub async fn scraper_subject_detail(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(subject): Path<String>,
    Query(params): Query<SubjectDetailParams>,
) -> Result<Json<SubjectDetailResponse>, ApiError> {
    let limit = params.limit.clamp(1, 200);

    let rows = sqlx::query(
        "SELECT id, completed_at, duration_ms, success, error_message, \
                courses_fetched, courses_changed, courses_unchanged, \
                audits_generated, metrics_generated \
         FROM scrape_job_results \
         WHERE target_type = 'Subject' AND payload->>'subject' = $1 \
         ORDER BY completed_at DESC \
         LIMIT $2",
    )
    .bind(&subject)
    .bind(limit)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, subject = %subject, "Failed to fetch subject detail");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to fetch subject detail"})),
        )
    })?;

    let results = rows
        .iter()
        .map(|row| SubjectResultEntry {
            id: row.get("id"),
            completed_at: row.get("completed_at"),
            duration_ms: row.get("duration_ms"),
            success: row.get("success"),
            error_message: row.get("error_message"),
            courses_fetched: row.get("courses_fetched"),
            courses_changed: row.get("courses_changed"),
            courses_unchanged: row.get("courses_unchanged"),
            audits_generated: row.get("audits_generated"),
            metrics_generated: row.get("metrics_generated"),
        })
        .collect();

    Ok(Json(SubjectDetailResponse { subject, results }))
}
