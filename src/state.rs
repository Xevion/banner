//! Application state shared across components (bot, web, scheduler).

use crate::banner::BannerApi;
use crate::banner::Course;
use crate::data::models::ReferenceData;
use crate::status::ServiceStatusRegistry;
use anyhow::Result;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// In-memory cache for reference data (code→description lookups).
///
/// Loaded from the `reference_data` table on startup and refreshed periodically.
/// Uses a two-level HashMap so lookups take `&str` without allocating.
pub struct ReferenceCache {
    /// category → (code → description)
    data: HashMap<String, HashMap<String, String>>,
}

impl Default for ReferenceCache {
    fn default() -> Self {
        Self::new()
    }
}

impl ReferenceCache {
    /// Create an empty cache.
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Build cache from a list of reference data entries.
    pub fn from_entries(entries: Vec<ReferenceData>) -> Self {
        let mut data: HashMap<String, HashMap<String, String>> = HashMap::new();
        for e in entries {
            data.entry(e.category)
                .or_default()
                .insert(e.code, e.description);
        }
        Self { data }
    }

    /// Look up a description by category and code. Zero allocations.
    pub fn lookup(&self, category: &str, code: &str) -> Option<&str> {
        self.data
            .get(category)
            .and_then(|codes| codes.get(code))
            .map(|s| s.as_str())
    }

    /// Get all `(code, description)` pairs for a category, sorted by description.
    pub fn entries_for_category(&self, category: &str) -> Vec<(&str, &str)> {
        let Some(codes) = self.data.get(category) else {
            return Vec::new();
        };
        let mut entries: Vec<(&str, &str)> = codes
            .iter()
            .map(|(code, desc)| (code.as_str(), desc.as_str()))
            .collect();
        entries.sort_by(|a, b| a.1.cmp(b.1));
        entries
    }
}

#[derive(Clone)]
pub struct AppState {
    pub banner_api: Arc<BannerApi>,
    pub db_pool: PgPool,
    pub service_statuses: ServiceStatusRegistry,
    pub reference_cache: Arc<RwLock<ReferenceCache>>,
}

impl AppState {
    pub fn new(banner_api: Arc<BannerApi>, db_pool: PgPool) -> Self {
        Self {
            banner_api,
            db_pool,
            service_statuses: ServiceStatusRegistry::new(),
            reference_cache: Arc::new(RwLock::new(ReferenceCache::new())),
        }
    }

    /// Initialize the reference cache from the database.
    pub async fn load_reference_cache(&self) -> Result<()> {
        let entries = crate::data::reference::get_all(&self.db_pool).await?;
        let count = entries.len();
        let cache = ReferenceCache::from_entries(entries);
        *self.reference_cache.write().await = cache;
        tracing::info!(entries = count, "Reference cache loaded");
        Ok(())
    }

    /// Get a course by CRN directly from Banner API
    pub async fn get_course_or_fetch(&self, term: &str, crn: &str) -> Result<Course> {
        self.banner_api
            .get_course_by_crn(term, crn)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Course not found for CRN {crn}"))
    }

    /// Get the total number of courses in the database
    pub async fn get_course_count(&self) -> Result<i64> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM courses")
            .fetch_one(&self.db_pool)
            .await?;
        Ok(count.0)
    }
}
