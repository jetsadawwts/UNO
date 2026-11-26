//! Change feed for the live page: `notify` (FSEvents on macOS, inotify on Linux) on `~/UNO/data`.
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use app::ports::ChangeFeed;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

/// Events closer together than this count as one change.
const QUIET: Duration = Duration::from_millis(300);

/// False for files this program writes itself and for snapshot folders.
pub fn relevant(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    name != "uno-report.html" && !name.ends_with(".tmp") && !path.components().any(|c| c.as_os_str() == "trash")
}

struct Armed {
    _watcher: RecommendedWatcher,
    rx: Receiver<notify::Result<Event>>,
    root_id: Option<(u64, u64)>,
}

pub struct FsChangeFeed {
    root: PathBuf,
    manual_dir: Option<PathBuf>,
    armed: Mutex<Option<Armed>>,
}

fn identity(p: &Path) -> Option<(u64, u64)> {
    std::fs::metadata(p).ok().map(|m| (m.dev(), m.ino()))
}

impl FsChangeFeed {
    /// Watch `root` recursively, and the manual's folder when it is outside `root`.
    pub fn new(root: &Path, manual_en: &Path) -> Result<Self, String> {
        let manual_dir = manual_en.parent().filter(|d| !d.starts_with(root)).map(Path::to_path_buf);
        let feed = FsChangeFeed { root: root.to_path_buf(), manual_dir, armed: Mutex::new(None) };
        let armed = feed.arm()?;
        *feed.armed.lock().unwrap() = Some(armed);
        Ok(feed)
    }

    fn arm(&self) -> Result<Armed, String> {
        let (tx, rx) = channel();
        let mut w = notify::recommended_watcher(tx).map_err(|e| format!("cannot watch {}: {e}", self.root.display()))?;
        w.watch(&self.root, RecursiveMode::Recursive).map_err(|e| format!("cannot watch {}: {e}", self.root.display()))?;
        if let Some(d) = &self.manual_dir {
            let _ = w.watch(d, RecursiveMode::NonRecursive);
        }
        Ok(Armed { _watcher: w, rx, root_id: identity(&self.root) })
    }

    /// The watched root itself went away: its watch is dead even if a new folder appears
    /// with the same inode number.
    fn root_removed(&self, ev: &notify::Result<Event>) -> bool {
        matches!(ev, Ok(e) if matches!(e.kind, EventKind::Remove(_)) && e.paths.iter().any(|p| p == &self.root))
    }
}

/// Reads (`Access`) never count: rendering the page opens every data file, and counting
/// those would make the live page refresh itself forever.
fn is_relevant(ev: &notify::Result<Event>) -> bool {
    match ev {
        Ok(e) if matches!(e.kind, EventKind::Access(_)) => false,
        Ok(e) => e.paths.is_empty() || e.paths.iter().any(|p| relevant(p)),
        Err(_) => true,
    }
}

impl ChangeFeed for FsChangeFeed {
    fn wait(&self, timeout: Duration) -> bool {
        let mut slot = self.armed.lock().unwrap();
        // The root was removed or replaced: the old watch is dead. Re-arm once it exists again.
        let current = identity(&self.root);
        if slot.as_ref().map(|a| a.root_id) != Some(current) {
            *slot = if current.is_some() { self.arm().ok() } else { None };
            if slot.is_some() {
                return true; // contents may differ; let the caller re-check
            }
        }
        let Some(armed) = slot.as_ref() else {
            std::thread::sleep(timeout);
            return identity(&self.root).is_some();
        };
        let deadline = Instant::now() + timeout;
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            match armed.rx.recv_timeout(left) {
                Ok(ev) if self.root_removed(&ev) => {
                    *slot = None;
                    return true;
                }
                Ok(ev) if is_relevant(&ev) => break,
                Ok(_) => continue,
                Err(RecvTimeoutError::Timeout) => return false,
                Err(RecvTimeoutError::Disconnected) => {
                    *slot = None;
                    return true;
                }
            }
        }
        // Drain the burst: stop after QUIET with no relevant event.
        loop {
            match armed.rx.recv_timeout(QUIET) {
                Ok(ev) if self.root_removed(&ev) => {
                    *slot = None;
                    return true;
                }
                Ok(ev) if is_relevant(&ev) => continue,
                Ok(_) => continue,
                Err(_) => return true,
            }
        }
    }
}
