//! Embedded assets for the web frontend
//!
//! This module handles serving static assets that are embedded into the binary
//! at compile time using rust-embed.

use axum::{
    extract::Path,
    http::{StatusCode, header},
    response::{Html, IntoResponse, Response},
};
use rust_embed::RustEmbed;

/// Embedded web assets from the dist directory
#[derive(RustEmbed)]
#[folder = "web/dist/"]
#[include = "*"]
#[exclude = "*.map"]
pub struct WebAssets;

/// Serve embedded static assets
pub async fn serve_asset(Path(path): Path<String>) -> Response {
    let path = path.trim_start_matches('/');

    match WebAssets::get(path) {
        Some(content) => {
            let mime_type = mime_guess::from_path(path).first_or_text_plain();
            let data = content.data.to_vec();
            ([(header::CONTENT_TYPE, mime_type.as_ref())], data).into_response()
        }
        None => (StatusCode::NOT_FOUND, "Asset not found").into_response(),
    }
}

/// Serve the main SPA index.html for client-side routing
pub async fn serve_spa_index() -> Response {
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

const ASSET_EXTENSIONS: &[&str] = &[
    "js", "css", "png", "jpg", "jpeg", "gif", "svg", "ico", "woff", "woff2", "ttf", "eot",
];

/// Check if a path should be served as a static asset
pub fn is_asset_path(path: &str) -> bool {
    if !path.starts_with("/assets/") {
        return path.eq("index.html");
    }

    match path.split_once('.') {
        Some((_, extension)) => ASSET_EXTENSIONS.contains(&extension),
        None => false,
    }
}
