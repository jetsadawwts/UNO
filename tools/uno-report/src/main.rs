//! uno-report: one local HTML file with the UNO dashboard and the manual.
//!
//! Reads every project folder in the UNO data dir (default `~/UNO/data`):
//!   - `audit.log`  JSON lines written by the UNO hooks
//!   - `tasks.md`   the task table (columns found by header, `Status` column)
//! the manuals `<data>/manual.md` (English) and `<data>/manual.th.md` (Thai), and which
//! Claude Code sessions are open in `~/UNO/projects/<name>` right now (via `lsof`).
//! Writes `<data>/uno-report.html`. Nothing leaves the machine: no network, no external
//! scripts or fonts.
//!
//! Usage: uno-report [--data DIR] [--out FILE] [--manual FILE] [--open] [--watch MINUTES] [--serve [--port N]]


use std::collections::BTreeMap;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

use adapters_file::write_atomic;
use adapters_fs::{manual_stamp, FsManual, FsStore};
use adapters_os::{LsofSessions, SystemClock};
use adapters_web::{render_page, PageMode, Server};
use app::ports::{Clock, ManualSource, SessionProbe};

/// Test seam: `UNO_REPORT_FAKE_SESSIONS="alpha=1,beta=0"` → fixed counts, `"none"` → lsof unavailable.
fn fake_sessions(v: &str) -> Option<BTreeMap<String, usize>> {
    if v == "none" {
        return None;
    }
    Some(v.split(',').filter_map(|kv| kv.split_once('=')).filter_map(|(k, n)| Some((k.to_string(), n.parse().ok()?))).collect())
}


/// Test seams: `UNO_REPORT_FAKE_NOW` fixes the clock, `UNO_REPORT_FAKE_SESSIONS` the session counts.
struct LegacyClock;
impl Clock for LegacyClock {
    fn now(&self) -> String {
        env::var("UNO_REPORT_FAKE_NOW").unwrap_or_else(|_| SystemClock.now())
    }
}
struct LegacySessions;
impl SessionProbe for LegacySessions {
    fn open_sessions(&self) -> Option<BTreeMap<String, usize>> {
        match env::var("UNO_REPORT_FAKE_SESSIONS") {
            Ok(v) => fake_sessions(&v),
            Err(_) => LsofSessions.open_sessions(),
        }
    }
}

fn build(data: &Path, out: &Path, manual_path: &Path, watch: Option<u64>) -> Result<usize, String> {
    let d = app::build_dashboard(&FsStore { root: data.to_path_buf() }, &LegacySessions, &LegacyClock)?;
    let m = FsManual { en_path: manual_path.to_path_buf() }.manual();
    write_atomic(out, &render_page(&d, &m, &data.display().to_string(), &PageMode::File { watch }))?;
    Ok(d.projects.len())
}

fn now() -> String {
    LegacyClock.now()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let flag = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!("uno-report [--data DIR] [--out FILE] [--manual FILE] [--open] [--watch MINUTES] [--serve [--port N]]\n\
                  \n  default data: ~/UNO/data, out: <data>/uno-report.html\n\
                  \n  manual: <data>/manual.md (English) and <data>/manual.th.md (Thai, optional)\n\
                  \n  --watch N  regenerate every N minutes until stopped (Ctrl+C)\n\
                  \n  --serve    live dashboard at http://localhost:8765 (127.0.0.1 only), updates within seconds");
        return;
    }
    let home = env::var("HOME").unwrap_or_default();
    let data = PathBuf::from(flag("--data").unwrap_or_else(|| format!("{home}/UNO/data")));
    let out = PathBuf::from(flag("--out").unwrap_or_else(|| data.join("uno-report.html").display().to_string()));
    let manual_path = PathBuf::from(flag("--manual").unwrap_or_else(|| data.join("manual.md").display().to_string()));
    let watch: Option<u64> = flag("--watch").and_then(|v| v.parse().ok()).filter(|m| *m > 0);
    if args.iter().any(|a| a == "--serve") {
        let port: u16 = flag("--port").and_then(|v| v.parse().ok()).unwrap_or(8765);
        let stamp_path = manual_path.clone();
        Server {
            port,
            store: FsStore { root: data.clone() },
            manual: FsManual { en_path: manual_path },
            sessions: LegacySessions,
            clock: LegacyClock,
            data_label: data.display().to_string(),
            manual_stamp: move || manual_stamp(&stamp_path),
        }
        .run(args.iter().any(|a| a == "--open"));
    }

    let run = || match build(&data, &out, &manual_path, watch) {
        Ok(n) => println!("uno-report: {} · {n} projects → {}", now(), out.display()),
        Err(e) => eprintln!("uno-report: {e}"),
    };
    run();
    if args.iter().any(|a| a == "--open") {
        let _ = Command::new("open").arg(&out).status();
    }
    if let Some(m) = watch {
        println!("uno-report: regenerating every {m} min (Ctrl+C to stop)");
        loop {
            thread::sleep(Duration::from_secs(m * 60));
            run();
        }
    }
}
