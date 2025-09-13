//! Web API endpoints for Banner bot monitoring and metrics.

use axum::{
    Router,
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, HeaderValue, StatusCode, Uri},
    response::{Html, IntoResponse, Json, Response},
    routing::get,
};
use http::header;
use serde_json::{Value, json};
use std::{sync::Arc, time::Duration};
use tower_http::{
    classify::ServerErrorsFailureClass,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{Span, debug, info, warn};

use crate::web::assets::{WebAssets, get_asset_metadata_cached};

use crate::banner::BannerApi;

/// Set appropriate caching headers based on asset type
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

    let mut router = Router::new().nest("/api", api_router);

    if cfg!(debug_assertions) {
        router = router.layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
    } else {
        router = router.fallback(fallback);
    }

    router.layer(
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
                    let (latency_str, status, code) = (
                        format!("{latency:.2?}"),
                        response.status().as_u16(),
                        format!(
                            "{} {}",
                            response.status().as_u16(),
                            response.status().canonical_reason().unwrap_or("??")
                        ),
                    );

                    // Log in warn if latency is above threshold, otherwise debug
                    if latency > latency_threshold {
                        warn!(
                            latency = latency_str,
                            status = status,
                            code = code,
                            "Response"
                        );
                    } else {
                        debug!(
                            latency = latency_str,
                            status = status,
                            code = code,
                            "Response"
                        );
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
    )
}

/// Handler that extracts request information for caching
async fn fallback(request: Request) -> Response {
    let uri = request.uri().clone();
    let headers = request.headers().clone();
    handle_spa_fallback_with_headers(uri, headers).await
}

/// Handles SPA routing by serving index.html for non-API, non-asset requests
/// This version includes HTTP caching headers and ETag support
async fn handle_spa_fallback_with_headers(uri: Uri, request_headers: HeaderMap) -> Response {
    let path = uri.path().trim_start_matches('/');

    if let Some(content) = WebAssets::get(path) {
        // Get asset metadata (MIME type and hash) with caching
        let metadata = get_asset_metadata_cached(path, &content.data);

        // Check if client has a matching ETag (conditional request)
        if let Some(etag) = request_headers.get(header::IF_NONE_MATCH)
            && metadata.etag_matches(etag.to_str().unwrap())
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
                && metadata.etag_matches(etag.to_str().unwrap())
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
        "git": {
            "commit": env!("GIT_COMMIT_HASH"),
            "short": env!("GIT_COMMIT_SHORT")
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
