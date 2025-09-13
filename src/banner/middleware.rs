//! HTTP middleware for the Banner API client.

use http::Extensions;
use reqwest::{Request, Response};
use reqwest_middleware::{Middleware, Next};
use tracing::{trace, warn};

pub struct TransparentMiddleware;

#[async_trait::async_trait]
impl Middleware for TransparentMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> std::result::Result<Response, reqwest_middleware::Error> {
        trace!(
            domain = req.url().domain(),
            headers = ?req.headers(),
            "{method} {path}",
            method = req.method().to_string(),
            path = req.url().path(),
        );
        let response_result = next.run(req, extensions).await;

        match response_result {
            Ok(response) => {
                if response.status().is_success() {
                    trace!(
                        "{code} {reason} {path}",
                        code = response.status().as_u16(),
                        reason = response.status().canonical_reason().unwrap_or("??"),
                        path = response.url().path(),
                    );
                    Ok(response)
                } else {
                    let e = response.error_for_status_ref().unwrap_err();
                    warn!(error = ?e, "Request failed (server)");
                    Ok(response)
                }
            }
            Err(error) => {
                warn!(error = ?error, "Request failed (middleware)");
                Err(error)
            }
        }
    }
}
