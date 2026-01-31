//! Standardized API error responses.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use ts_rs::TS;

/// Standardized error response for all API endpoints.
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ApiError {
    /// Machine-readable error code (e.g., "NOT_FOUND", "INVALID_TERM")
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Optional additional details (validation errors, field info, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ApiError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new("NOT_FOUND", message)
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new("BAD_REQUEST", message)
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new("INTERNAL_ERROR", message)
    }

    pub fn invalid_term(term: impl std::fmt::Display) -> Self {
        Self::new("INVALID_TERM", format!("Invalid term: {}", term))
    }

    fn status_code(&self) -> StatusCode {
        match self.code.as_str() {
            "NOT_FOUND" => StatusCode::NOT_FOUND,
            "BAD_REQUEST" | "INVALID_TERM" | "INVALID_RANGE" => StatusCode::BAD_REQUEST,
            "UNAUTHORIZED" => StatusCode::UNAUTHORIZED,
            "FORBIDDEN" => StatusCode::FORBIDDEN,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        (status, Json(self)).into_response()
    }
}

/// Convert `(StatusCode, String)` tuple errors to ApiError
impl From<(StatusCode, String)> for ApiError {
    fn from((status, message): (StatusCode, String)) -> Self {
        let code = match status {
            StatusCode::NOT_FOUND => "NOT_FOUND",
            StatusCode::BAD_REQUEST => "BAD_REQUEST",
            StatusCode::UNAUTHORIZED => "UNAUTHORIZED",
            StatusCode::FORBIDDEN => "FORBIDDEN",
            _ => "INTERNAL_ERROR",
        };
        Self::new(code, message)
    }
}

/// Helper for converting database errors to ApiError
pub fn db_error(context: &str, error: anyhow::Error) -> ApiError {
    tracing::error!(error = %error, context = context, "Database error");
    ApiError::internal_error(format!("{} failed", context))
}
