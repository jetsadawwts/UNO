//! The live page in named pieces, and the socket message that carries only what changed.
use std::collections::BTreeMap;

use app::ports::ManualText;
use serde_json::{json, Map, Value};

use crate::manual_html;
use crate::views::{meta_html, overview, project_view, psel_options};

/// Inner HTML of each live-updated element, by element id, plus the header note.
#[derive(Clone, Debug, PartialEq)]
pub struct Panes {
    pub panes: BTreeMap<String, String>,
    pub meta: String,
}

/// Pane ids: `psel` (picker options), `all`, `p0`…, `ofc-data` (Office data), `manual` — the same HTML `render_page` emits.
pub fn render_panes(d: &app::Dashboard, m: &ManualText, data_label: &str) -> Panes {
    let mut panes = BTreeMap::new();
    panes.insert("psel".to_string(), psel_options(&d.projects));
    panes.insert("all".to_string(), overview(&d.projects, d.sessions_known));
    for (i, p) in d.projects.iter().enumerate() {
        panes.insert(format!("p{i}"), project_view(p, d.sessions_known));
    }
    panes.insert("ofc-data".to_string(), crate::office(&d.projects.iter().collect::<Vec<_>>()));
    panes.insert("manual".to_string(), manual_html(m));
    Panes { panes, meta: meta_html(d, data_label, None) }
}

