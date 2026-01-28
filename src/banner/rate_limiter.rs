//! Rate limiting for Banner API requests to prevent overwhelming the server.

use crate::config::RateLimitingConfig;
use governor::{
    Quota, RateLimiter,
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

/// Different types of Banner API requests with different rate limits
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RequestType {
    /// Session creation and management (very conservative)
    Session,
    /// Course search requests (moderate)
    Search,
    /// Term and metadata requests (moderate)
    Metadata,
    /// Data form resets (low priority)
    Reset,
}

/// A rate limiter that manages different request types with different limits
pub struct BannerRateLimiter {
    session_limiter: RateLimiter<NotKeyed, InMemoryState, DefaultClock>,
    search_limiter: RateLimiter<NotKeyed, InMemoryState, DefaultClock>,
    metadata_limiter: RateLimiter<NotKeyed, InMemoryState, DefaultClock>,
    reset_limiter: RateLimiter<NotKeyed, InMemoryState, DefaultClock>,
}

impl BannerRateLimiter {
    /// Creates a new rate limiter with the given configuration
    pub fn new(config: RateLimitingConfig) -> Self {
        let session_quota = Quota::with_period(Duration::from_secs(60) / config.session_rpm)
            .unwrap()
            .allow_burst(NonZeroU32::new(config.burst_allowance).unwrap());

        let search_quota = Quota::with_period(Duration::from_secs(60) / config.search_rpm)
            .unwrap()
            .allow_burst(NonZeroU32::new(config.burst_allowance).unwrap());

        let metadata_quota = Quota::with_period(Duration::from_secs(60) / config.metadata_rpm)
            .unwrap()
            .allow_burst(NonZeroU32::new(config.burst_allowance).unwrap());

        let reset_quota = Quota::with_period(Duration::from_secs(60) / config.reset_rpm)
            .unwrap()
            .allow_burst(NonZeroU32::new(config.burst_allowance).unwrap());

        Self {
            session_limiter: RateLimiter::direct(session_quota),
            search_limiter: RateLimiter::direct(search_quota),
            metadata_limiter: RateLimiter::direct(metadata_quota),
            reset_limiter: RateLimiter::direct(reset_quota),
        }
    }

    /// Waits for permission to make a request of the given type
    pub async fn wait_for_permission(&self, request_type: RequestType) {
        let limiter = match request_type {
            RequestType::Session => &self.session_limiter,
            RequestType::Search => &self.search_limiter,
            RequestType::Metadata => &self.metadata_limiter,
            RequestType::Reset => &self.reset_limiter,
        };

        // Wait until we can make the request (logging handled by middleware)
        limiter.until_ready().await;
    }
}

impl Default for BannerRateLimiter {
    fn default() -> Self {
        Self::new(RateLimitingConfig::default())
    }
}

/// A shared rate limiter instance
pub type SharedRateLimiter = Arc<BannerRateLimiter>;

/// Creates a new shared rate limiter with custom configuration
pub fn create_shared_rate_limiter(config: Option<RateLimitingConfig>) -> SharedRateLimiter {
    Arc::new(BannerRateLimiter::new(config.unwrap_or_default()))
}
