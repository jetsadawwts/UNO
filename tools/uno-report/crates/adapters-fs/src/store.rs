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
            out.push(ProjectFiles { name, audit: parse_audit(&read(&audit)), tasks: parse_tasks(&read(&tasks)) });
        }
        Ok(out)
    }

    fn stamp(&self) -> String {
        let mut sig = String::new();
        for d in self.dirs().unwrap_or_default() {
            sig += &file_stamp(&d.join("audit.log"));
            sig += &file_stamp(&d.join("tasks.md"));
        }
        sig
    }
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
