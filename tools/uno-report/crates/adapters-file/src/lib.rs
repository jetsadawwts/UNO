//! Writes the dashboard page to a file.
use std::fs;
use std::path::Path;

/// Write to `<out>.html.tmp`, then rename over `out`, so a reload never sees a half-written page.
pub fn write_atomic(out: &Path, page: &str) -> Result<(), String> {
    let tmp = out.with_extension("html.tmp");
    fs::write(&tmp, page).map_err(|e| format!("cannot write {}: {e}", tmp.display()))?;
    fs::rename(&tmp, out).map_err(|e| format!("cannot write {}: {e}", out.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_atomic_replaces_and_leaves_no_tmp() {
        let d = std::env::temp_dir().join(format!("uno-file-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let out = d.join("uno-report.html");
        write_atomic(&out, "one").unwrap();
        write_atomic(&out, "two").unwrap();
        assert_eq!(std::fs::read_to_string(&out).unwrap(), "two");
        assert!(!d.join("uno-report.html.tmp").exists());
        std::fs::remove_dir_all(d).unwrap();
    }

    #[test]
    fn write_atomic_reports_missing_folder() {
        let e = write_atomic(std::path::Path::new("/nonexistent-uno/x.html"), "p").unwrap_err();
        assert!(e.starts_with("cannot write /nonexistent-uno/x.html.tmp: "), "{e}");
    }
}
