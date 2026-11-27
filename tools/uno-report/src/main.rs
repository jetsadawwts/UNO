//! uno-report: one local HTML page with the UNO dashboard and the manual.
//!
//! Composition root: parses the command line and wires the adapters to the use cases.
//! See `ARCHITECTURE.md` for the layers. Reads every project folder in the UNO data dir
//! (default `~/UNO/data`) and writes `<data>/uno-report.html`, or serves it live on
//! 127.0.0.1. Nothing leaves the machine: no network, no external scripts or fonts.
//!
//! Usage: uno-report [--data DIR] [--out FILE] [--manual FILE] [--open] [--watch MINUTES] [--serve [--port N]]


use std::collections::BTreeMap;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

use clap::Parser;

use adapters_file::write_atomic;
use adapters_fs::{manual_stamp, AuditSessions, FsChangeFeed, FsManual, FsStore};
use adapters_os::SystemClock;
use adapters_web::{render_page, PageMode, Server};
use app::ports::{ChangeFeed, Clock, ManualSource, SessionProbe};

/// Test seam: `UNO_REPORT_FAKE_SESSIONS="alpha=1,beta=0"` → fixed counts, `"none"` → lsof unavailable.
fn fake_sessions(v: &str) -> Option<BTreeMap<String, usize>> {
    if v == "none" {
        return None;
    }
    Some(v.split(',').filter_map(|kv| kv.split_once('=')).filter_map(|(k, n)| Some((k.to_string(), n.parse().ok()?))).collect())
}


/// Test seams: `UNO_REPORT_FAKE_NOW` fixes the clock, `UNO_REPORT_FAKE_SESSIONS` the session counts.
struct LocalClock;
impl Clock for LocalClock {
    fn now(&self) -> String {
        env::var("UNO_REPORT_FAKE_NOW").unwrap_or_else(|_| SystemClock.now())
    }
}
/// Open sessions from the data folder's audit events (`AuditSessions`), or the test seam.
struct LocalSessions(PathBuf);
impl SessionProbe for LocalSessions {
    fn open_sessions(&self) -> Option<BTreeMap<String, usize>> {
        match env::var("UNO_REPORT_FAKE_SESSIONS") {
            Ok(v) => fake_sessions(&v),
            Err(_) => AuditSessions { root: self.0.clone() }.open_sessions(),
        }
    }
}

fn build(data: &Path, out: &Path, manual_path: &Path, watch: Option<u64>) -> Result<usize, String> {
    let d = app::build_dashboard(&FsStore { root: data.to_path_buf() }, &LocalSessions(data.to_path_buf()), &LocalClock)?;
    let m = FsManual { en_path: manual_path.to_path_buf() }.manual();
    write_atomic(out, &render_page(&d, &m, &data.display().to_string(), &PageMode::File { watch }))?;
    Ok(d.projects.len())
}

/// Without file events (the watcher could not start) the page still updates every `TICK`.
struct TickOnly;
impl ChangeFeed for TickOnly {
    fn wait(&self, timeout: Duration) -> bool {
        thread::sleep(timeout);
        false
    }
}

/// Session counts have no events, so the watcher also re-checks every `TICK`.
const TICK: Duration = Duration::from_secs(5);

/// Background thread: file changes (or every `TICK`) → new signature → every live socket.
fn spawn_watcher(data: PathBuf, manual_path: PathBuf, tx: tokio::sync::watch::Sender<String>) {
    thread::spawn(move || {
        let store = FsStore { root: data.clone() };
        let feed: Box<dyn ChangeFeed> = match FsChangeFeed::new(&data, &manual_path) {
            Ok(f) => Box::new(f),
            Err(e) => {
                eprintln!("uno-report: {e}; checking every {} s instead", TICK.as_secs());
                Box::new(TickOnly)
            }
        };
        let mut watch = app::Watch::new();
        loop {
            if let Some(sig) = watch.step(feed.as_ref(), &store, &manual_stamp(&manual_path), &LocalSessions(data.clone()), TICK) {
                tx.send_replace(sig);
            }
        }
    });
}

