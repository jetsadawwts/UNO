//! uno-report application layer: ports the adapters implement, and the use cases.
mod dashboard;
pub mod ports;
mod signature;

pub use dashboard::{build_dashboard, Dashboard};
pub use signature::signature;
