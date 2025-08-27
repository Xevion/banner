//! Bot commands module.

pub mod search;
pub mod terms;
pub mod time;
pub mod ics;
pub mod gcal;

pub use search::search;
pub use terms::terms;
pub use time::time;
pub use ics::ics;
pub use gcal::gcal;