fn now() -> String {
    LocalClock.now()
}

const HELP: &str = "uno-report [--data DIR] [--out FILE] [--manual FILE] [--open] [--watch MINUTES] [--serve [--port N]]\n\
                    \n  default data: ~/UNO/data, out: <data>/uno-report.html\n\
                    \n  manual: <data>/manual.md (English) and <data>/manual.th.md (Thai, optional)\n\
                    \n  --watch N  regenerate every N minutes until stopped (Ctrl+C)\n\
                    \n  --serve    live dashboard at http://localhost:8765 (127.0.0.1 only), updates within seconds";

/// Flags as in v0.5.12. Values are parsed leniently: a bad `--watch` means no watch, a bad
/// `--port` means 8765, and unknown arguments are ignored (`ignore_errors`).
#[derive(Parser)]
#[command(name = "uno-report", disable_help_flag = true, disable_version_flag = true, ignore_errors = true)]
struct Cli {
    #[arg(long)]
    data: Option<PathBuf>,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long)]
    manual: Option<PathBuf>,
    #[arg(long)]
    open: bool,
    #[arg(long)]
    watch: Option<String>,
    #[arg(long)]
    serve: bool,
    #[arg(long)]
    port: Option<String>,
    #[arg(short = 'h', long = "help")]
    help: bool,
}

/// Make argv look like what v0.5.12 accepted before clap sees it: unknown arguments are
/// dropped (clap stops at the first one), and each flag keeps only its first occurrence
/// (v0.5.12 read the first value; clap rejects repeats). Value flags take the next argument.
fn known_args(args: impl Iterator<Item = String>) -> Vec<String> {
    const VALUE: &[&str] = &["--data", "--out", "--manual", "--watch", "--port"];
    const SWITCH: &[&str] = &["--open", "--serve", "--help", "-h"];
    let mut args = args;
    let mut out: Vec<String> = args.next().into_iter().collect();
    let mut seen: Vec<String> = Vec::new();
    while let Some(a) = args.next() {
        let key = if a == "-h" { "--help".to_string() } else { a.clone() };
        let first = !seen.contains(&key);
        if VALUE.contains(&a.as_str()) {
            if let Some(v) = args.next() {
                if first {
                    out.extend([a, v]);
                }
            }
        } else if SWITCH.contains(&a.as_str()) && first {
            out.push(a);
        }
        seen.push(key);
    }
    out
}

fn main() {
    let cli = Cli::parse_from(known_args(env::args()));
    if cli.help {
        println!("{HELP}");
        return;
    }
    let home = env::var("HOME").unwrap_or_default();
    let data = cli.data.unwrap_or_else(|| PathBuf::from(format!("{home}/UNO/data")));
    let out = cli.out.unwrap_or_else(|| data.join("uno-report.html"));
    let manual_path = cli.manual.unwrap_or_else(|| data.join("manual.md"));
    let watch: Option<u64> = cli.watch.and_then(|v| v.parse().ok()).filter(|m| *m > 0);
    if cli.serve {
        let port: u16 = cli.port.and_then(|v| v.parse().ok()).unwrap_or(8765);
        let stamp_path = manual_path.clone();
        let (changes_tx, changes) = tokio::sync::watch::channel(String::new());
        spawn_watcher(data.clone(), manual_path.clone(), changes_tx);
        let server = Server {
            port,
            store: FsStore { root: data.clone() },
            manual: FsManual { en_path: manual_path },
            sessions: LocalSessions(data.clone()),
            clock: LocalClock,
            data_label: data.display().to_string(),
            manual_stamp: move || manual_stamp(&stamp_path),
            changes,
        };
        let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
        rt.block_on(server.run(cli.open));
        std::process::exit(0);
    }

    let run = || match build(&data, &out, &manual_path, watch) {
        Ok(n) => println!("uno-report: {} · {n} projects → {}", now(), out.display()),
        Err(e) => eprintln!("uno-report: {e}"),
    };
    run();
    if cli.open {
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
