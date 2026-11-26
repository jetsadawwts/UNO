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

#[test]
fn records_without_full_time_do_not_panic() {
    struct Odd;
    impl ProjectStore for Odd {
        fn projects(&self) -> Result<Vec<ProjectFiles>, String> {
            let a = AuditRecord { event: "plan-approved".into(), task: "T1".into(), ..Default::default() };
            let b = AuditRecord { event: "smoke".into(), time: "2026".into(), ..Default::default() };
            Ok(vec![ProjectFiles { name: "alpha".into(), audit: vec![a, b], tasks: vec![] }])
        }
        fn stamp(&self) -> String { String::new() }
    }
    let d = app::build_dashboard(&Odd, &Sess(None), &Clk).unwrap();
    let page = render_page(&d, &Man.manual(), "/d", &PageMode::Live);
    assert!(page.contains("alpha"));
}

#[test]
fn live_page_has_socket_script() {
    let d = app::build_dashboard(&Store, &Sess(None), &Clk).unwrap();
    let page = render_page(&d, &Man.manual(), "/d", &PageMode::Live);
    assert!(page.contains("new WebSocket") && page.contains("/ws"));
    assert!(page.contains("fetch('/version'"), "polling stays as the fallback");
    let file = render_page(&d, &Man.manual(), "/d", &PageMode::File { watch: None });
    assert!(!file.contains("new WebSocket"));
}
