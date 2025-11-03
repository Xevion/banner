//! HTTP middleware that enforces rate limiting for Banner API requests.

use crate::banner::rate_limiter::{RequestType, SharedRateLimiter};
use http::Extensions;
use reqwest::{Request, Response};
use reqwest_middleware::{Middleware, Next};
use tracing::debug;
use url::Url;

/// Middleware that enforces rate limiting based on request URL patterns
pub struct RateLimitMiddleware {
    rate_limiter: SharedRateLimiter,
}

impl RateLimitMiddleware {
    /// Creates a new rate limiting middleware
    pub fn new(rate_limiter: SharedRateLimiter) -> Self {
        Self { rate_limiter }
    }

    /// Returns a human-readable description of the rate limit for a request type
    fn get_rate_limit_description(request_type: RequestType) -> &'static str {
        match request_type {
            RequestType::Session => "6 rpm (~10s interval)",
            RequestType::Search => "30 rpm (~2s interval)",
            RequestType::Metadata => "20 rpm (~3s interval)",
            RequestType::Reset => "10 rpm (~6s interval)",
        }
    }

    /// Determines the request type based on the URL path
    fn get_request_type(url: &Url) -> RequestType {
        let path = url.path();

        if path.contains("/registration")
            || path.contains("/selfServiceMenu")
            || path.contains("/term/termSelection")
        {
            RequestType::Session
        } else if path.contains("/searchResults") || path.contains("/classSearch") {
            RequestType::Search
        } else if path.contains("/getTerms")
            || path.contains("/getSubjects")
            || path.contains("/getCampuses")
        {
            RequestType::Metadata
        } else if path.contains("/resetDataForm") {
            RequestType::Reset
        } else {
            // Default to search for unknown endpoints
            RequestType::Search
        }
    }
}

#[async_trait::async_trait]
impl Middleware for RateLimitMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> std::result::Result<Response, reqwest_middleware::Error> {
        let request_type = Self::get_request_type(req.url());

        let start = std::time::Instant::now();
        self.rate_limiter.wait_for_permission(request_type).await;
        let wait_duration = start.elapsed();

        // Only log if rate limiting caused significant delay (>= 500ms)
        if wait_duration.as_millis() >= 500 {
            let limit_desc = Self::get_rate_limit_description(request_type);
            debug!(
                request_type = ?request_type,
                wait_ms = wait_duration.as_millis(),
                rate_limit = limit_desc,
                "Rate limit caused delay"
            );
        }

        // Make the actual request
        next.run(req, extensions).await
    }
}
