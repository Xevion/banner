//! Bot commands module.

pub mod gcal;
pub mod ics;
pub mod search;
pub mod terms;
pub mod time;

pub use gcal::gcal;
pub use ics::ics;
pub use search::search;
pub use terms::terms;
pub use time::time;
