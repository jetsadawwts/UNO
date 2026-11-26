//! Ports: what the use cases need from the outside world. Adapters implement them.
use std::collections::BTreeMap;

use domain::{AuditRecord, Task};

/// The raw files of one project folder, already parsed.
pub struct ProjectFiles {
    pub name: String,
    pub audit: Vec<AuditRecord>,
    pub tasks: Vec<Task>,
}

pub trait ProjectStore {
    /// Projects sorted by name. Folders starting with `_` or `.`, and folders with neither
    /// `audit.log` nor `tasks.md`, are left out. Err carries a message for people.
    fn projects(&self) -> Result<Vec<ProjectFiles>, String>;
    /// Cheap fingerprint (sizes and modification times) of the files the page shows.
    fn stamp(&self) -> String;
}

/// Manual source text (Markdown) in each language, and where it was looked for.
pub struct ManualText {
    pub en: Option<String>,
    pub th: Option<String>,
    pub location: String,
}

pub trait ManualSource {
    fn manual(&self) -> ManualText;
}

pub trait SessionProbe {
    /// Open Claude Code sessions by project name; None when they cannot be counted.
    fn open_sessions(&self) -> Option<BTreeMap<String, usize>>;
}

pub trait Clock {
    /// Local time as `YYYY-MM-DD HH:MM`.
    fn now(&self) -> String;
}
