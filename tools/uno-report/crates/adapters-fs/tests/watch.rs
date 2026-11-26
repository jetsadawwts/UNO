use adapters_fs::{relevant, FsChangeFeed};
use app::ports::ChangeFeed;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

fn temp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("uno-watch-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(d.join("alpha")).unwrap();
    std::fs::write(d.join("alpha/audit.log"), "").unwrap();
    d
}
fn append(p: &Path) {
    let mut f = std::fs::OpenOptions::new().append(true).create(true).open(p).unwrap();
    writeln!(f, "{{}}").unwrap();
}
fn feed(d: &Path) -> FsChangeFeed {
    let f = FsChangeFeed::new(d, &d.join("manual.md")).unwrap();
    std::thread::sleep(Duration::from_millis(200)); // let the watcher settle
    f
}

#[test]
fn relevant_ignores_own_output_tmp_and_trash() {
    assert!(!relevant(Path::new("/d/uno-report.html")));
    assert!(!relevant(Path::new("/d/x/uno-report.html.tmp")));
    assert!(!relevant(Path::new("/d/alpha/trash/2026/a.txt")));
    assert!(relevant(Path::new("/d/alpha/audit.log")));
    assert!(relevant(Path::new("/d/manual.th.md")));
}

#[test]
fn write_is_seen_within_a_second() {
    let d = temp("seen");
    let f = feed(&d);
    let p = d.join("alpha/audit.log");
    let t = std::thread::spawn(move || { std::thread::sleep(Duration::from_millis(100)); append(&p) });
    let start = Instant::now();
    assert!(f.wait(Duration::from_secs(3)));
    assert!(start.elapsed() < Duration::from_secs(1), "{:?}", start.elapsed());
    t.join().unwrap();
    std::fs::remove_dir_all(d).unwrap();
}

#[test]
fn burst_is_one_change() {
    let d = temp("burst");
    let f = feed(&d);
    let p = d.join("alpha/audit.log");
    let t = std::thread::spawn(move || for _ in 0..50 { append(&p); std::thread::sleep(Duration::from_millis(5)) });
    assert!(f.wait(Duration::from_secs(3)));
    t.join().unwrap();
    assert!(!f.wait(Duration::from_millis(400)));
    std::fs::remove_dir_all(d).unwrap();
}

#[test]
fn own_output_is_not_a_change() {
    let d = temp("own");
    let f = feed(&d);
    std::fs::write(d.join("uno-report.html"), "x").unwrap();
    std::fs::write(d.join("uno-report.html.tmp"), "x").unwrap();
    assert!(!f.wait(Duration::from_millis(800)));
    std::fs::remove_dir_all(d).unwrap();
}

#[test]
fn watch_survives_root_recreated() {
    let d = temp("recreate");
    let f = feed(&d);
    std::fs::remove_dir_all(&d).unwrap();
    std::fs::create_dir_all(d.join("alpha")).unwrap();
    let _ = f.wait(Duration::from_millis(500)); // sees the removal and re-arms
    std::thread::sleep(Duration::from_millis(200));
    append(&d.join("alpha/audit.log"));
    let seen = f.wait(Duration::from_secs(3)) || f.wait(Duration::from_secs(3));
    assert!(seen);
    std::fs::remove_dir_all(d).unwrap();
}

#[test]
fn reading_files_is_not_a_change() {
    let d = temp("read");
    let f = feed(&d);
    let _ = std::fs::read_to_string(d.join("alpha/audit.log"));
    let _ = std::fs::read_dir(&d).map(|r| r.count());
    assert!(!f.wait(Duration::from_millis(800)), "the page's own reads must not trigger a refresh");
    std::fs::remove_dir_all(d).unwrap();
}

#[test]
fn steady_writes_still_report_within_about_a_second() {
    let d = temp("steady");
    let f = feed(&d);
    let p = d.join("alpha/audit.log");
    let t = std::thread::spawn(move || for _ in 0..30 { append(&p); std::thread::sleep(Duration::from_millis(100)) });
    let start = Instant::now();
    assert!(f.wait(Duration::from_secs(5)));
    assert!(start.elapsed() < Duration::from_millis(1500), "drain must stop ~1 s after the first event: {:?}", start.elapsed());
    t.join().unwrap();
    std::fs::remove_dir_all(d).unwrap();
}

#[test]
fn starts_without_root_and_watches_once_it_appears() {
    let d = std::env::temp_dir().join(format!("uno-watch-late-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    let f = FsChangeFeed::new(&d, &d.join("manual.md")).expect("a missing data folder is not an error");
    assert!(!f.wait(Duration::from_millis(300)));
    std::fs::create_dir_all(d.join("alpha")).unwrap();
    let _ = f.wait(Duration::from_millis(500)); // arms now that the folder exists
    std::thread::sleep(Duration::from_millis(200));
    append(&d.join("alpha/audit.log"));
    assert!(f.wait(Duration::from_secs(3)), "the new watch sees the write");
    append(&d.join("alpha/audit.log"));
    assert!(f.wait(Duration::from_secs(3)), "and keeps seeing writes");
    std::fs::remove_dir_all(d).unwrap();
}
