//! Admin API handlers for term management.
//!
//! All endpoints require the `AdminUser` extractor, returning 401/403 as needed.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde::Serialize;
use serde_json::{Value, json};
use ts_rs::TS;

use crate::data::terms::{self, DbTerm, SyncResult};
use crate::state::AppState;
use crate::web::extractors::AdminUser;

type ApiError = (StatusCode, Json<Value>);

fn db_error(context: &str, e: anyhow::Error) -> ApiError {
    tracing::error!(error = %e, "{context}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error": context})),
    )
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// Response for `GET /api/admin/terms`.
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TermsListResponse {
    pub terms: Vec<DbTerm>,
}

/// Response for `POST /api/admin/terms/:code/enable` and `disable`.
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TermUpdateResponse {
    pub success: bool,
    pub term: Option<DbTerm>,
}

/// Response for `POST /api/admin/terms/sync`.
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TermSyncResponse {
    pub inserted: usize,
    pub updated: usize,
}

impl From<SyncResult> for TermSyncResponse {
    fn from(result: SyncResult) -> Self {
        Self {
            inserted: result.inserted,
            updated: result.updated,
        }
    }
}

// ---------------------------------------------------------------------------
// 1. GET /api/admin/terms — List all terms
// ---------------------------------------------------------------------------

/// `GET /api/admin/terms` — List all terms with their scraping status.
pub async fn list_terms(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> Result<Json<TermsListResponse>, ApiError> {
    let terms = terms::get_all_terms(&state.db_pool)
        .await
        .map_err(|e| db_error("Failed to fetch terms", e))?;

    Ok(Json(TermsListResponse { terms }))
}

// ---------------------------------------------------------------------------
// 2. POST /api/admin/terms/:code/enable — Enable scraping
// ---------------------------------------------------------------------------

/// `POST /api/admin/terms/:code/enable` — Enable scraping for a term.
pub async fn enable_term(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Json<TermUpdateResponse>, ApiError> {
    let found = terms::enable_scraping(&state.db_pool, &code)
        .await
        .map_err(|e| db_error("Failed to enable scraping", e))?;

    if !found {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Term not found"})),
        ));
    }

    let term = terms::get_term_by_code(&state.db_pool, &code)
        .await
        .map_err(|e| db_error("Failed to fetch updated term", e))?;

    tracing::info!(term_code = %code, "Term scraping enabled by admin");

    Ok(Json(TermUpdateResponse {
        success: true,
        term,
    }))
}

// ---------------------------------------------------------------------------
// 3. POST /api/admin/terms/:code/disable — Disable scraping
// ---------------------------------------------------------------------------

/// `POST /api/admin/terms/:code/disable` — Disable scraping for a term.
pub async fn disable_term(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Json<TermUpdateResponse>, ApiError> {
    let found = terms::disable_scraping(&state.db_pool, &code)
        .await
        .map_err(|e| db_error("Failed to disable scraping", e))?;

    if !found {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Term not found"})),
        ));
    }

    let term = terms::get_term_by_code(&state.db_pool, &code)
        .await
        .map_err(|e| db_error("Failed to fetch updated term", e))?;

    tracing::info!(term_code = %code, "Term scraping disabled by admin");

    Ok(Json(TermUpdateResponse {
        success: true,
        term,
    }))
}

// ---------------------------------------------------------------------------
// 4. POST /api/admin/terms/sync — Sync terms from Banner API
// ---------------------------------------------------------------------------

/// `POST /api/admin/terms/sync` — Manually sync terms from the Banner API.
pub async fn sync_terms(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> Result<Json<TermSyncResponse>, ApiError> {
    let banner_terms = state
        .banner_api
        .sessions
        .get_terms("", 1, 500)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to fetch terms from Banner API");
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({"error": "Failed to fetch terms from Banner API"})),
            )
        })?;

    let result = terms::sync_terms_from_banner(&state.db_pool, banner_terms)
        .await
        .map_err(|e| db_error("Failed to sync terms to database", e))?;

    tracing::info!(
        inserted = result.inserted,
        updated = result.updated,
        "Terms synced from Banner API by admin"
    );

    Ok(Json(result.into()))
}
