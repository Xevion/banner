//! Data models for the Banner API.

pub mod common;
pub mod courses;
pub mod meetings;
pub mod search;
pub mod terms;

// Re-export commonly used types
pub use common::*;
pub use courses::*;
pub use meetings::*;
pub use search::*;
pub use terms::*;
