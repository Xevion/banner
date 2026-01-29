//! Admin API handlers.
//!
//! All endpoints require the `AdminUser` extractor, returning 401/403 as needed.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::data::models::User;
use crate::state::AppState;
use crate::web::extractors::AdminUser;

/// `GET /api/admin/status` — Enhanced system status for admins.
pub async fn admin_status(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let (user_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to count users");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "failed to count users"})),
            )
        })?;

    let (session_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM user_sessions WHERE expires_at > now()")
            .fetch_one(&state.db_pool)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "failed to count sessions");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "failed to count sessions"})),
                )
            })?;

    let course_count = state.get_course_count().await.map_err(|e| {
        tracing::error!(error = %e, "failed to count courses");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "failed to count courses"})),
        )
    })?;

    let (scrape_job_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM scrape_jobs")
        .fetch_one(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to count scrape jobs");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "failed to count scrape jobs"})),
            )
        })?;

    let services: Vec<Value> = state
        .service_statuses
        .all()
        .into_iter()
        .map(|(name, status)| {
            json!({
                "name": name,
                "status": status,
            })
        })
        .collect();

    Ok(Json(json!({
        "userCount": user_count,
        "sessionCount": session_count,
        "courseCount": course_count,
        "scrapeJobCount": scrape_job_count,
        "services": services,
    })))
}

/// `GET /api/admin/users` — List all users.
pub async fn list_users(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<User>>, (StatusCode, Json<Value>)> {
    let users = crate::data::users::list_users(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to list users");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "failed to list users"})),
            )
        })?;

    Ok(Json(users))
}

#[derive(Deserialize)]
pub struct SetAdminBody {
    is_admin: bool,
}

/// `PUT /api/admin/users/{discord_id}/admin` — Set admin status for a user.
pub async fn set_user_admin(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    Path(discord_id): Path<i64>,
    Json(body): Json<SetAdminBody>,
) -> Result<Json<User>, (StatusCode, Json<Value>)> {
    let user = crate::data::users::set_admin(&state.db_pool, discord_id, body.is_admin)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to set admin status");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "failed to set admin status"})),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "user not found"})),
            )
        })?;

    state.session_cache.evict_user(discord_id);

    Ok(Json(user))
}

/// `GET /api/admin/scrape-jobs` — List scrape jobs.
pub async fn list_scrape_jobs(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let rows = sqlx::query_as::<_, crate::data::models::ScrapeJob>(
        "SELECT * FROM scrape_jobs ORDER BY priority DESC, execute_at ASC LIMIT 100",
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to list scrape jobs");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "failed to list scrape jobs"})),
        )
    })?;

    let jobs: Vec<Value> = rows
        .iter()
        .map(|j| {
            json!({
                "id": j.id,
                "targetType": format!("{:?}", j.target_type),
                "targetPayload": j.target_payload,
                "priority": format!("{:?}", j.priority),
                "executeAt": j.execute_at.to_rfc3339(),
                "createdAt": j.created_at.to_rfc3339(),
                "lockedAt": j.locked_at.map(|t| t.to_rfc3339()),
                "retryCount": j.retry_count,
                "maxRetries": j.max_retries,
            })
        })
        .collect();

    Ok(Json(json!({ "jobs": jobs })))
}

/// `GET /api/admin/audit-log` — List recent audit entries.
pub async fn list_audit_log(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let rows = sqlx::query_as::<_, crate::data::models::CourseAudit>(
        "SELECT * FROM course_audits ORDER BY timestamp DESC LIMIT 200",
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to list audit log");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "failed to list audit log"})),
        )
    })?;

    let entries: Vec<Value> = rows
        .iter()
        .map(|a| {
            json!({
                "id": a.id,
                "courseId": a.course_id,
                "timestamp": a.timestamp.to_rfc3339(),
                "fieldChanged": a.field_changed,
                "oldValue": a.old_value,
                "newValue": a.new_value,
            })
        })
        .collect();

    Ok(Json(json!({ "entries": entries })))
}
