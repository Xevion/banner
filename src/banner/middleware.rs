//! HTTP middleware for the Banner API client.

use http::Extensions;
use reqwest::{Request, Response};
use reqwest_middleware::{Middleware, Next};
use tracing::{debug, trace, warn};

pub struct LoggingMiddleware;

/// Threshold for logging slow requests at DEBUG level (in milliseconds)
const SLOW_REQUEST_THRESHOLD_MS: u128 = 1000;

#[async_trait::async_trait]
impl Middleware for LoggingMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> std::result::Result<Response, reqwest_middleware::Error> {
        let method = req.method().to_string();
        // Use the full URL (including query parameters) for logging
        let url = req.url().to_string();

        let start = std::time::Instant::now();
        let response_result = next.run(req, extensions).await;
        let duration = start.elapsed();

        match response_result {
            Ok(response) => {
                let status = response.status().as_u16();
                let duration_ms = duration.as_millis();

                if response.status().is_success() {
                    if duration_ms >= SLOW_REQUEST_THRESHOLD_MS {
                        debug!(method, url, status, duration_ms, "Request completed (slow)");
                    } else {
                        trace!(method, url, status, duration_ms, "Request completed");
                    }
                    Ok(response)
                } else {
                    warn!(method, url, status, duration_ms, "Request failed");
                    Ok(response)
                }
            }
            Err(error) => {
                warn!(
                    method,
                    url,
                    duration_ms = duration.as_millis(),
                    "Request failed"
                );
                Err(error)
            }
        }
    }
}
