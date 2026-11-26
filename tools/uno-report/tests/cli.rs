//! CLI behaviour that must stay as in v0.5.12.
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn bin() -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_uno-report"));
    c.env("UNO_REPORT_FAKE_NOW", "2026-01-15 09:30").env("UNO_REPORT_FAKE_SESSIONS", "");
    c
}
fn data() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/data")
}
fn out(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("uno-cli-{name}-{}.html", std::process::id()))
}
fn run(args: &[&str]) -> Output {
    bin().args(args).output().unwrap()
}

#[test]
fn writes_file_and_reports() {
    let o = out("basic");
    let r = run(&["--data", data().to_str().unwrap(), "--out", o.to_str().unwrap()]);
    assert!(r.status.success());
    assert!(String::from_utf8_lossy(&r.stdout).contains("2 projects →"));
    assert!(o.exists());
    std::fs::remove_file(o).unwrap();
}

#[test]
fn watch_zero_and_garbage_mean_no_watch() {
    for w in ["0", "abc"] {
        let o = out(&format!("w{w}"));
        let r = run(&["--data", data().to_str().unwrap(), "--out", o.to_str().unwrap(), "--watch", w]);
        assert!(r.status.success(), "--watch {w} must exit after one build");
        assert!(!String::from_utf8_lossy(&r.stdout).contains("regenerating"));
        std::fs::remove_file(o).unwrap();
    }
}

#[test]
fn unknown_flag_is_ignored() {
    let o = out("unknown");
    let r = run(&["--bogus", "--data", data().to_str().unwrap(), "--out", o.to_str().unwrap(), "stray"]);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    assert!(o.exists());
    std::fs::remove_file(o).unwrap();
}

#[test]
fn short_help_matches_long_help() {
    assert_eq!(run(&["-h"]).stdout, run(&["--help"]).stdout);
}

#[test]
fn missing_data_dir_reports_and_exits_zero() {
    let r = run(&["--data", "/nonexistent-uno", "--out", out("missing").to_str().unwrap()]);
    assert!(r.status.success(), "v0.5.12 printed the error and exited 0");
    assert!(String::from_utf8_lossy(&r.stderr).starts_with("uno-report: cannot read /nonexistent-uno: "));
}

#[test]
fn serve_on_busy_port_fails() {
    let l = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = l.local_addr().unwrap().port().to_string();
    let r = run(&["--serve", "--port", &port, "--data", data().to_str().unwrap()]);
    assert_eq!(r.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&r.stderr).contains(&format!("cannot listen on 127.0.0.1:{port}")));
}

#[test]
fn repeated_value_flag_first_wins() {
    let o = out("repeat");
    let r = run(&["--data", data().to_str().unwrap(), "--data", "/nonexistent-uno", "--out", o.to_str().unwrap(), "--out", "/nonexistent-uno/x.html"]);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    assert!(String::from_utf8_lossy(&r.stdout).contains("2 projects →"));
    assert!(o.exists());
    std::fs::remove_file(o).unwrap();
}

#[test]
fn repeated_switch_stays_on() {
    let l = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = l.local_addr().unwrap().port().to_string();
    let r = run(&["--serve", "--serve", "--port", &port, "--data", data().to_str().unwrap()]);
    assert_eq!(r.status.code(), Some(1), "--serve --serve must still serve");
    assert_eq!(run(&["-h", "-h"]).stdout, run(&["--help"]).stdout);
}
