//! Web API module for the banner application.

#[cfg(feature = "embed-assets")]
pub mod assets;
pub mod routes;

pub use routes::*;
