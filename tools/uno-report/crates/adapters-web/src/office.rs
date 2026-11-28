//! The Office tab: UNO's team in a pixel-art office. The server renders only data (`#ofc-data`, a
//! live pane) and the page frame; `OFFICE_JS` (assets.rs) draws the people, walks them between
//! their desk and lounge spots, and fills the side panels from that data.
//!
//! Data, all from what UNO already records:
//! - each member's newest heartbeat line (state, tool, task, since) — who works, who waits;
//! - agent calls in `audit.log` (Uno → member, description) — the hand-off list and animation;
//! - the status in `tasks.md` of each task id seen in a live line — the stage bar.
//!
//! A member sits at their desk only while working (a fresh line) or receiving a hand-off; Uno walks
//! out to that desk for each hand-off and otherwise stays in his office while any Claude Code
//! session is open. Everyone else takes a break around the office: where they go is decoration,
//! not data; the page says so.
use domain::{Handoff, Live, Project};

use crate::*;

const FLOOR: &[u8] = include_bytes!("../../../assets/office/floor.webp");
const PEOPLE: &[u8] = include_bytes!("../../../assets/office/people.webp");
const CEO_DESK: &[u8] = include_bytes!("../../../assets/office/ceo-desk.webp");
const LAYOUT: &str = include_str!("../../../assets/office/layout.json");
const CHAIRS: [&[u8]; 12] = [
    include_bytes!("../../../assets/office/chair01.webp"), include_bytes!("../../../assets/office/chair02.webp"),
    include_bytes!("../../../assets/office/chair03.webp"), include_bytes!("../../../assets/office/chair04.webp"),
    include_bytes!("../../../assets/office/chair05.webp"), include_bytes!("../../../assets/office/chair06.webp"),
    include_bytes!("../../../assets/office/chair07.webp"), include_bytes!("../../../assets/office/chair08.webp"),
    include_bytes!("../../../assets/office/chair09.webp"), include_bytes!("../../../assets/office/chair10.webp"),
    include_bytes!("../../../assets/office/chair11.webp"), include_bytes!("../../../assets/office/chair12.webp"),
];
const DESKS: [&[u8]; 12] = [
    include_bytes!("../../../assets/office/desk01.webp"), include_bytes!("../../../assets/office/desk02.webp"),
    include_bytes!("../../../assets/office/desk03.webp"), include_bytes!("../../../assets/office/desk04.webp"),
    include_bytes!("../../../assets/office/desk05.webp"), include_bytes!("../../../assets/office/desk06.webp"),
    include_bytes!("../../../assets/office/desk07.webp"), include_bytes!("../../../assets/office/desk08.webp"),
    include_bytes!("../../../assets/office/desk09.webp"), include_bytes!("../../../assets/office/desk10.webp"),
    include_bytes!("../../../assets/office/desk11.webp"), include_bytes!("../../../assets/office/desk12.webp"),
];

/// Name and role (English, Thai), in sprite-sheet row order (`people.webp`, `desk01..12`).
pub(crate) const MEMBERS: [(&str, &str, &str); 13] = [
    ("Uno", "Lead · chief of staff", "หัวหน้าทีม"),
    ("Alpha", "Intake triage", "คัดแยกงานเข้า"),
    ("Beta", "Codebase explorer", "สำรวจโค้ด"),
    ("Gamma", "UI designer", "ออกแบบ UI"),
    ("Delta", "Test writer", "เขียนเทสต์"),
    ("Epsilon", "Security reviewer", "ตรวจความปลอดภัย"),
    ("Zeta", "Release manager", "จัดการออกรุ่น"),
    ("Eta", "Incident scribe", "บันทึกเหตุการณ์"),
    ("Theta", "Doc writer", "เขียนเอกสาร"),
    ("Iota", "DevOps engineer", "DevOps"),
    ("Kappa", "Data engineer", "วิศวกรข้อมูล"),
    ("Lambda", "Architect", "สถาปนิกระบบ"),
    ("Mu", "Developer", "นักพัฒนา (เขียนโค้ด)"),
];

