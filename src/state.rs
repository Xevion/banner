//! Application state shared across components (bot, web, scheduler).

use crate::banner::BannerApi;
use crate::banner::Course;
use anyhow::Result;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub banner_api: Arc<BannerApi>,
    pub db_pool: PgPool,
}

impl AppState {
    pub fn new(banner_api: Arc<BannerApi>, db_pool: PgPool) -> Self {
        Self {
            banner_api,
            db_pool,
        }
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
