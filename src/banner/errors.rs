//! Error types for the Banner API client.

use thiserror::Error;

#[derive(Debug, thiserror::Error)]
pub enum BannerApiError {
    #[error("Banner session is invalid or expired: {0}")]
    InvalidSession(String),
    #[error(transparent)]
    RequestFailed(#[from] anyhow::Error),
}