/// Most hand-offs listed.
const HANDOFFS: usize = 20;

/// `tasks.md` statuses that wait for a person (escalated, review or merge, approval).
const NEEDS_PERSON: [&str; 3] = ["escalated", "in-review", "proposed"];

fn webp(b: &[u8]) -> String {
    format!("data:image/webp;base64,{}", base64(b))
}

/// The art and its layout as one script, put once in the page head (not in the live pane).
pub(crate) fn office_style() -> String {
    let list = |xs: &[&[u8]]| xs.iter().map(|b| format!("\"{}\"", webp(b))).collect::<Vec<_>>().join(",");
    format!(
        "<script>window.UNO_OFFICE={{floor:\"{}\",people:\"{}\",ceo:\"{}\",chairs:[{}],desks:[{}],layout:{}}};</script>",
        webp(FLOOR), webp(PEOPLE), webp(CEO_DESK), list(&CHAIRS), list(&DESKS), LAYOUT.trim())
}

/// What a heartbeat state means, in both languages.
pub(crate) fn state_text(l: &Live) -> (String, String) {
    match l.state.as_str() {
        "running" => (format!("running {}", l.tool), format!("กำลังรัน {}", l.tool)),
        "between" => ("working".into(), "กำลังทำงาน".into()),
        "waiting" => ("waiting for you".into(), "รอคุณตอบ".into()),
        "done" => ("turn finished".into(), "ตอบรอบนั้นเสร็จ".into()),
        "error" => ("stopped by an error".into(), "หยุดเพราะ error".into()),
        other => (other.to_string(), other.to_string()),
    }
}

/// Newest line for `who` across all projects, with its project name.
fn newest<'a>(projects: &[&'a Project], who: &str) -> Option<(&'a str, &'a Live)> {
    projects.iter()
        .flat_map(|p| p.live.iter().map(move |l| (p.name.as_str(), l)))
        .filter(|(_, l)| l.who == who)
        .max_by(|a, b| a.1.time.cmp(&b.1.time))
}

fn member(kind: &str, who: &str, line: Option<(&str, &Live)>) -> String {
    match line {
        Some((project, l)) => format!(
            "<i class=\"{kind}\" data-who=\"{}\" data-project=\"{}\" data-time=\"{}\" data-since=\"{}\" data-state=\"{}\" data-tool=\"{}\" data-task=\"{}\"></i>",
            esc(who), esc(project), esc(&l.time), esc(&l.since), esc(&l.state), esc(&l.tool), esc(&l.task)),
        None => format!("<i class=\"{kind}\" data-who=\"{}\"></i>", esc(who)),
    }
}

