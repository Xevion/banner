//! Embedded assets for the web frontend
//!
//! This module handles serving static assets that are embedded into the binary
//! at compile time using rust-embed.

use dashmap::DashMap;
use once_cell::sync::Lazy;
use rust_embed::RustEmbed;

/// Embedded web assets from the dist directory
#[derive(RustEmbed)]
#[folder = "web/dist/"]
#[include = "*"]
#[exclude = "*.map"]
pub struct WebAssets;

/// Global cache for MIME types to avoid repeated mime_guess lookups
static MIME_CACHE: Lazy<DashMap<String, Option<String>>> = Lazy::new(DashMap::new);

/// Get cached MIME type for a file path, caching on-demand
/// Returns None if the MIME type is text/plain or if no MIME type could be determined
pub fn get_mime_type_cached(path: &str) -> Option<String> {
    // Check cache first
    if let Some(cached) = MIME_CACHE.get(path) {
        return cached.value().as_ref().cloned();
    }

    // Perform MIME guess and cache the result
    let result = mime_guess::from_path(path)
        .first()
        .map(|mime| mime.to_string());

    // Cache the result
    MIME_CACHE.insert(path.to_string(), result.clone());

    result
}
