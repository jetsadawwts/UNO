//! Golden test: the page built from the fake fixture data must stay byte-identical to the saved
//! copy, except the Office art, which is checked for its type and replaced by `<OFFICE-ART>`.
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_uno-report"))
        .args(args)
        .env("UNO_REPORT_FAKE_NOW", "2026-01-15 09:30")
        .env("UNO_REPORT_FAKE_SESSIONS", "alpha=1,beta=0")
        .output()
        .unwrap()
}

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/data")
}

#[test]
fn golden_file_output_matches() {
    let out = std::env::temp_dir().join(format!("uno-golden-{}.html", std::process::id()));
    let data = fixture();
    let o = run(&["--data", data.to_str().unwrap(), "--out", out.to_str().unwrap()]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let got = std::fs::read_to_string(&out).unwrap().replace(data.to_str().unwrap(), "<DATA>")
        .replace(concat!("dashboard v", env!("CARGO_PKG_VERSION")), "dashboard v<VERSION>");
    let got = without_picture(&got);
    let _ = std::fs::remove_file(&out);
    assert_eq!(got, include_str!("golden/report.html"));
}

#[test]
fn golden_help_matches() {
    let o = run(&["--help"]);
    assert_eq!(String::from_utf8_lossy(&o.stdout), include_str!("golden/help.txt"));
}

#[test]
fn file_page_has_no_live_script() {
    let page = include_str!("golden/report.html");
    assert!(!page.contains("new WebSocket"));
    assert!(!page.contains("fetch('/version'"), "polling belongs to the live page only");
}

/// The embedded Office art (27 WebP images, ~500 KB of base64); keep it out of the saved copy.
fn without_picture(page: &str) -> String {
    let head = "data:image/webp;base64,";
    let (mut out, mut rest, mut n) = (String::new(), page, 0);
    while let Some(i) = rest.find(head) {
        let start = i + head.len();
        let end = start + rest[start..].find('"').unwrap();
        assert!(rest[start..end].starts_with("UklGR"), "office art is WebP");
        out += &rest[..start];
        out += "<OFFICE-ART>";
        rest = &rest[end..];
        n += 1;
    }
    assert_eq!(n, 27, "floor, people, Uno's desk, 12 chairs, 12 desks");
    out + rest
}