/// The tab's live pane (`#ofc-data`): one entry per member (newest line), other agents with a
/// line, the latest hand-offs, and the status of each task id seen in a line. No visible text.
pub(crate) fn office(projects: &[&Project]) -> String {
    let mut out = String::from("<div class=\"ofc-data\" hidden>");
    // Open Claude Code sessions across projects: members sit at their desks while any is open.
    out += &format!("<i class=\"s\" data-open=\"{}\"></i>", projects.iter().map(|p| p.sessions).sum::<usize>());
    for (name, _, _) in MEMBERS {
        out += &member("m", name, newest(projects, name));
    }
    let mut others: Vec<(&str, &Live)> = projects.iter()
        .flat_map(|p| p.live.iter().map(move |l| (p.name.as_str(), l)))
        .filter(|(_, l)| !MEMBERS.iter().any(|m| m.0 == l.who))
        .collect();
    others.sort_by(|a, b| b.1.time.cmp(&a.1.time));
    for (project, l) in &others {
        out += &member("o", &l.who, Some((project, l)));
    }
    // Hand-offs with the member's work after each one, newest first across projects.
    let mut hand: Vec<(&str, Handoff, bool)> = projects.iter()
        .flat_map(|p| domain::handoffs(&p.calls, &p.steps).into_iter().map(move |h| {
            let ended = p.ended.get(&h.session).is_some_and(|end| *end >= h.time);
            (p.name.as_str(), h, ended)
        }))
        .collect();
    hand.sort_by(|a, b| b.1.time.cmp(&a.1.time));
    for (project, h, ended) in hand.into_iter().take(HANDOFFS) {
        let tools: Vec<String> = h.tools.iter().map(|(k, v)| format!("{k}:{v}")).collect();
        out += &format!("<i class=\"h\" data-time=\"{}\" data-who=\"{}\" data-desc=\"{}\" data-project=\"{}\" data-start=\"{}\" data-end=\"{}\" data-last=\"{}\" data-tools=\"{}\" data-closed=\"{}\"></i>",
                        esc(&h.time), esc(&h.who), esc(&h.description), esc(project), esc(&h.started), esc(&h.ended), esc(&h.last),
                        esc(&tools.join(",")), if ended { "1" } else { "" });
    }
    // Uno's turns that ended (waiting for the owner's reply), one per session.
    for p in projects {
        for l in p.live.iter().filter(|l| l.who == "Uno" && l.state == "done") {
            out += &format!("<i class=\"w\" data-project=\"{}\" data-time=\"{}\" data-task=\"{}\"></i>", esc(&p.name), esc(&l.time), esc(&l.task));
        }
    }
    // Tasks only a person can move on.
    for p in projects {
        for task in p.tasks.iter().filter(|t| NEEDS_PERSON.contains(&t.status.as_str())) {
            out += &format!("<i class=\"n\" data-project=\"{}\" data-id=\"{}\" data-status=\"{}\" data-title=\"{}\"></i>",
                            esc(&p.name), esc(&task.id), esc(&task.status), esc(&short(&task.title, 80)));
        }
    }
    let mut seen: Vec<(&str, &str)> = Vec::new();
    for p in projects {
        for l in &p.live {
            if l.task.is_empty() || seen.contains(&(p.name.as_str(), l.task.as_str())) {
                continue;
            }
            seen.push((p.name.as_str(), l.task.as_str()));
            if let Some(task) = p.tasks.iter().find(|t| t.id == l.task) {
                out += &format!("<i class=\"t\" data-project=\"{}\" data-id=\"{}\" data-status=\"{}\" data-title=\"{}\"></i>",
                                esc(&p.name), esc(&task.id), esc(&task.status), esc(&short(&task.title, 80)));
            }
        }
    }
    out + "</div>"
}

