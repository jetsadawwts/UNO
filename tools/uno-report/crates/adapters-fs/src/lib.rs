//! Reads `~/UNO/data`: one folder per project with `audit.log` and `tasks.md`, plus the manuals.
mod audit;
mod store;
mod tasks;

pub use audit::parse_audit;
pub use store::{manual_stamp, FsManual, FsStore};
pub use tasks::parse_tasks;
