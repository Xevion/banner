//! Web API endpoints for Banner bot monitoring and metrics.

use axum::{Router, extract::State, response::Json, routing::get};
use serde_json::{Value, json};
use std::sync::Arc;
use tracing::{debug, info};

/// Shared application state for web server
#[derive(Clone)]
pub struct BannerState {
    pub api: Arc<crate::banner::BannerApi>,
    pub scraper: Arc<crate::banner::scraper::CourseScraper>,
}

/// Creates the web server router
pub fn create_banner_router(state: BannerState) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/status", get(status))
        .route("/metrics", get(metrics))
        .with_state(state)
}

/// Root endpoint - shows API info
async fn root() -> Json<Value> {
    debug!("root endpoint accessed");
    Json(json!({
        "message": "Banner Discord Bot API",
        "version": "0.1.0",
        "endpoints": {
            "health": "/health",
            "status": "/status",
            "metrics": "/metrics"
        }
    }))
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
        "redis": {
            "status": "connected",
            "connected_clients": "TODO: implement client counting",
            "used_memory": "TODO: implement memory tracking"
        },
        "cache": {
            "courses": {
                "count": "TODO: implement course counting"
            },
            "subjects": {
                "count": "TODO: implement subject counting"
            }
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