/// The whole tab: summary cards, team list, scene, details, must-see list, hand-offs, stage bar.
pub(crate) fn office_page(projects: &[&Project]) -> String {
    let roles: String = MEMBERS.iter()
        .map(|(n, en, th)| format!("<i data-who=\"{n}\" data-en=\"{}\" data-th=\"{}\"></i>", esc(en), esc(th)))
        .collect();
    format!(
        "<div class=\"ofc-head\"><h2>{}{} <span class=\"muted hint\">{}</span></h2>\
         <div class=\"ofc-tools\"><button type=\"button\" class=\"pill-btn\" id=\"ofc-zoom\" aria-pressed=\"false\">{}</button>\
         <button type=\"button\" class=\"pill-btn\" id=\"ofc-names\" aria-pressed=\"false\">{}</button></div></div>\
         <div class=\"ofc-roles\" hidden>{roles}</div>\
         <section class=\"ofc-stats\" id=\"ofc-stats\"></section>\
         <div class=\"ofc\">\
         <section class=\"ofc-card\"><h3>{}</h3><div class=\"ofc-roster\" id=\"ofc-roster\"></div></section>\
         <div class=\"ofc-mid\"><div class=\"ofc-scene\" id=\"ofc-sc\" role=\"img\" aria-label=\"UNO office: team members at their desks or away\"><div class=\"ofc-stage\" id=\"ofc-st\"></div></div>\
         <section class=\"ofc-card ofc-detail\" id=\"ofc-detail\"></section></div>\
         <div class=\"ofc-side\"><section class=\"ofc-card ofc-must\"><h3>{}</h3><ul id=\"ofc-must\"></ul></section>\
         <section class=\"ofc-card\"><h3>{}</h3><ol class=\"ofc-hand\" id=\"ofc-hand\"></ol><div class=\"ofc-legend\" id=\"ofc-legend\"></div></section></div>\
         </div>\
         <section class=\"ofc-card ofc-stagebar\" id=\"ofc-stagebar\" hidden></section>\
         <div id=\"ofc-data\">{}</div>\
         <p class=\"muted small\">{}</p>",
        icon("office"), t("Office", "ออฟฟิศ"),
        t("From UNO's hooks: heartbeats, agent calls in audit.log and tasks.md", "จาก hook ของ UNO: heartbeat, การเรียก agent ใน audit.log และ tasks.md"),
        t("Uno's room", "ห้อง Uno"), t("Show names", "แสดงชื่อ"),
        t("Team", "ทีม"), t("Needs you", "ต้องดู"), t("Hand-offs from Uno", "การส่งงานจาก Uno"),
        office(projects),
        t("A member sits at their desk only while working on something from Uno; Uno walks to that desk for each hand-off. The rest of the time members take a break around the office. Uno stays in his office while a Claude Code session with UNO is open (offline: no session in the last 30 minutes, then he takes a break too). Where people go on a break (kitchen, lounge, restroom and so on) is decoration, not data.",
          "ลูกทีมจะนั่งโต๊ะเฉพาะตอนที่กำลังทำงานที่ Uno ส่งมา และ Uno จะเดินไปที่โต๊ะนั้นทุกครั้งที่ส่งงาน เวลาอื่นลูกทีมเดินพักอยู่ในออฟฟิศ Uno อยู่ในห้องตัวเองระหว่างที่มี session ของ Claude Code ที่ใช้ UNO เปิดอยู่ (ออฟไลน์ คือไม่มี session ใน 30 นาทีล่าสุด Uno ก็ไปพักด้วย) ที่ที่เดินไปพัก (ห้องครัว ห้องพักผ่อน ห้องน้ำ ฯลฯ) เป็นของตกแต่ง ไม่ได้มาจากข้อมูลจริง"))
}

