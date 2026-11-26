use adapters_web::*;
use app::ports::*;
use app::Dashboard;
use domain::{AuditRecord, Task};
use std::collections::BTreeMap;

struct Store;
impl ProjectStore for Store {
    fn projects(&self) -> Result<Vec<ProjectFiles>, String> {
        let r = AuditRecord { event: "smoke".into(), time: "2026-01-01T00:00".into(), ..Default::default() };
        Ok(vec![ProjectFiles { name: "alpha".into(), audit: vec![r], tasks: vec![Task { id: "T1".into(), title: "x".into(), status: "open".into() }] }])
    }
    fn stamp(&self) -> String { "s".into() }
}
struct Man;
impl ManualSource for Man {
    fn manual(&self) -> ManualText { ManualText { en: Some("# Hi".into()), th: None, location: "/m/manual.md".into() } }
}
struct Sess(Option<BTreeMap<String, usize>>);
impl SessionProbe for Sess {
    fn open_sessions(&self) -> Option<BTreeMap<String, usize>> { self.0.clone() }
}
struct Clk;
impl Clock for Clk {
    fn now(&self) -> String { "2026-01-15 09:30".into() }
}

fn server() -> Server<Store, Man, Sess, Clk, fn() -> String> {
    fn stamp() -> String { "m".into() }
    Server { port: 8765, store: Store, manual: Man, sessions: Sess(None), clock: Clk, data_label: "/d".into(), manual_stamp: stamp }
}

fn req(method: &str, path: &str, host: &str) -> String {
    format!("{method} {path} HTTP/1.1\r\nHost: {host}\r\nAccept: */*\r\n\r\n")
}

#[test]
fn handle_rejects_foreign_host() {
    assert_eq!(server().handle(&req("GET", "/", "evil.example:8765")).0, 403);
    assert_eq!(server().handle(&req("GET", "/", "localhost:9999")).0, 403);
    assert_eq!(server().handle("GET / HTTP/1.1\r\n\r\n").0, 403);
}

#[test]
fn handle_accepts_both_local_hosts() {
    assert_eq!(server().handle(&req("GET", "/version", "localhost:8765")).0, 200);
    assert_eq!(server().handle(&req("GET", "/version", "127.0.0.1:8765")).0, 200);
}

#[test]
fn handle_rejects_post() {
    assert_eq!(server().handle(&req("POST", "/", "localhost:8765")).0, 405);
}

#[test]
fn handle_unknown_path() {
    assert_eq!(server().handle(&req("GET", "/etc/passwd", "localhost:8765")).0, 404);
}

#[test]
fn handle_version_is_16_hex() {
    let (code, ctype, body) = server().handle(&req("GET", "/version?x=1", "localhost:8765"));
    assert_eq!((code, ctype), (200, "text/plain"));
    let b = String::from_utf8(body).unwrap();
    assert_eq!(b.len(), 16);
    assert!(b.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn handle_root_is_live_page() {
    let (code, ctype, body) = server().handle(&req("GET", "/", "localhost:8765"));
    assert_eq!((code, ctype), (200, "text/html; charset=utf-8"));
    let b = String::from_utf8(body).unwrap();
    assert!(b.contains("window.UNO_LIVE=true;"));
    assert!(b.contains("alpha"));
}

#[test]
fn handle_reports_store_error_as_500() {
    struct Bad;
    impl ProjectStore for Bad {
        fn projects(&self) -> Result<Vec<ProjectFiles>, String> { Err("cannot read /x: gone".into()) }
        fn stamp(&self) -> String { String::new() }
    }
    fn stamp() -> String { String::new() }
    let s = Server { port: 1, store: Bad, manual: Man, sessions: Sess(None), clock: Clk, data_label: String::new(), manual_stamp: stamp as fn() -> String };
    let (code, _, body) = s.handle(&req("GET", "/", "localhost:1"));
    assert_eq!(code, 500);
    assert_eq!(body, b"cannot read /x: gone");
}

#[test]
fn file_page_has_no_live_flag_and_watch_note() {
    let d = app::build_dashboard(&Store, &Sess(None), &Clk).unwrap();
    let page = render_page(&d, &Man.manual(), "/d", &PageMode::File { watch: Some(5) });
    assert!(!page.contains("window.UNO_LIVE=true;"));
    assert!(page.contains("regenerated every 5 min"));
}

#[test]
fn sessions_unknown_panel() {
    let d = app::build_dashboard(&Store, &Sess(None), &Clk).unwrap();
    let page = render_page(&d, &Man.manual(), "/d", &PageMode::Live);
    assert!(page.contains("data-sessions=\"?\""));
    assert!(page.contains("? open session(s)"));
}

#[test]
fn sessions_known_panel() {
    let d: Dashboard = app::build_dashboard(&Store, &Sess(Some(BTreeMap::from([("alpha".into(), 2)]))), &Clk).unwrap();
    let page = render_page(&d, &Man.manual(), "/d", &PageMode::Live);
    assert!(page.contains("data-sessions=\"2\""));
}

#[test]
fn manual_only_en_copies_to_th() {
    let h = manual_html(&ManualText { en: Some("# A".into()), th: None, location: "/m".into() });
    assert_eq!(h, "<div class=\"l-en\"><h1>A</h1>\n</div><div class=\"l-th\"><h1>A</h1>\n</div>");
}

#[test]
fn manual_only_th_copies_to_en() {
    let h = manual_html(&ManualText { en: None, th: Some("# ก".into()), location: "/m".into() });
    assert_eq!(h, "<div class=\"l-en\"><h1>ก</h1>\n</div><div class=\"l-th\"><h1>ก</h1>\n</div>");
}

#[test]
fn manual_none_shows_location_message() {
    let h = manual_html(&ManualText { en: None, th: None, location: "/m/manual.md".into() });
    assert!(h.contains("No manual found at /m/manual.md — add manual.md there and run again."));
    assert!(h.contains("ไม่พบคู่มือที่ /m/manual.md — วางไฟล์ manual.md ไว้แล้วรันใหม่"));
}
