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
use tracing::debug;

/// Embedded web assets from the dist directory
#[derive(RustEmbed)]
#[folder = "web/dist/"]
#[include = "*"]
#[exclude = "*.map"]
pub struct WebAssets;

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
