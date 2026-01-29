//! Embedded assets for the web frontend
//!
//! This module handles serving static assets that are embedded into the binary
//! at compile time using rust-embed.

use dashmap::DashMap;
use rapidhash::v3::rapidhash_v3;
use rust_embed::RustEmbed;
use std::fmt;
use std::sync::LazyLock;

/// Embedded web assets from the dist directory
#[derive(RustEmbed)]
#[folder = "web/dist/"]
#[include = "*"]
#[exclude = "*.map"]
pub struct WebAssets;

/// RapidHash hash type for asset content (u64 native output size)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssetHash(u64);

impl AssetHash {
    /// Create a new AssetHash from u64 value
    pub fn new(hash: u64) -> Self {
        Self(hash)
    }

    /// Get the hash as a hex string
    pub fn to_hex(&self) -> String {
        format!("{:016x}", self.0)
    }

    /// Get the hash as a quoted hex string
    pub fn quoted(&self) -> String {
        format!("\"{}\"", self.to_hex())
    }
}

impl fmt::Display for AssetHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Metadata for an asset including MIME type and RapidHash hash
#[derive(Debug, Clone)]
pub struct AssetMetadata {
    pub mime_type: Option<String>,
    pub hash: AssetHash,
}

impl AssetMetadata {
    /// Check if the etag matches the asset hash
    pub fn etag_matches(&self, etag: &str) -> bool {
        // Remove quotes if present (ETags are typically quoted)
        let etag = etag.trim_matches('"');

        // ETags generated from u64 hex should be 16 characters
        etag.len() == 16
            && u64::from_str_radix(etag, 16)
                .map(|parsed| parsed == self.hash.0)
                .unwrap_or(false)
    }
}

/// Global cache for asset metadata to avoid repeated calculations
static ASSET_CACHE: LazyLock<DashMap<String, AssetMetadata>> = LazyLock::new(DashMap::new);

/// Get cached asset metadata for a file path, caching on-demand
/// Returns AssetMetadata containing MIME type and RapidHash hash
pub fn get_asset_metadata_cached(path: &str, content: &[u8]) -> AssetMetadata {
    // Check cache first
    if let Some(cached) = ASSET_CACHE.get(path) {
        return cached.value().clone();
    }

    // Calculate MIME type
    let mime_type = mime_guess::from_path(path)
        .first()
        .map(|mime| mime.to_string());

    // Calculate RapidHash hash (using u64 native output size)
    let hash_value = rapidhash_v3(content);
    let hash = AssetHash::new(hash_value);

    let metadata = AssetMetadata { mime_type, hash };

    // Only cache if we haven't exceeded the limit
    if ASSET_CACHE.len() < 1000 {
        ASSET_CACHE.insert(path.to_string(), metadata.clone());
    }

    metadata
}
