//! Web API endpoints for Banner bot monitoring and metrics.

use axum::{
    Router,
    extract::State,
    http::{StatusCode, Uri},
    response::{Html, IntoResponse, Json, Response},
    routing::get,
};
use http::header;
use serde_json::{Value, json};
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::info;

use crate::web::assets::{WebAssets, get_mime_type_cached};

use crate::banner::BannerApi;

/// Shared application state for web server
#[derive(Clone)]
pub struct BannerState {
    pub api: Arc<BannerApi>,
}

/// Creates the web server router
pub fn create_router(state: BannerState) -> Router {
    let api_router = Router::new()
        .route("/health", get(health))
        .route("/status", get(status))
        .route("/metrics", get(metrics))
        .with_state(state);

    if cfg!(debug_assertions) {
        // Development mode: API routes only, frontend served by Vite dev server
        Router::new()
            .route("/", get(root))
            .nest("/api", api_router)
            .layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any),
            )
            .layer(TraceLayer::new_for_http())
    } else {
        // Production mode: serve embedded assets and handle SPA routing
        Router::new()
            .route("/", get(root))
            .nest("/api", api_router)
            .fallback(handle_spa_fallback)
            .layer(TraceLayer::new_for_http())
    }
}

async fn root() -> Response {
    if cfg!(debug_assertions) {
        // Development mode: return API info
        Json(json!({
            "message": "Banner Discord Bot API",
            "version": "0.2.1",
            "mode": "development",
            "frontend": "http://localhost:3000",
            "endpoints": {
                "health": "/api/health",
                "status": "/api/status",
                "metrics": "/api/metrics"
            }
        }))
        .into_response()
    } else {
        // Production mode: serve the SPA index.html
        handle_spa_fallback(Uri::from_static("/")).await
    }
}

/// Handles SPA routing by serving index.html for non-API, non-asset requests
async fn handle_spa_fallback(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');

    if let Some(content) = WebAssets::get(path) {
        let data = content.data.to_vec();

        // Use cached MIME type, only set Content-Type if we have a valid MIME type
        let mime_type = get_mime_type_cached(path);
        return (
            [(
                header::CONTENT_TYPE,
                // For unknown types, set to application/octet-stream
                mime_type.unwrap_or("application/octet-stream".to_string()),
            )],
            data,
        )
            .into_response();
    } else {
        // Any assets that are not found should be treated as a 404, not falling back to the SPA index.html
        if path.starts_with("assets/") {
            return (StatusCode::NOT_FOUND, "Asset not found").into_response();
        }
    }

    // Fall back to the SPA index.html
    match WebAssets::get("index.html") {
        Some(content) => {
            let data = content.data.to_vec();
            let html_content = String::from_utf8_lossy(&data).to_string();
            Html(html_content).into_response()
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
    info!("health check requested");
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Status endpoint showing bot and system status
async fn status(State(_state): State<BannerState>) -> Json<Value> {
    // For now, return basic status without accessing private fields
    Json(json!({
        "status": "operational",
        "bot": {
            "status": "running",
            "uptime": "TODO: implement uptime tracking"
        },
        "cache": {
            "status": "connected",
            "courses": "TODO: implement course counting",
            "subjects": "TODO: implement subject counting"
        },
        "banner_api": {
            "status": "connected"
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Metrics endpoint for monitoring
async fn metrics(State(_state): State<BannerState>) -> Json<Value> {
    // For now, return basic metrics structure
    Json(json!({
        "banner_api": {
            "status": "connected"
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
