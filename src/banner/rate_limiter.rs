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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_default_config() {
        let _limiter = BannerRateLimiter::new(RateLimitingConfig::default());
    }

    #[test]
    fn test_new_with_custom_config() {
        let config = RateLimitingConfig {
            session_rpm: 10,
            search_rpm: 30,
            metadata_rpm: 20,
            reset_rpm: 15,
            burst_allowance: 5,
        };
        let _limiter = BannerRateLimiter::new(config);
    }

    #[test]
    fn test_new_with_minimum_valid_values() {
        let config = RateLimitingConfig {
            session_rpm: 1,
            search_rpm: 1,
            metadata_rpm: 1,
            reset_rpm: 1,
            burst_allowance: 1,
        };
        let _limiter = BannerRateLimiter::new(config);
    }

    #[test]
    fn test_new_with_high_rpm_values() {
        let config = RateLimitingConfig {
            session_rpm: 10000,
            search_rpm: 10000,
            metadata_rpm: 10000,
            reset_rpm: 10000,
            burst_allowance: 1,
        };
        let _limiter = BannerRateLimiter::new(config);
    }

    #[test]
    fn test_default_impl() {
        let _limiter = BannerRateLimiter::default();
    }

    #[test]
    #[should_panic]
    fn test_new_panics_on_zero_session_rpm() {
        let config = RateLimitingConfig {
            session_rpm: 0,
            ..RateLimitingConfig::default()
        };
        let _limiter = BannerRateLimiter::new(config);
    }

    #[test]
    #[should_panic]
    fn test_new_panics_on_zero_search_rpm() {
        let config = RateLimitingConfig {
            search_rpm: 0,
            ..RateLimitingConfig::default()
        };
        let _limiter = BannerRateLimiter::new(config);
    }

    #[test]
    #[should_panic]
    fn test_new_panics_on_zero_metadata_rpm() {
        let config = RateLimitingConfig {
            metadata_rpm: 0,
            ..RateLimitingConfig::default()
        };
        let _limiter = BannerRateLimiter::new(config);
    }

    #[test]
    #[should_panic]
    fn test_new_panics_on_zero_reset_rpm() {
        let config = RateLimitingConfig {
            reset_rpm: 0,
            ..RateLimitingConfig::default()
        };
        let _limiter = BannerRateLimiter::new(config);
    }

    #[test]
    #[should_panic]
    fn test_new_panics_on_zero_burst_allowance() {
        let config = RateLimitingConfig {
            burst_allowance: 0,
            ..RateLimitingConfig::default()
        };
        let _limiter = BannerRateLimiter::new(config);
    }

    #[tokio::test]
    async fn test_wait_for_permission_completes() {
        let limiter = BannerRateLimiter::default();
        let timeout_duration = std::time::Duration::from_secs(1);

        for request_type in [
            RequestType::Session,
            RequestType::Search,
            RequestType::Metadata,
            RequestType::Reset,
        ] {
            let result =
                tokio::time::timeout(timeout_duration, limiter.wait_for_permission(request_type))
                    .await;
            assert!(
                result.is_ok(),
                "wait_for_permission timed out for {:?}",
                request_type
            );
        }
    }
}
