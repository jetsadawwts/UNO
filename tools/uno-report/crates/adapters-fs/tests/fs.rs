use adapters_fs::*;
use app::ports::{ManualSource, ProjectStore};
use std::path::{Path, PathBuf};

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/data")
}

fn temp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("uno-fs-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn parse_audit_skips_garbage_and_non_utf8() {
    let bad = String::from_utf8_lossy(&[0xff, 0xfe]).to_string();
    let text = format!("not json\n{{}}\n{bad}\n{{\"time\":\"t\",\"event\":\"smoke\",\"extra\":1}}\n[1,2]\n");
    let r = parse_audit(&text);
    assert_eq!(r.len(), 3, "{{}}, the smoke line, and [1,2] parse as JSON");
    assert_eq!(r[0], domain::AuditRecord::default());
    assert_eq!((r[1].time.as_str(), r[1].event.as_str()), ("t", "smoke"));
}

#[test]
fn parse_audit_non_string_fields_are_empty() {
    let r = parse_audit("{\"time\":5,\"event\":null,\"task\":[\"T1\"]}");
    assert_eq!(r[0], domain::AuditRecord::default());
}

#[test]
fn parse_tasks_normalises_status_and_dedupes() {
    let text = "| ID | Task | Status |\n|---|---|---|\n| T1 | A | **Done** |\n| T2 | B | `dropped` |\n| T3 | C | In progress |\n| T1 | dup | open |\n| T2a | x | open |\n| X1 | y | open |\n| T | z | open |\nfree text\n";
    let t = parse_tasks(text);
    let got: Vec<_> = t.iter().map(|t| (t.id.as_str(), t.title.as_str(), t.status.as_str())).collect();
    assert_eq!(got, [("T1", "A", "done"), ("T2", "B", "dropped"), ("T3", "C", "in")]);
}

#[test]
fn parse_tasks_without_header_is_empty() {
    assert!(parse_tasks("| T1 | A | open |\n").is_empty());
}

#[test]
fn store_lists_fixture_projects() {
    let p = FsStore { root: fixture() }.projects().unwrap();
    let names: Vec<_> = p.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, ["alpha", "beta"]);
    assert_eq!(p[0].audit.len(), 11);
    assert_eq!(p[0].tasks.len(), 4);
    assert!(p[1].audit.is_empty());
    assert_eq!(p[1].tasks.len(), 1);
}

#[test]
fn store_keeps_folder_with_empty_files() {
    let d = temp("empty");
    std::fs::create_dir_all(d.join("delta")).unwrap();
    std::fs::write(d.join("delta/audit.log"), "").unwrap();
    let p = FsStore { root: d.clone() }.projects().unwrap();
    assert_eq!(p.len(), 1);
    assert!(p[0].audit.is_empty() && p[0].tasks.is_empty());
    std::fs::remove_dir_all(d).unwrap();
}

#[test]
fn store_missing_root_errors() {
    let e = FsStore { root: PathBuf::from("/nonexistent-uno") }.projects().err().unwrap();
    assert!(e.starts_with("cannot read /nonexistent-uno: "), "{e}");
}

#[test]
fn stamp_changes_when_a_file_grows() {
    let d = temp("stamp");
    std::fs::create_dir_all(d.join("a")).unwrap();
    std::fs::write(d.join("a/audit.log"), "x").unwrap();
    let s = FsStore { root: d.clone() };
    let before = s.stamp();
    std::fs::write(d.join("a/audit.log"), "xy").unwrap();
    assert_ne!(before, s.stamp());
    std::fs::remove_dir_all(d).unwrap();
}

#[test]
fn manual_reads_both_languages() {
    let m = FsManual { en_path: fixture().join("manual.md") }.manual();
    assert!(m.en.unwrap().starts_with("# Fixture manual"));
    assert!(m.th.unwrap().starts_with("# คู่มือทดสอบ"));
    assert_eq!(m.location, fixture().join("manual.md").display().to_string());
}

#[test]
fn manual_falls_back_when_th_missing() {
    let d = temp("man");
    std::fs::write(d.join("manual.md"), "# EN").unwrap();
    let m = FsManual { en_path: d.join("manual.md") }.manual();
    assert_eq!((m.en.as_deref(), m.th.as_deref()), (Some("# EN"), None));
    assert!(!manual_stamp(&d.join("manual.md")).is_empty());
    std::fs::remove_dir_all(d).unwrap();
}
