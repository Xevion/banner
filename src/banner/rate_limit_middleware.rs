//! HTTP middleware that enforces rate limiting for Banner API requests.

use crate::banner::rate_limiter::{RequestType, SharedRateLimiter};
use http::Extensions;
use reqwest::{Request, Response};
use reqwest_middleware::{Middleware, Next};
use tracing::{debug, warn};
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

        debug!(
            url = %req.url(),
            request_type = ?request_type,
            "Rate limiting request"
        );

        // Wait for permission to make the request
        self.rate_limiter.wait_for_permission(request_type).await;

        debug!(
            url = %req.url(),
            request_type = ?request_type,
            "Rate limit permission granted, making request"
        );

        // Make the actual request
        let response_result = next.run(req, extensions).await;

        match response_result {
            Ok(response) => {
                if response.status().is_success() {
                    debug!(
                        url = %response.url(),
                        status = response.status().as_u16(),
                        "Request completed successfully"
                    );
                } else {
                    warn!(
                        url = %response.url(),
                        status = response.status().as_u16(),
                        "Request completed with error status"
                    );
                }
                Ok(response)
            }
            Err(error) => {
                warn!(
                    url = %error.url().unwrap_or(&Url::parse("unknown").unwrap()),
                    error = ?error,
                    "Request failed"
                );
                Err(error)
            }
        }
    }
}
