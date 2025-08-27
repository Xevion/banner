//! Banner API module for interacting with Ellucian Banner systems.
//!
//! This module provides functionality to:
//! - Search for courses and retrieve course information
//! - Manage Banner API sessions and authentication
//! - Scrape course data and cache it in Redis
//! - Generate ICS files and calendar links

pub mod api;
pub mod models;
pub mod query;
pub mod scraper;
pub mod session;

pub use api::BannerApi;
pub use models::*;
pub use query::SearchQuery;
pub use scraper::CourseScraper;
pub use session::SessionManager;
