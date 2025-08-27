//! Application state shared across components (bot, web, scheduler).

use crate::banner::BannerApi;
use redis::Client;

#[derive(Clone)]
pub struct AppState {
    pub banner_api: std::sync::Arc<BannerApi>,
    pub redis_client: std::sync::Arc<Client>,
}

impl AppState {
    pub fn new(
        banner_api: BannerApi,
        redis_url: &str,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let redis_client = Client::open(redis_url)?;

        Ok(Self {
            banner_api: std::sync::Arc::new(banner_api),
            redis_client: std::sync::Arc::new(redis_client),
        })
    }
}
