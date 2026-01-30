//! Web API module for the banner application.

pub mod admin;
#[cfg(feature = "embed-assets")]
pub mod assets;
pub mod auth;
#[cfg(feature = "embed-assets")]
pub mod encoding;
pub mod extractors;
pub mod routes;
pub mod session_cache;
pub mod ws;

pub use routes::*;
