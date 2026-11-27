//! One version for UNO: the dashboard binary and the plugin are released together.
use std::process::Command;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[test]
fn matches_the_plugin_manifest() {
    let manifest = include_str!("../../../plugins/core/.claude-plugin/plugin.json");
    assert!(manifest.contains(&format!("\"version\": \"{VERSION}\"")), "Cargo.toml says {VERSION}; plugin.json differs");
}

#[test]
fn version_flag_prints_it_and_writes_nothing() {
    for flag in ["--version", "-V"] {
        let o = Command::new(env!("CARGO_BIN_EXE_uno-report")).args([flag, "--data", "/nonexistent"]).output().unwrap();
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
        assert_eq!(String::from_utf8_lossy(&o.stdout), format!("uno-report {VERSION}\n"));
    }
}

#[test]
fn page_header_shows_it() {
    let data = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/data");
    let out = std::env::temp_dir().join(format!("uno-version-{}.html", std::process::id()));
    let o = Command::new(env!("CARGO_BIN_EXE_uno-report")).args(["--data", data, "--out", out.to_str().unwrap()]).output().unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let page = std::fs::read_to_string(&out).unwrap();
    let _ = std::fs::remove_file(&out);
    let header = &page[page.find("<header>").unwrap()..page.find("</header>").unwrap()];
    assert!(header.contains(&format!("<span class=\"ver\">dashboard v{VERSION}</span>")), "{header}");
}
