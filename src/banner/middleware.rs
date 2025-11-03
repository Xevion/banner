//! HTTP middleware for the Banner API client.

use http::Extensions;
use reqwest::{Request, Response};
use reqwest_middleware::{Middleware, Next};
use tracing::{debug, trace, warn};

pub struct TransparentMiddleware;

/// Threshold for logging slow requests at DEBUG level (in milliseconds)
const SLOW_REQUEST_THRESHOLD_MS: u128 = 1000;

#[async_trait::async_trait]
impl Middleware for TransparentMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> std::result::Result<Response, reqwest_middleware::Error> {
        let method = req.method().to_string();
        let path = req.url().path().to_string();

        let start = std::time::Instant::now();
        let response_result = next.run(req, extensions).await;
        let duration = start.elapsed();

        match response_result {
            Ok(response) => {
                if response.status().is_success() {
                    let duration_ms = duration.as_millis();
                    if duration_ms >= SLOW_REQUEST_THRESHOLD_MS {
                        debug!(
                            method = method,
                            path = path,
                            status = response.status().as_u16(),
                            duration_ms = duration_ms,
                            "Request completed (slow)"
                        );
                    } else {
                        trace!(
                            method = method,
                            path = path,
                            status = response.status().as_u16(),
                            duration_ms = duration_ms,
                            "Request completed"
                        );
                    }
                    Ok(response)
                } else {
                    let e = response.error_for_status_ref().unwrap_err();
                    warn!(
                        method = method,
                        path = path,
                        error = ?e,
                        status = response.status().as_u16(),
                        duration_ms = duration.as_millis(),
                        "Request failed"
                    );
                    Ok(response)
                }
            }
            Err(error) => {
                warn!(
                    method = method,
                    path = path,
                    error = ?error,
                    duration_ms = duration.as_millis(),
                    "Request failed"
                );
                Err(error)
            }
        }
    }
}
