//! Application state shared across components (bot, web, scheduler).

use crate::banner::BannerApi;
use crate::banner::Course;
use anyhow::Result;
use redis::AsyncCommands;
use redis::Client;
use serde_json;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct AppState {
    pub banner_api: Arc<BannerApi>,
    pub redis: Arc<Client>,
}

impl AppState {
    pub fn new(
        banner_api: Arc<BannerApi>,
        redis_url: &str,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let redis_client = Client::open(redis_url)?;

        Ok(Self {
            banner_api,
            redis: Arc::new(redis_client),
        })
    }

    /// Get a course by CRN with Redis cache fallback to Banner API
    pub async fn get_course_or_fetch(&self, term: &str, crn: &str) -> Result<Course> {
        let mut conn = self.redis.get_multiplexed_async_connection().await?;

        let key = format!("class:{}", crn);
        if let Some(serialized) = conn.get::<_, Option<String>>(&key).await? {
            let course: Course = serde_json::from_str(&serialized)?;
            return Ok(course);
        }

        // Fallback: fetch from Banner API
        if let Some(course) = self.banner_api.get_course_by_crn(term, crn).await? {
            let serialized = serde_json::to_string(&course)?;
            let _: () = conn.set(&key, serialized).await?;
            return Ok(course);
        }

        Err(anyhow::anyhow!("Course not found for CRN {}", crn))
    }
}