fn base64(bytes: &[u8]) -> String {
    // The standard alphabet, built from ranges so it never reads as one long key-like string.
    let a: Vec<u8> = (b'A'..=b'Z').chain(b'a'..=b'z').chain(b'0'..=b'9').chain(*b"+/").collect();
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for c in bytes.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        for i in 0..4 {
            out.push(if i <= c.len() { a[(n >> (18 - 6 * i) & 63) as usize] as char } else { '=' });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{AuditRecord, Task};

    fn beat(name: &str, state: &str, task: &str, time: &str) -> AuditRecord {
        AuditRecord { action: "heartbeat".into(), session: format!("s-{name}-{time}"), time: time.into(),
                      since: "2026-01-02T03:09:00+00:00".into(), state: state.into(), tool: "Bash".into(),
                      task: task.into(), name: name.into(), ..Default::default() }
    }
    fn call(name: &str, desc: &str, time: &str) -> AuditRecord {
        AuditRecord { action: "called".into(), tool: "Agent".into(), name: name.into(), description: desc.into(),
                      session: "s".into(), time: time.into(), ..Default::default() }
    }
    fn project(name: &str, recs: &[AuditRecord]) -> Project {
        let mut p = Project { name: name.into(), sessions: 1, ..Default::default() };
        recs.iter().for_each(|r| p.apply(r));
        p.finish();
        p
    }
    fn entry<'a>(html: &'a str, kind: &str, who: &str) -> &'a str {
        let i = html.find(&format!("<i class=\"{kind}\" data-who=\"{who}\"")).unwrap_or_else(|| panic!("{kind} {who}: {html}"));
        &html[i..i + html[i..].find("</i>").unwrap()]
    }

    #[test]
    fn base64_matches_the_standard() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn style_carries_all_art_and_the_layout() {
        let s = office_style();
        assert_eq!(s.matches("data:image/webp;base64,UklGR").count(), 3 + 12 + 12, "floor, people, ceo, chairs, desks");
        assert!(s.contains("layout:{") && s.contains("\"desks\"") && s.contains("\"seat\""));
    }

    #[test]
    fn every_member_has_an_entry_in_sprite_order() {
        let html = office(&[]);
        let mut at = 0;
        for m in MEMBERS {
            let i = html.find(&format!("<i class=\"m\" data-who=\"{}\"></i>", m.0)).expect(m.0);
            assert!(i >= at, "{} out of order", m.0);
            at = i;
        }
        assert!(html.starts_with("<div class=\"ofc-data\" hidden>") && html.ends_with("</div>"));
    }

    #[test]
    fn open_sessions_are_summed_for_online() {
        let (mut a, mut b) = (project("a", &[]), project("b", &[]));
        a.sessions = 2;
        b.sessions = 1;
        assert!(office(&[&a, &b]).contains("<i class=\"s\" data-open=\"3\"></i>"));
        b.sessions = 0;
        a.sessions = 0;
        assert!(office(&[&a, &b]).contains("data-open=\"0\""));
    }

    #[test]
    fn member_with_a_line_carries_its_fields() {
        let p = project("proj", &[beat("core:codebase-explorer", "running", "T38", "2026-01-02T03:10:00+00:00")]);
        let html = office(&[&p]);
        let b = entry(&html, "m", "Beta");
        for want in ["data-project=\"proj\"", "data-time=\"2026-01-02T03:10:00+00:00\"", "data-state=\"running\"", "data-tool=\"Bash\"", "data-task=\"T38\""] {
            assert!(b.contains(want), "{want}: {b}");
        }
        assert_eq!(entry(&html, "m", "Alpha"), "<i class=\"m\" data-who=\"Alpha\">");
    }

    #[test]
    fn newest_line_wins_across_projects() {
        let a = project("a", &[beat("", "done", "T1", "2026-01-02T03:00:00+00:00")]);
        let b = project("b", &[beat("", "waiting", "T2", "2026-01-02T03:05:00+00:00")]);
        let u = office(&[&a, &b]);
        let u = entry(&u, "m", "Uno");
        assert!(u.contains("data-state=\"waiting\"") && u.contains("data-task=\"T2\"") && u.contains("data-project=\"b\""), "{u}");
    }

    fn step(event: &str, agent: &str, name: &str, tool: &str, time: &str) -> AuditRecord {
        AuditRecord { action: "office".into(), event: event.into(), agent: agent.into(), name: name.into(), tool: tool.into(),
                      session: "s".into(), time: time.into(), ..Default::default() }
    }

    #[test]
    fn hand_off_carries_the_members_work() {
        let p = project("proj", &[call("core:security-reviewer", "Review T2", "2026-01-02T03:00:00+00:00"),
                                  step("start", "a1", "core:security-reviewer", "", "2026-01-02T03:00:05+00:00"),
                                  step("tool", "a1", "core:security-reviewer", "Read", "2026-01-02T03:01:00+00:00"),
                                  step("tool", "a1", "core:security-reviewer", "Read", "2026-01-02T03:02:00+00:00"),
                                  step("tool", "a1", "core:security-reviewer", "Bash", "2026-01-02T03:03:00+00:00"),
                                  step("stop", "a1", "core:security-reviewer", "", "2026-01-02T03:04:00+00:00")]);
        let html = office(&[&p]);
        assert!(html.contains("data-who=\"Epsilon\" data-desc=\"Review T2\" data-project=\"proj\" data-start=\"2026-01-02T03:00:05+00:00\" data-end=\"2026-01-02T03:04:00+00:00\" data-last=\"2026-01-02T03:04:00+00:00\" data-tools=\"Bash:1,Read:2\" data-closed=\"\""), "{html}");
    }

    #[test]
    fn needs_you_has_finished_uno_turns_and_tasks_for_people() {
        let mut p = project("proj", &[beat("", "done", "T9", "2026-01-02T03:10:00+00:00"), beat("core:test-writer", "done", "T9", "2026-01-02T03:11:00+00:00")]);
        for (id, status) in [("T1", "proposed"), ("T2", "in-review"), ("T3", "escalated"), ("T4", "in-progress"), ("T5", "done")] {
            p.tasks.push(domain::Task { id: id.into(), title: format!("<{id}>"), status: status.into() });
        }
        let html = office(&[&p]);
        assert_eq!(html.matches("<i class=\"w\"").count(), 1, "only Uno's turn: {html}");
        assert!(html.contains("<i class=\"w\" data-project=\"proj\" data-time=\"2026-01-02T03:10:00+00:00\" data-task=\"T9\"></i>"));
        for id in ["T1", "T2", "T3"] {
            assert!(html.contains(&format!("<i class=\"n\" data-project=\"proj\" data-id=\"{id}\"")), "{id}");
        }
        assert!(!html.contains("data-id=\"T4\" data-status") && !html.contains("data-id=\"T5\""));
        assert!(html.contains("data-title=\"&lt;T1&gt;\""));
    }

    #[test]
    fn hand_offs_are_newest_first_and_escaped() {
        let p = project("proj", &[call("core:ui-designer", "Draw <the> office", "2026-01-02T03:00:00+00:00"),
                                  call("core:test-writer", "Write tests", "2026-01-02T03:05:00+00:00")]);
        let html = office(&[&p]);
        let d = html.find("data-who=\"Delta\" data-desc=\"Write tests\"").expect("Delta");
        let g = html.find("data-who=\"Gamma\" data-desc=\"Draw &lt;the&gt; office\"").expect("Gamma");
        assert!(d < g, "newest first: {html}");
        assert!(!html.contains("<the>"));
    }

    #[test]
    fn task_status_is_listed_only_for_tasks_seen_in_a_line() {
        let mut p = project("proj", &[beat("", "running", "T7", "2026-01-02T03:10:00+00:00")]);
        p.tasks = vec![Task { id: "T7".into(), title: "Office tab".into(), status: "in-review".into() },
                       Task { id: "T8".into(), title: "Other".into(), status: "ready".into() }];
        let html = office(&[&p]);
        assert!(html.contains("<i class=\"t\" data-project=\"proj\" data-id=\"T7\" data-status=\"in-review\" data-title=\"Office tab\"></i>"), "{html}");
        assert!(!html.contains("data-id=\"T8\""));
    }

    #[test]
    fn unknown_agents_are_other_entries_and_escaped() {
        let p = project("proj", &[beat("x:<odd>", "between", "", "2026-01-02T03:10:00+00:00")]);
        let html = office(&[&p]);
        assert!(html.contains("<i class=\"o\" data-who=\"&lt;odd&gt;\""), "{html}");
        assert!(!html.contains("<odd>"));
    }

    #[test]
    fn page_has_every_panel_and_the_data_pane() {
        let html = office_page(&[]);
        for id in ["ofc-stats", "ofc-roster", "ofc-sc", "ofc-st", "ofc-detail", "ofc-must", "ofc-hand", "ofc-legend", "ofc-stagebar", "ofc-zoom", "ofc-names"] {
            assert!(html.contains(&format!("id=\"{id}\"")), "{id}");
        }
        assert!(html.contains(&format!("<div id=\"ofc-data\">{}</div>", office(&[]))));
        assert!(html.contains("decoration, not data"));
    }
}
