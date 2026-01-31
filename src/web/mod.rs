//! Web API module for the banner application.

pub mod admin;
pub mod admin_rmp;
pub mod admin_scraper;
#[cfg(feature = "embed-assets")]
pub mod assets;
pub mod auth;
pub mod calendar;
#[cfg(feature = "embed-assets")]
pub mod encoding;
pub mod error;
pub mod extractors;
pub mod routes;
pub mod schedule_cache;
pub mod session_cache;
pub mod timeline;
pub mod ws;

pub use routes::*;
