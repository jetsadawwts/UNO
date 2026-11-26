//! uno-report domain: plain data and the rules that count it. No I/O.
mod agent;
mod event;
mod project;
mod task;

pub use agent::agent_name;
pub use event::{AuditRecord, Event, EventKind};
pub use project::Project;
pub use task::{clip, is_open, Task, DONE, DROPPED, RECENT_KEEP};