/// `{"sig","panes","meta"}` with every pane when there is no `prev`, only changed panes when the
/// ids are the same, or `{"sig","reload":true}` when projects were added or removed.
pub fn diff_message(sig: &str, prev: Option<&Panes>, next: &Panes) -> String {
    let changed: Map<String, Value> = match prev {
        Some(p) if !p.panes.keys().eq(next.panes.keys()) => return json!({ "sig": sig, "reload": true }).to_string(),
        Some(p) => next.panes.iter().filter(|(k, v)| p.panes.get(*k) != Some(v)).map(|(k, v)| (k.clone(), json!(v))).collect(),
        None => next.panes.iter().map(|(k, v)| (k.clone(), json!(v))).collect(),
    };
    json!({ "sig": sig, "panes": changed, "meta": next.meta }).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use app::ports::*;
    use domain::{AuditRecord, Task};
    use std::collections::BTreeMap;

    struct Store(usize, &'static str);
    impl ProjectStore for Store {
        fn projects(&self) -> Result<Vec<ProjectFiles>, String> {
            Ok((0..self.0).map(|i| ProjectFiles {
                name: format!("proj{i}"),
                audit: vec![AuditRecord { event: "smoke".into(), time: self.1.into(), ..Default::default() }],
                tasks: vec![Task { id: "T1".into(), title: "x".into(), status: "open".into() }],
            }).collect())
        }
        fn stamp(&self) -> String { String::new() }
    }
    struct Sess;
    impl SessionProbe for Sess { fn open_sessions(&self) -> Option<BTreeMap<String, usize>> { None } }
    struct Clk;
    impl Clock for Clk { fn now(&self) -> String { "2026-01-15 09:30".into() } }
    fn man() -> ManualText { ManualText { en: Some("# M".into()), th: None, location: "/m".into() } }
    fn dash(n: usize, time: &'static str) -> app::Dashboard { app::build_dashboard(&Store(n, time), &Sess, &Clk).unwrap() }

    #[test]
    fn pane_ids_are_psel_all_projects_manual() {
        let p = render_panes(&dash(2, "2026-01-01T00:00"), &man(), "/d");
        let ids: Vec<_> = p.panes.keys().map(String::as_str).collect();
        assert_eq!(ids, ["all", "manual", "ofc-data", "p0", "p1", "psel"]);
    }

    #[test]
    fn panes_match_page() {
        let d = dash(2, "2026-01-01T00:00");
        let page = crate::render_page(&d, &man(), "/d", &crate::PageMode::Live);
        let p = render_panes(&d, &man(), "/d");
        for (id, html) in &p.panes {
            assert!(page.contains(html.as_str()), "pane {id} not in page");
        }
        assert!(page.contains(&format!("<div class=\"meta\" style=\"flex-basis:100%\">{}</div>", p.meta)));
    }

    fn json(s: &str) -> serde_json::Value { serde_json::from_str(s).unwrap() }

    #[test]
    fn diff_first_is_full() {
        let p = render_panes(&dash(1, "2026-01-01T00:00"), &man(), "/d");
        let v = json(&diff_message("abc", None, &p));
        assert_eq!(v["sig"], "abc");
        assert_eq!(v["panes"].as_object().unwrap().len(), 5);
        assert!(v["meta"].is_string());
        assert!(v.get("reload").is_none());
    }

    #[test]
    fn diff_same_keys_only_changed() {
        let a = render_panes(&dash(1, "2026-01-01T00:00"), &man(), "/d");
        let b = render_panes(&dash(1, "2026-01-02T00:00"), &man(), "/d");
        let v = json(&diff_message("s2", Some(&a), &b));
        let keys: Vec<_> = v["panes"].as_object().unwrap().keys().cloned().collect();
        assert!(keys.contains(&"all".to_string()) && keys.contains(&"p0".to_string()));
        assert!(!keys.contains(&"manual".to_string()));
        let same = json(&diff_message("s1", Some(&a), &a));
        assert_eq!(same["panes"].as_object().unwrap().len(), 0);
    }

    #[test]
    fn diff_new_project_is_reload() {
        let a = render_panes(&dash(1, "2026-01-01T00:00"), &man(), "/d");
        let b = render_panes(&dash(2, "2026-01-01T00:00"), &man(), "/d");
        let v = json(&diff_message("s", Some(&a), &b));
        assert_eq!(v["reload"], true);
        assert!(v.get("panes").is_none());
    }

    #[test]
    fn card_time_is_the_activity_time_not_session_end() {
        let rec = |action: &str, time: &str| AuditRecord { action: action.into(), tool: "Agent".into(), name: "core:intake-triager".into(), session: "s".into(), time: time.into(), ..Default::default() };
        let mut p = domain::Project { name: "x".into(), ..Default::default() };
        p.apply(&rec("called", "2026-01-02T03:05:49+00:00"));
        p.apply(&rec("session-end", "2026-01-02T03:47:56+00:00"));
        p.finish();
        let card = crate::views::activity_panel(&[&p], true);
        assert!(card.contains("data-last=\"2026-01-02T03:05:49+00:00\""), "{card}");
        assert!(card.contains("data-time=\"2026-01-02T03:05:49+00:00\""), "{card}");
        assert!(!card.contains("03:47:56"), "{card}");
    }

    fn card_for(state: &str, tool: &str, task: &str, name: &str) -> String {
        let mut p = domain::Project { name: "x".into(), sessions: 1, ..Default::default() };
        p.apply(&AuditRecord { action: "heartbeat".into(), session: "s".into(), time: "2026-01-02T03:10:00+00:00".into(), since: "2026-01-02T03:09:00+00:00".into(), state: state.into(), tool: tool.into(), task: task.into(), name: name.into(), ..Default::default() });
        p.finish();
        crate::views::activity_panel(&[&p], true)
    }

    #[test]
    fn card_shows_what_each_session_is_doing() {
        for (state, en, th) in [("running", "running Bash", "กำลังรัน Bash"), ("between", "working", "กำลังทำงาน"),
                                ("waiting", "waiting for you", "รอคุณตอบ"), ("done", "turn finished", "ตอบรอบนั้นเสร็จ"),
                                ("error", "stopped by an error", "หยุดเพราะ error")] {
            let card = card_for(state, "Bash", "T38", "");
            assert!(card.contains(&format!("T38 · Uno: {en}")), "{state}: {card}");
            assert!(card.contains(&format!("T38 · Uno: {th}")), "{state}: {card}");
            assert!(card.contains("class=\"lc-line\" data-time=\"2026-01-02T03:10:00+00:00\""), "{card}");
            assert!(card.contains("class=\"since\" data-since=\"2026-01-02T03:09:00+00:00\""), "{card}");
        }
    }

    #[test]
    fn card_without_task_has_no_prefix_and_escapes_values() {
        let card = card_for("running", "<b>", "", "core:codebase-explorer");
        assert!(card.contains("Beta: running &lt;b&gt;") && !card.contains("<b>:") && !card.contains(" · Beta"), "{card}");
    }

    fn card_with_hook_version(version: &str) -> String {
        let mut p = domain::Project { name: "x".into(), sessions: 1, ..Default::default() };
        p.apply(&AuditRecord { action: "heartbeat".into(), session: "s".into(), time: "2026-01-02T03:10:00+00:00".into(), since: "2026-01-02T03:09:00+00:00".into(), state: "between".into(), version: version.into(), ..Default::default() });
        p.finish();
        crate::views::activity_panel(&[&p], true)
    }

    #[test]
    fn card_line_shows_the_hook_version() {
        let same = card_with_hook_version(env!("CARGO_PKG_VERSION"));
        assert!(same.contains(&format!("<span class=\"hv\"> · hook v{}</span>", env!("CARGO_PKG_VERSION"))), "{same}");
        assert!(!same.contains("differs from dashboard"), "{same}");
    }

    #[test]
    fn card_line_flags_a_hook_version_that_differs() {
        let card = card_with_hook_version("0.0.1<");
        assert!(card.contains("class=\"hv diff\"") && card.contains("hook v0.0.1&lt;"), "{card}");
        assert!(card.contains("differs from dashboard") && card.contains("ไม่ตรงกับ dashboard"), "{card}");
    }

    #[test]
    fn card_line_without_a_hook_version_shows_none() {
        assert!(!card_with_hook_version("").contains("hook v"));
    }

    #[test]
    fn message_escapes_quotes_newlines_and_thai() {
        let mut p = Panes { panes: BTreeMap::new(), meta: "a\"b\\c\nกข\u{1}".into() };
        p.panes.insert("all".into(), "<p class=\"x\">\t</p>".into());
        let v = json(&diff_message("s", None, &p));
        assert_eq!(v["meta"], "a\"b\\c\nกข\u{1}");
        assert_eq!(v["panes"]["all"], "<p class=\"x\">\t</p>");
    }
}
