//! Rate limiting for Banner API requests to prevent overwhelming the server.

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

/// Rate limiter configuration for different request types
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Requests per minute for session operations
    pub session_rpm: u32,
    /// Requests per minute for search operations
    pub search_rpm: u32,
    /// Requests per minute for metadata operations
    pub metadata_rpm: u32,
    /// Requests per minute for reset operations
    pub reset_rpm: u32,
    /// Burst allowance (extra requests allowed in short bursts)
    pub burst_allowance: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            // Very conservative for session creation
            session_rpm: 6, // 1 every 10 seconds
            // Moderate for search operations
            search_rpm: 30, // 1 every 2 seconds
            // Moderate for metadata
            metadata_rpm: 20, // 1 every 3 seconds
            // Low for resets
            reset_rpm: 10, // 1 every 6 seconds
            // Allow small bursts
            burst_allowance: 3,
        }
    }
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
    pub fn new(config: RateLimitConfig) -> Self {
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
        Self::new(RateLimitConfig::default())
    }
}

/// A shared rate limiter instance
pub type SharedRateLimiter = Arc<BannerRateLimiter>;

/// Creates a new shared rate limiter with custom configuration
pub fn create_shared_rate_limiter(config: Option<RateLimitConfig>) -> SharedRateLimiter {
    Arc::new(BannerRateLimiter::new(config.unwrap_or_default()))
}

/// Conversion from config module's RateLimitingConfig to this module's RateLimitConfig
impl From<crate::config::RateLimitingConfig> for RateLimitConfig {
    fn from(config: crate::config::RateLimitingConfig) -> Self {
        Self {
            session_rpm: config.session_rpm,
            search_rpm: config.search_rpm,
            metadata_rpm: config.metadata_rpm,
            reset_rpm: config.reset_rpm,
            burst_allowance: config.burst_allowance,
        }
    }
}
