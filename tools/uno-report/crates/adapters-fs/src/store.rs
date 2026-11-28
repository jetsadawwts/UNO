use std::fs;
use std::path::{Path, PathBuf};

use app::ports::{ManualSource, ManualText, ProjectFiles, ProjectStore};

use crate::{parse_audit, parse_tasks};

/// `~/UNO/data`: one folder per project.
pub struct FsStore {
    pub root: PathBuf,
}

impl FsStore {
    fn dirs(&self) -> std::io::Result<Vec<PathBuf>> {
        let mut dirs: Vec<PathBuf> = fs::read_dir(&self.root)?.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect();
        dirs.sort();
        Ok(dirs)
    }
}

impl ProjectStore for FsStore {
    fn projects(&self) -> Result<Vec<ProjectFiles>, String> {
        let dirs = self.dirs().map_err(|e| format!("cannot read {}: {e}", self.root.display()))?;
        let mut out = Vec::new();
        for dir in dirs {
            let name = dir.file_name().unwrap_or_default().to_string_lossy().to_string();
            if name.starts_with('_') || name.starts_with('.') {
                continue;
            }
            let (audit, tasks) = (dir.join("audit.log"), dir.join("tasks.md"));
            if !audit.exists() && !tasks.exists() {
                continue;
            }
            let read = |p: &Path| fs::read_to_string(p).unwrap_or_default();
            let mut records = parse_audit(&read(&audit));
            // One small file per session from trace_heartbeat.py.
            for beat in beat_files(&dir) {
                records.extend(parse_audit(&read(&beat)).into_iter().filter(|r| r.action == "heartbeat"));
            }
            // Work trail of subagents from trace_heartbeat.py, for the Office tab.
            records.extend(parse_audit(&read(&dir.join("office.log"))).into_iter().filter(|r| r.action == "office"));
            out.push(ProjectFiles { name, audit: records, tasks: parse_tasks(&read(&tasks)) });
        }
        Ok(out)
    }

    fn stamp(&self) -> String {
        let mut sig = String::new();
        for d in self.dirs().unwrap_or_default() {
            sig += &file_stamp(&d.join("audit.log"));
            sig += &file_stamp(&d.join("tasks.md"));
            sig += &file_stamp(&d.join("office.log"));
            for beat in beat_files(&d) {
                sig += &file_stamp(&beat);
            }
        }
        sig
    }
}

/// `heartbeat/*.json` in one project folder, sorted by path.
fn beat_files(project: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(project.join("heartbeat")) else { return Vec::new() };
    let mut files: Vec<PathBuf> = entries.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "json")).collect();
    files.sort();
    files
}

/// `manual.md` (English) and `manual.th.md` next to it (Thai).
pub struct FsManual {
    pub en_path: PathBuf,
}

impl ManualSource for FsManual {
    fn manual(&self) -> ManualText {
        ManualText {
            en: fs::read_to_string(&self.en_path).ok(),
            th: fs::read_to_string(self.en_path.with_file_name("manual.th.md")).ok(),
            location: self.en_path.display().to_string(),
        }
    }
}

/// Fingerprint of both manuals, for the live page's change check.
pub fn manual_stamp(en_path: &Path) -> String {
    file_stamp(en_path) + &file_stamp(&en_path.with_file_name("manual.th.md"))
}

fn file_stamp(p: &Path) -> String {
    fs::metadata(p).ok().map(|m| format!("{}:{:?};", m.len(), m.modified().ok())).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root(tag: &str) -> PathBuf {
        let r = std::env::temp_dir().join(format!("uno-store-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&r);
        fs::create_dir_all(r.join("p/heartbeat")).unwrap();
        fs::write(r.join("p/audit.log"), "{\"time\":\"2026-01-02T03:00:00+00:00\",\"session\":\"a\",\"action\":\"session-start\"}\n").unwrap();
        r
    }
    const BEAT: &str = "{\"time\":\"2026-01-02T03:50:00+00:00\",\"session\":\"b\",\"action\":\"heartbeat\",\"tool\":\"Read\",\"name\":\"\"}";

    #[test]
    fn heartbeat_files_are_read_with_the_audit_records() {
        let r = root("read");
        fs::write(r.join("p/heartbeat/b.json"), BEAT).unwrap();
        fs::write(r.join("p/heartbeat/b.tmp"), BEAT).unwrap();
        fs::write(r.join("p/heartbeat/broken.json"), "{").unwrap();
        fs::write(r.join("p/heartbeat/other.json"), "{\"action\":\"blocked\",\"time\":\"t\"}").unwrap();
        let files = FsStore { root: r.clone() }.projects().unwrap();
        let actions: Vec<_> = files[0].audit.iter().map(|a| a.action.as_str()).collect();
        assert_eq!(actions, ["session-start", "heartbeat"]);
        assert_eq!(crate::sessions_at(&r, domain::parse_time("2026-01-02T04:00:00+00:00").unwrap()).get("p"), Some(&1));
        let _ = fs::remove_dir_all(&r);
    }

    #[test]
    fn stamp_changes_with_a_heartbeat() {
        let r = root("stamp");
        let store = FsStore { root: r.clone() };
        let before = store.stamp();
        fs::write(r.join("p/heartbeat/b.json"), BEAT).unwrap();
        assert_ne!(before, store.stamp());
        let _ = fs::remove_dir_all(&r);
    }
}
