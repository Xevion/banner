//! Web API module for the banner application.

pub mod admin;
#[cfg(feature = "embed-assets")]
pub mod assets;
pub mod auth;
pub mod extractors;
pub mod routes;
pub mod session_cache;

pub use routes::*;
