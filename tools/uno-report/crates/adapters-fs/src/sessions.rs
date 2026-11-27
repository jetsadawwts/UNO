//! Open Claude Code sessions per project, from the `session-start` / `session-end` events and
//! session ids that UNO's hooks write to each `audit.log`. Nothing outside `~/UNO/data` is read.
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use app::ports::{ProjectStore, SessionProbe};
use domain::{open_sessions, SESSION_IDLE_SECS};

use crate::FsStore;

pub struct AuditSessions {
    pub root: PathBuf,
}

impl SessionProbe for AuditSessions {
    /// Always known (an unreadable data folder simply has no sessions).
    fn open_sessions(&self) -> Option<BTreeMap<String, usize>> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
        Some(sessions_at(&self.root, now))
    }
}

/// Projects with at least one open session at `now` (seconds since 1970).
pub fn sessions_at(root: &Path, now: i64) -> BTreeMap<String, usize> {
    let projects = FsStore { root: root.to_path_buf() }.projects().unwrap_or_default();
    projects
        .into_iter()
        .map(|p| (p.name, open_sessions(&p.audit, now, SESSION_IDLE_SECS)))
        .filter(|(_, n)| *n > 0)
        .collect()
}
