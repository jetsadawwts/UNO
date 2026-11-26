//! uno-report: one local HTML file with the UNO dashboard and the manual.
//!
//! Reads every project folder in the UNO data dir (default `~/UNO/data`):
//!   - `audit.log`  JSON lines written by the UNO hooks
//!   - `tasks.md`   the task table (columns found by header, `Status` column)
//! and the manual from `<data>/manual.md`. Writes `<data>/uno-report.html`.
//! Nothing leaves the machine: no network, no external scripts or fonts.
//!
//! Usage: uno-report [--data DIR] [--out FILE] [--manual FILE] [--open]

use pulldown_cmark::{html, Options, Parser};
use serde_json::Value;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Default)]
struct Project {
    name: String,
    events: usize,
    first: String,
    last: String,
    approvals: usize,
    approvals_local: usize,
    smoke: usize,
    asked_owner: usize,
    edits_normal: usize,
    edits_review: usize,
    blocked: BTreeMap<String, usize>,
    agents: BTreeMap<String, usize>,
    per_day: BTreeMap<String, usize>,
    tasks: Vec<Task>,
}

struct Task {
    id: String,
    title: String,
    status: String,
}

impl Project {
    fn count_status(&self, wanted: &[&str]) -> usize {
        self.tasks.iter().filter(|t| wanted.contains(&t.status.as_str())).count()
    }
    fn blocked_total(&self) -> usize {
        self.blocked.values().sum()
    }
    fn reviews(&self) -> usize {
        *self.agents.get("Epsilon").unwrap_or(&0)
    }
}

const DONE: &[&str] = &["done", "merged"];
const DROPPED: &[&str] = &["dropped", "won't", "wont"];

fn agent_name(raw: &str) -> String {
    let id = raw.rsplit(':').next().unwrap_or(raw);
    let greek = match id {
        "intake-triager" => "Alpha",
        "codebase-explorer" => "Beta",
        "ui-designer" => "Gamma",
        "test-writer" => "Delta",
        "security-reviewer" => "Epsilon",
        "release-manager" => "Zeta",
        "incident-scribe" => "Eta",
        "doc-writer" => "Theta",
        "devops-engineer" => "Iota",
        "data-engineer" => "Kappa",
        "architect" => "Lambda",
        _ => return id.to_string(),
    };
    greek.to_string()
}

fn read_audit(p: &mut Project, path: &Path) {
    let Ok(text) = fs::read_to_string(path) else { return };
    for line in text.lines() {
        let Ok(v) = serde_json::from_str::<Value>(line) else { continue };
        let s = |k: &str| v.get(k).and_then(Value::as_str).unwrap_or("").to_string();
        p.events += 1;
        let time = s("time");
        if !time.is_empty() {
            if p.first.is_empty() || time < p.first {
                p.first = time.clone();
            }
            if time > p.last {
                p.last = time.clone();
            }
            *p.per_day.entry(time.chars().take(10).collect()).or_default() += 1;
        }
        match (s("event").as_str(), s("action").as_str()) {
            ("plan-approved", _) => {
                p.approvals += 1;
                let backend = s("backend");
                if !backend.is_empty() && backend != "anthropic" {
                    p.approvals_local += 1;
                }
            }
            ("smoke", _) => p.smoke += 1,
            (_, "blocked") => {
                let rule = [s("rule"), s("zone"), s("tool")].into_iter().find(|x| !x.is_empty()).unwrap_or_default();
                *p.blocked.entry(rule).or_default() += 1;
            }
            (_, "asked-owner") => p.asked_owner += 1,
            (_, "allowed") => match s("zone").as_str() {
                "review" => p.edits_review += 1,
                _ => p.edits_normal += 1,
            },
            (_, "called") if matches!(s("tool").as_str(), "Agent" | "Task") => {
                *p.agents.entry(agent_name(&s("name"))).or_default() += 1;
            }
            _ => {}
        }
    }
}

fn cells(line: &str) -> Vec<String> {
    let t = line.trim().trim_start_matches('|').trim_end_matches('|');
    t.split('|').map(|c| c.trim().to_string()).collect()
}

fn read_tasks(p: &mut Project, path: &Path) {
    let Ok(text) = fs::read_to_string(path) else { return };
    let (mut id_col, mut task_col, mut status_col) = (None, None, None);
    for line in text.lines() {
        let l = line.trim();
        if !l.starts_with('|') {
            continue;
        }
        let c = cells(l);
        if c.first().map(|x| x == "ID").unwrap_or(false) {
            id_col = Some(0);
            task_col = c.iter().position(|x| x == "Task");
            status_col = c.iter().position(|x| x == "Status");
            continue;
        }
        let (Some(i), Some(s)) = (id_col, status_col) else { continue };
        let id = c.get(i).cloned().unwrap_or_default();
        if !id.starts_with('T') || !id[1..].chars().all(|ch| ch.is_ascii_digit()) || id.len() < 2 {
            continue;
        }
        if p.tasks.iter().any(|t| t.id == id) {
            continue;
        }
        let status = c.get(s).map(|x| x.to_lowercase()).unwrap_or_default();
        let status = status.trim_matches(|ch: char| ch == '*' || ch == '`').split_whitespace().next().unwrap_or("").to_string();
        let title = task_col.and_then(|t| c.get(t)).cloned().unwrap_or_default();
        p.tasks.push(Task { id, title, status });
    }
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// Bilingual text: both languages are in the page, CSS shows the one in `html[data-lang]`.
fn t(en: &str, th: &str) -> String {
    format!("<span class=\"l-en\">{}</span><span class=\"l-th\">{}</span>", esc(en), esc(th))
}

fn short(s: &str, n: usize) -> String {
    let plain: String = s.replace('`', "").replace("**", "");
    if plain.chars().count() <= n {
        plain
    } else {
        format!("{}…", plain.chars().take(n).collect::<String>())
    }
}

struct Bar {
    en: String,
    th: String,
    value: usize,
}

fn bar(label: &str, value: usize) -> Bar {
    Bar { en: label.to_string(), th: label.to_string(), value }
}

/// Horizontal bar chart; each label is drawn in both languages.
fn hbars(ic: &str, title: (&str, &str), items: &[Bar], color: &str) -> String {
    let cap = format!("<figcaption>{}{}</figcaption>", icon(ic), t(title.0, title.1));
    if items.is_empty() {
        return format!("<figure class=\"chart\">{cap}<p class=\"muted\">{}</p></figure>", t("No data", "ไม่มีข้อมูล"));
    }
    let max = items.iter().map(|x| x.value).max().unwrap_or(1).max(1) as f64;
    let (label_w, bar_w, row) = (215.0, 300.0, 26.0);
    let h = items.len() as f64 * row + 8.0;
    let mut svg = format!("<svg viewBox=\"0 0 560 {h}\" role=\"img\" aria-label=\"{}\">", esc(title.0));
    for (i, b) in items.iter().enumerate() {
        let y = i as f64 * row + 4.0;
        let w = (b.value as f64 / max * bar_w).max(2.0);
        let (lx, ty) = (label_w - 8.0, y + 13.5);
        svg += &format!(
            "<text x=\"{lx}\" y=\"{ty}\" text-anchor=\"end\" class=\"lbl l-en\">{}</text>\
             <text x=\"{lx}\" y=\"{ty}\" text-anchor=\"end\" class=\"lbl l-th\">{}</text>\
             <rect x=\"{label_w}\" y=\"{y}\" width=\"{w:.1}\" height=\"18\" rx=\"3\" fill=\"{color}\"/>\
             <text x=\"{vx:.1}\" y=\"{ty}\" class=\"val\">{v}</text>",
            esc(&b.en), esc(&b.th), vx = label_w + w + 6.0, v = b.value
        );
    }
    svg += "</svg>";
    format!("<figure class=\"chart\">{cap}{svg}</figure>")
}

/// Vertical bars over days.
fn day_bars(ic: &str, title: (&str, &str), days: &BTreeMap<String, usize>) -> String {
    if days.is_empty() {
        return String::new();
    }
    let max = *days.values().max().unwrap_or(&1) as f64;
    let n = days.len() as f64;
    let (left, width, height) = (30.0, 940.0, 150.0);
    let step = width / n;
    let bw = (step * 0.6).min(48.0);
    let mut svg = format!("<svg viewBox=\"0 0 1000 {}\" role=\"img\" aria-label=\"{}\">", height + 30.0, esc(title.0));
    svg += &format!("<line x1=\"{left}\" y1=\"{height}\" x2=\"{}\" y2=\"{height}\" class=\"axis\"/>", left + width);
    for (i, (day, v)) in days.iter().enumerate() {
        let x = left + i as f64 * step + (step - bw) / 2.0;
        let h = (*v as f64 / max * (height - 16.0)).max(2.0);
        svg += &format!(
            "<rect x=\"{x:.1}\" y=\"{y:.1}\" width=\"{bw:.1}\" height=\"{h:.1}\" rx=\"3\" class=\"bar\"/>\
             <text x=\"{cx:.1}\" y=\"{vy:.1}\" text-anchor=\"middle\" class=\"val\">{v}</text>\
             <text x=\"{cx:.1}\" y=\"{dy}\" text-anchor=\"middle\" class=\"lbl\">{d}</text>",
            y = height - h, cx = x + bw / 2.0, vy = height - h - 4.0, dy = height + 16.0, d = &day[5..]
        );
    }
    svg += "</svg>";
    format!("<figure class=\"chart wide\"><figcaption>{}{}</figcaption>{svg}</figure>", icon(ic), t(title.0, title.1))
}

fn tile(ic: &str, label: (&str, &str), value: usize, note: (String, String)) -> String {
    format!("<div class=\"tile\"><div class=\"top\"><div class=\"num\">{value}</div><span class=\"badge\">{}</span></div><div class=\"lbl\">{}</div><div class=\"note\">{}</div></div>",
            icon(ic), t(label.0, label.1), t(&note.0, &note.1))
}

/// UNO mark: a loop "U" with an arrow. The tile takes the text colour and the glyph the
/// background colour, so it is black on the light theme and white on the dark theme.
fn logo(size: u32) -> String {
    format!("<svg class=\"logo\" width=\"{size}\" height=\"{size}\" viewBox=\"0 0 64 64\" aria-hidden=\"true\">\
             <rect width=\"64\" height=\"64\" rx=\"14\" fill=\"var(--fg)\"/>\
             <path d=\"M18.5 18V33.5A11 11 0 0 0 40.5 33.5V20M35 25.5L40.5 20L46 25.5\" fill=\"none\" stroke=\"var(--bg)\" stroke-width=\"5\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/></svg>")
}

/// Small line icons (Lucide-style paths, drawn inline; no external files).
fn icon(name: &str) -> String {
    let body = match name {
        "folder" => "<path d='M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z'/>",
        "check" => "<circle cx='12' cy='12' r='9'/><path d='m8 12 3 3 5-6'/>",
        "clock" => "<circle cx='12' cy='12' r='9'/><path d='M12 7v5l3 2'/>",
        "approve" => "<path d='M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z'/><path d='M14 3v5h5'/><path d='m9 14 2 2 4-4'/>",
        "shield" => "<path d='M12 3 5 6v6c0 4.5 3 7.5 7 9 4-1.5 7-4.5 7-9V6z'/><path d='m9 12 2 2 4-4'/>",
        "ban" => "<circle cx='12' cy='12' r='9'/><path d='m6 6 12 12'/>",
        "bars" => "<path d='M4 20V10M10 20V4M16 20v-7M22 20H2'/>",
        "users" => "<circle cx='9' cy='8' r='3.5'/><path d='M2.5 20a6.5 6.5 0 0 1 13 0'/><path d='M16 4.5a3.5 3.5 0 0 1 0 7M18 14a6 6 0 0 1 3.5 6'/>",
        "activity" => "<path d='M3 12h4l3 7 4-14 3 7h4'/>",
        "list" => "<path d='M9 6h11M9 12h11M9 18h11'/><circle cx='4.5' cy='6' r='1'/><circle cx='4.5' cy='12' r='1'/><circle cx='4.5' cy='18' r='1'/>",
        "table" => "<rect x='3' y='4' width='18' height='16' rx='2'/><path d='M3 10h18M9 10v10'/>",
        "grid" => "<rect x='3' y='3' width='7' height='7' rx='1.5'/><rect x='14' y='3' width='7' height='7' rx='1.5'/><rect x='3' y='14' width='7' height='7' rx='1.5'/><rect x='14' y='14' width='7' height='7' rx='1.5'/>",
        "book" => "<path d='M4 5a2 2 0 0 1 2-2h13v16H6a2 2 0 0 0-2 2z'/><path d='M4 19V5'/>",
        "back" => "<path d='M19 12H5M11 6l-6 6 6 6'/>",
        "sun" => "<circle cx='12' cy='12' r='4'/><path d='M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4'/>",
        "moon" => "<path d='M20 14.5A8 8 0 1 1 9.5 4a6.5 6.5 0 0 0 10.5 10.5z'/>",
        "auto" => "<circle cx='12' cy='12' r='9'/><path d='M12 3a9 9 0 0 1 0 18z' fill='currentColor'/>",
        _ => "",
    };
    format!("<svg class='ic' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1.8' stroke-linecap='round' stroke-linejoin='round' aria-hidden='true'>{body}</svg>")
}



/// Task status code → (English, Thai).
fn status_names(code: &str) -> (String, String) {
    let (en, th) = match code {
        "done" | "merged" => ("Completed", "เสร็จสิ้น"),
        "dropped" | "won't" | "wont" => ("Cancelled", "ยกเลิก"),
        "proposed" => ("Proposed", "เสนอแล้ว"),
        "ready" => ("Ready", "พร้อมดำเนินการ"),
        "in-progress" => ("In progress", "กำลังดำเนินการ"),
        "in-review" => ("In review", "รอตรวจทาน"),
        "blocked" => ("Blocked", "ติดขัด"),
        "deferred" => ("Deferred", "เลื่อนออกไป"),
        "" | "?" => ("Unspecified", "ไม่ระบุ"),
        other => (other, other),
    };
    (en.to_string(), th.to_string())
}

/// Audit rule code → (English, Thai).
fn rule_names(code: &str) -> (String, String) {
    let (en, th) = match code {
        "git-guard" | "git" => ("Git rules (guard_git)", "กฎ Git (guard_git)"),
        "plan-gate" => ("Plan not approved (guard_plan)", "ยังไม่อนุมัติแผน (guard_plan)"),
        "self-approval" => ("Self-approval attempt", "อนุมัติแผนด้วยตนเอง"),
        "secrets" => ("Secret file access (guard_secrets)", "อ่านไฟล์ข้อมูลลับ (guard_secrets)"),
        "infra" => ("Infrastructure (guard_infra)", "ระบบโครงสร้างพื้นฐาน (guard_infra)"),
        "browser" => ("Out-of-scope site (guard_browser)", "เว็บนอกขอบเขต (guard_browser)"),
        "forbidden" => ("Forbidden zone (guard_zones)", "เขตห้ามแก้ไข (guard_zones)"),
        other => (other, other),
    };
    (en.to_string(), th.to_string())
}

fn sorted(m: BTreeMap<String, usize>) -> Vec<(String, usize)> {
    let mut v: Vec<_> = m.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1));
    v
}

fn status_map(tasks: &[&Task]) -> BTreeMap<String, usize> {
    let mut m = BTreeMap::new();
    for t in tasks {
        *m.entry(if t.status.is_empty() { "?".into() } else { t.status.clone() }).or_default() += 1;
    }
    m
}

fn is_open(t: &Task) -> bool {
    !DONE.contains(&t.status.as_str()) && !DROPPED.contains(&t.status.as_str())
}

fn task_table(tasks: &[&Task]) -> String {
    let mut out = format!("<table><thead><tr><th>{}</th><th>{}</th><th>{}</th></tr></thead><tbody>",
                          t("Task ID", "รหัส"), t("Status", "สถานะ"), t("Description", "รายละเอียดงาน"));
    if tasks.is_empty() {
        out += &format!("<tr><td colspan=\"3\" class=\"muted\">{}</td></tr>", t("No items", "ไม่มีรายการ"));
    }
    for task in tasks {
        let (en, th) = status_names(&task.status);
        out += &format!("<tr><td>{}</td><td><span class=\"pill s-{}\">{}</span></td><td>{}</td></tr>",
                        esc(&task.id), esc(&task.status), t(&en, &th), esc(&short(&task.title, 160)));
    }
    out + "</tbody></table>"
}

fn tiles_for(projects: &[&Project], show_count: bool) -> String {
    let sum = |f: &dyn Fn(&Project) -> usize| projects.iter().map(|p| f(p)).sum::<usize>();
    let tasks = sum(&|p| p.tasks.len());
    let done = sum(&|p| p.count_status(DONE));
    let dropped = sum(&|p| p.count_status(DROPPED));
    let local = sum(&|p| p.approvals_local);
    let asked = sum(&|p| p.asked_owner);
    let mut out = String::from("<section class=\"tiles\">");
    if show_count {
        out += &tile("folder", ("Projects", "จำนวนโครงการ"), projects.len(),
                     ("Projects managed by UNO".into(), "โครงการที่อยู่ภายใต้ UNO".into()));
    }
    out += &tile("check", ("Completed tasks", "งานที่เสร็จสิ้น"), done,
                 (format!("Of {tasks} in total · {dropped} cancelled"), format!("จากทั้งหมด {tasks} งาน · ยกเลิก {dropped} งาน")));
    out += &tile("clock", ("Tasks in progress", "งานที่อยู่ระหว่างดำเนินการ"), tasks - done - dropped,
                 ("Excludes completed and cancelled".into(), "ไม่รวมงานที่เสร็จสิ้นหรือยกเลิก".into()));
    out += &tile("approve", ("Approved plans", "แผนงานที่ได้รับอนุมัติ"), sum(&|p| p.approvals),
                 (format!("{local} approved with a local model"), format!("ดำเนินการด้วยโมเดลภายใน {local} ครั้ง")));
    out += &tile("shield", ("Security reviews", "การตรวจสอบความปลอดภัย"), sum(&|p| p.reviews()),
                 ("Performed by Epsilon".into(), "ดำเนินการโดย Epsilon".into()));
    out += &tile("ban", ("Blocked actions", "การดำเนินการที่ถูกระงับ"), sum(&|p| p.blocked_total()),
                 (format!("{asked} owner confirmations requested"), format!("ขอการยืนยันจากผู้ดูแล {asked} ครั้ง")));
    out + "</section>"
}

fn charts_for(projects: &[&Project]) -> String {
    let mut blocked = BTreeMap::new();
    let mut agents = BTreeMap::new();
    let mut days = BTreeMap::new();
    let mut tasks: Vec<&Task> = Vec::new();
    for p in projects {
        tasks.extend(p.tasks.iter());
        for (k, v) in &p.blocked { *blocked.entry(k.clone()).or_default() += v; }
        for (k, v) in &p.agents { *agents.entry(k.clone()).or_default() += v; }
        for (k, v) in &p.per_day { *days.entry(k.clone()).or_default() += v; }
    }
    let st: Vec<Bar> = sorted(status_map(&tasks)).into_iter()
        .map(|(k, v)| { let (en, th) = status_names(&k); Bar { en, th, value: v } }).collect();
    let bl: Vec<Bar> = sorted(blocked).into_iter()
        .map(|(k, v)| { let (en, th) = rule_names(&k); Bar { en, th, value: v } }).collect();
    let ag: Vec<Bar> = sorted(agents).into_iter().map(|(k, v)| bar(&k, v)).collect();
    let mut out = String::from("<section class=\"charts\">");
    out += &hbars("bars", ("Task status", "สถานะของงาน"), &st, "var(--c1)");
    out += &hbars("ban", ("Blocked actions by rule", "การดำเนินการที่ถูกระงับ แยกตามกฎ"), &bl, "var(--c3)");
    out += &hbars("users", ("AI team usage", "การเรียกใช้ทีม AI"), &ag, "var(--c2)");
    out += &day_bars("activity", ("Daily activity (from audit.log)", "กิจกรรมรายวัน (จากบันทึกการตรวจสอบ audit.log)"), &days);
    out + "</section>"
}

fn period(p: &Project) -> String {
    format!("{} → {}", p.first.get(..10).unwrap_or("-"), p.last.get(..10).unwrap_or("-"))
}

fn overview(projects: &[Project]) -> String {
    let all: Vec<&Project> = projects.iter().collect();
    let mut out = tiles_for(&all, true);
    out += &format!(
        "<h2>{}{} <span class=\"muted hint\">{}</span></h2><table><thead><tr><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th></tr></thead><tbody>",
        icon("table"), t("Project summary", "สรุปรายโครงการ"),
        t("Click a row for details · sorted by tasks in progress", "คลิกที่แถวเพื่อดูรายละเอียด · เรียงตามจำนวนงานที่อยู่ระหว่างดำเนินการ"),
        t("Project", "โครงการ"), t("Period", "ช่วงเวลา"), t("Total tasks", "งานทั้งหมด"), t("Completed", "เสร็จสิ้น"),
        t("In progress", "ระหว่างดำเนินการ"), t("Approved plans", "อนุมัติแผน"), t("Security reviews", "ตรวจสอบความปลอดภัย"),
        t("Blocked", "ถูกระงับ"), t("File edits (review zone)", "แก้ไขไฟล์ (เขตตรวจสอบ)"));
    let mut order: Vec<(usize, &Project)> = projects.iter().enumerate().collect();
    order.sort_by_key(|(_, p)| std::cmp::Reverse(p.tasks.iter().filter(|t| is_open(t)).count()));
    for (i, p) in order {
        let d = p.count_status(DONE);
        let dr = p.count_status(DROPPED);
        out += &format!(
            "<tr class=\"go\" data-go=\"p{i}\"><td><a href=\"#\">{}</a></td><td>{}</td><td>{}</td><td>{d}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{} ({})</td></tr>",
            esc(&p.name), period(p), p.tasks.len(), p.tasks.len() - d - dr, p.approvals, p.reviews(),
            p.blocked_total(), p.edits_normal + p.edits_review, p.edits_review);
    }
    out += "</tbody></table>";
    out + &charts_for(&all)
}

fn project_view(p: &Project) -> String {
    let mut out = format!("<h2 class=\"ptitle\">{}<span class=\"muted\"> · {}</span></h2>", esc(&p.name), period(p));
    out += &tiles_for(&[p], false);
    let open: Vec<&Task> = p.tasks.iter().filter(|t| is_open(t)).collect();
    out += &format!("<h2>{}{}</h2>", icon("list"), t("Tasks in progress", "งานที่อยู่ระหว่างดำเนินการ"));
    out += &task_table(&open);
    out += &charts_for(&[p]);
    let all: Vec<&Task> = p.tasks.iter().collect();
    out += &format!("<details><summary>{}</summary>{}</details>",
                    t(&format!("All tasks ({})", all.len()), &format!("งานทั้งหมด ({} รายการ)", all.len())), task_table(&all));
    out
}

fn dashboard(projects: &[Project]) -> String {
    // <option> cannot hold markup, so options carry both texts and the script picks one.
    let mut bar = format!(
        "<div class=\"pbar\"><button class=\"back\" data-p=\"all\">{}{}</button><select id=\"psel\" aria-label=\"Select a project\">\
         <option value=\"all\" data-en=\"All projects (overview)\" data-th=\"ภาพรวมทุกโครงการ\">All projects (overview)</option>",
        icon("back"), t("Back to overview", "กลับสู่ภาพรวม"));
    let mut views = format!("<div class=\"pview on\" id=\"all\">{}</div>", overview(projects));
    for (i, p) in projects.iter().enumerate() {
        let open = p.tasks.iter().filter(|t| is_open(t)).count();
        let (en, th) = (format!("{} · {open} in progress", p.name), format!("{} · ระหว่างดำเนินการ {open} งาน", p.name));
        bar += &format!("<option value=\"p{i}\" data-en=\"{}\" data-th=\"{}\">{}</option>", esc(&en), esc(&th), esc(&en));
        views += &format!("<div class=\"pview\" id=\"p{i}\">{}</div>", project_view(p));
    }
    bar += "</select></div>";
    bar + &views
}

fn render_md(path: &Path) -> Option<String> {
    let md = fs::read_to_string(path).ok()?;
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_TASKLISTS);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    let mut s = String::new();
    html::push_html(&mut s, Parser::new_ext(&md, opts));
    Some(s)
}

/// English manual from `manual.md`, Thai from `manual.th.md`; a missing one falls back to the other.
fn manual(en_path: &Path) -> String {
    let th_path = en_path.with_file_name("manual.th.md");
    let en = render_md(en_path);
    let th = render_md(&th_path);
    let missing = format!(
        "<p class=\"muted\">{}</p>",
        t(&format!("No manual found at {} — add manual.md there and run again.", en_path.display()),
          &format!("ไม่พบคู่มือที่ {} — วางไฟล์ manual.md ไว้แล้วรันใหม่", en_path.display())));
    let (en, th) = match (en, th) {
        (Some(e), Some(t)) => (e, t),
        (Some(e), None) => (e.clone(), e),
        (None, Some(t)) => (t.clone(), t),
        (None, None) => return missing,
    };
    format!("<div class=\"l-en\">{en}</div><div class=\"l-th\">{th}</div>")
}

const CSS: &str = r#"
:root{--bg:#fbfaf8;--fg:#1f1e1c;--muted:#6b6862;--line:#e4e1da;--card:#fff;--c1:#2f6fbf;--c2:#2e8b6e;--c3:#c2593a}
:root[data-theme=dark]{--bg:#1b1a19;--fg:#ecebe8;--muted:#a19e97;--line:#34322f;--card:#242321;--c1:#6fa3e6;--c2:#5fc0a0;--c3:#e58a6c;color-scheme:dark}#theme{background:var(--card);border:1px solid var(--line);border-radius:99px;padding:4px 12px;font:inherit;font-size:13px;color:var(--fg);cursor:pointer}
*{box-sizing:border-box}body{margin:0;background:var(--bg);color:var(--fg);font:15px/1.6 -apple-system,"Sukhumvit Set","Thonburi",system-ui,sans-serif}
header{position:sticky;top:0;background:var(--bg);border-bottom:1px solid var(--line);padding:14px 28px;display:flex;gap:24px;align-items:center;z-index:1}
header h1{font-size:18px;margin:0;display:flex;align-items:center;gap:10px;letter-spacing:.02em}header .meta{color:var(--muted);font-size:13px}#lang{margin-left:auto}
nav button{background:none;border:0;border-bottom:2px solid transparent;color:var(--muted);font:inherit;padding:6px 2px;margin-right:14px;cursor:pointer}
nav button.on{color:var(--fg);border-color:var(--c1);font-weight:600}
main{max-width:1080px;margin:0 auto;padding:24px 28px 60px}.view{display:none}.view.on{display:block}
.tiles{display:grid;grid-template-columns:repeat(auto-fit,minmax(150px,1fr));gap:12px;margin-bottom:28px}
.tile{background:var(--card);border:1px solid var(--line);border-radius:10px;padding:14px 16px}
.tile .top{display:flex;justify-content:space-between;align-items:flex-start}.tile .num{font-size:30px;font-weight:650;line-height:1.1}.badge{display:inline-flex;width:34px;height:34px;border-radius:9px;align-items:center;justify-content:center;color:var(--c1);background:color-mix(in srgb,var(--c1) 12%,var(--card))}.badge .ic{width:19px;height:19px}.ic{width:17px;height:17px;flex:none}figcaption,h2{display:flex;align-items:center;gap:8px}nav button,.pbar .back,#theme>span{display:inline-flex;align-items:center;gap:7px}figcaption .ic,h2 .ic{color:var(--c1)}#theme>span{display:none}html[data-mode=auto] #theme .m-auto,html[data-mode=light] #theme .m-light,html[data-mode=dark] #theme .m-dark,html:not([data-mode]) #theme .m-auto{display:inline-flex}.tile .lbl{font-weight:600}.tile .note{color:var(--muted);font-size:12.5px}
h2{font-size:17px;margin:30px 0 10px}table{width:100%;border-collapse:collapse;background:var(--card);border:1px solid var(--line);border-radius:10px;overflow:hidden}
th,td{text-align:left;padding:8px 12px;border-bottom:1px solid var(--line);vertical-align:top;font-size:14px}th{color:var(--muted);font-weight:600;font-size:13px}
tr:last-child td{border-bottom:0}.charts{display:grid;grid-template-columns:repeat(auto-fit,minmax(420px,1fr));gap:14px;margin-top:22px}
.chart{margin:0;background:var(--card);border:1px solid var(--line);border-radius:10px;padding:14px 16px}.chart.wide{grid-column:1/-1}
figcaption{font-weight:600;margin-bottom:8px}svg{width:100%;height:auto}svg .lbl{font-size:12px;fill:var(--muted)}svg .val{font-size:12px;fill:var(--fg)}
svg .axis{stroke:var(--line)}svg .bar{fill:var(--c1)}.pill{background:var(--line);border-radius:99px;padding:1px 9px;font-size:12.5px;white-space:nowrap}td:first-child{white-space:nowrap}
.muted{color:var(--muted)}.pbar{display:flex;gap:10px;align-items:center;margin-bottom:22px}.pbar select,.pbar .back{background:var(--card);border:1px solid var(--line);border-radius:8px;padding:6px 12px;font:inherit;color:var(--fg)}.pbar select{min-width:280px}.pbar .back{cursor:pointer;display:none}.pbar.inp .back{display:inline-block}tr.go{cursor:pointer}tr.go:hover td{background:color-mix(in srgb,var(--c1) 7%,var(--card))}.hint{font-size:13px;font-weight:400}.pview{display:none}.pview.on{display:block}.ptitle{font-size:20px;margin-top:0}details{margin-top:22px}summary{cursor:pointer;font-weight:600;margin-bottom:10px}tr.go a{color:var(--c1);text-decoration:none}.s-done,.s-merged{background:color-mix(in srgb,var(--c2) 18%,var(--card))}.s-in-progress,.s-in-review,.s-ready{background:color-mix(in srgb,var(--c1) 18%,var(--card))}.s-blocked{background:color-mix(in srgb,var(--c3) 20%,var(--card))}#manual{max-width:860px}#manual table{margin:10px 0}#manual pre{background:var(--card);border:1px solid var(--line);border-radius:8px;padding:10px 14px;overflow:auto}
#manual code{font-size:13px}#manual h1{font-size:24px}#manual h2{border-top:1px solid var(--line);padding-top:18px}
"#;

const EXTRA_CSS: &str = r#"
html[data-lang=en] .l-th,html[data-lang=th] .l-en,html:not([data-lang]) .l-th{display:none}
#lang{display:inline-flex;border:1px solid var(--line);border-radius:99px;overflow:hidden}
#lang button{background:var(--card);border:0;padding:4px 11px;font:inherit;font-size:13px;color:var(--muted);cursor:pointer}
html[data-lang=en] #lang [data-l=en],html[data-lang=th] #lang [data-l=th]{background:var(--c1);color:#fff;font-weight:600}
"#;

const HEAD_JS: &str = r#"(function(){var DAY=6,NIGHT=18,d=document.documentElement;
function pick(){var m='auto';try{m=localStorage.getItem('uno-theme')||'auto'}catch(e){}
var h=new Date().getHours();var th=m==='auto'?((h>=DAY&&h<NIGHT)?'light':'dark'):m;d.dataset.theme=th;d.dataset.mode=m;
var bg=th==='dark'?'%23FFFFFF':'%23111111',fg=th==='dark'?'%23111111':'%23FFFFFF';
var svg="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 64 64'%3E%3Crect width='64' height='64' rx='14' fill='"+bg+"'/%3E%3Cpath d='M18.5 18V33.5A11 11 0 0 0 40.5 33.5V20M35 25.5L40.5 20L46 25.5' fill='none' stroke='"+fg+"' stroke-width='5' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E";
var f=document.getElementById('fav');if(!f){f=document.createElement('link');f.id='fav';f.rel='icon';f.type='image/svg+xml';document.head.appendChild(f);}f.href=svg;}
window.unoTheme=pick;pick();setInterval(pick,600000);
var l='en';try{l=localStorage.getItem('uno-lang')||'en'}catch(e){}d.dataset.lang=l;d.lang=l;})();"#;

const JS: &str = r#"
var D=document.documentElement;
function setLang(l){D.dataset.lang=l;D.lang=l;try{localStorage.setItem('uno-lang',l)}catch(e){}
 document.querySelectorAll('option[data-en]').forEach(o=>o.textContent=o.dataset[l]);
 document.title=l==='th'?'UNO — แดชบอร์ดและคู่มือการใช้งาน':'UNO — Dashboard and Manual';}
document.querySelectorAll('#lang button').forEach(b=>b.onclick=()=>setLang(b.dataset.l));
setLang(D.dataset.lang||'en');
document.querySelectorAll('nav button').forEach(b=>b.onclick=()=>{
 document.querySelectorAll('nav button,.view').forEach(e=>e.classList.remove('on'));
 b.classList.add('on');document.getElementById(b.dataset.v).classList.add('on');
 try{localStorage.setItem('uno-view',b.dataset.v)}catch(e){}
});
document.getElementById('theme').onclick=()=>{var o=['auto','light','dark'];var m=D.dataset.mode||'auto';
 var n=o[(o.indexOf(m)+1)%3];try{localStorage.setItem('uno-theme',n)}catch(e){}window.unoTheme()};window.unoTheme();
function showP(id){if(!document.getElementById(id))id='all';
 document.querySelectorAll('.pview').forEach(e=>e.classList.remove('on'));document.getElementById(id).classList.add('on');
 document.getElementById('psel').value=id;document.querySelector('.pbar').classList.toggle('inp',id!=='all');
 window.scrollTo(0,0);try{localStorage.setItem('uno-proj',id)}catch(e){}}
document.getElementById('psel').onchange=e=>showP(e.target.value);
document.querySelector('.pbar .back').onclick=()=>showP('all');
document.querySelectorAll('tr.go').forEach(r=>r.onclick=e=>{e.preventDefault();showP(r.dataset.go)});
try{const p=localStorage.getItem('uno-proj');if(p)showP(p)}catch(e){}
try{const v=localStorage.getItem('uno-view');if(v)document.querySelector('nav button[data-v="'+v+'"]').click()}catch(e){}
"#;

fn now() -> String {
    Command::new("date").arg("+%Y-%m-%d %H:%M").output().ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let flag = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!("uno-report [--data DIR] [--out FILE] [--manual FILE] [--open]\n  default data: ~/UNO/data, out: <data>/uno-report.html,\n  manual: <data>/manual.md (English) and <data>/manual.th.md (Thai, optional)");
        return;
    }
    let home = env::var("HOME").unwrap_or_default();
    let data = PathBuf::from(flag("--data").unwrap_or_else(|| format!("{home}/UNO/data")));
    let out = PathBuf::from(flag("--out").unwrap_or_else(|| data.join("uno-report.html").display().to_string()));
    let manual_path = PathBuf::from(flag("--manual").unwrap_or_else(|| data.join("manual.md").display().to_string()));

    let mut projects = Vec::new();
    let Ok(entries) = fs::read_dir(&data) else {
        eprintln!("uno-report: cannot read {}", data.display());
        std::process::exit(1);
    };
    let mut dirs: Vec<PathBuf> = entries.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect();
    dirs.sort();
    for dir in dirs {
        let name = dir.file_name().unwrap().to_string_lossy().to_string();
        if name.starts_with('_') || name.starts_with('.') {
            continue;
        }
        let (audit, tasks) = (dir.join("audit.log"), dir.join("tasks.md"));
        if !audit.exists() && !tasks.exists() {
            continue;
        }
        let mut p = Project { name, ..Default::default() };
        read_audit(&mut p, &audit);
        read_tasks(&mut p, &tasks);
        projects.push(p);
    }

    let n = projects.len();
    let header = format!(
        "<header><h1>{}UNO</h1><nav><button class=\"on\" data-v=\"dash\">{}{}</button><button data-v=\"manual\">{}{}</button></nav>\
         <div id=\"lang\" role=\"group\" aria-label=\"Language\"><button data-l=\"en\">EN</button><button data-l=\"th\">TH</button></div>\
         <button id=\"theme\" title=\"Theme\"><span class=\"m-auto\">{}{}</span><span class=\"m-light\">{}{}</span><span class=\"m-dark\">{}{}</span></button>\
         <div class=\"meta\">{}</div></header>",
        logo(26), icon("grid"), t("Dashboard", "แดชบอร์ด"), icon("book"), t("Manual", "คู่มือการใช้งาน"),
        icon("auto"), t("Auto (by time)", "อัตโนมัติ (ตามเวลา)"), icon("sun"), t("Light", "โหมดสว่าง"), icon("moon"), t("Dark", "โหมดมืด"),
        t(&format!("Generated {} · {n} projects · Source {}", now(), data.display()),
          &format!("จัดทำเมื่อ {} · {n} โครงการ · แหล่งข้อมูล {}", now(), data.display())));
    let page = format!(
        "<!DOCTYPE html><html lang=\"en\" data-lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\
         <title>UNO — Dashboard and Manual</title><script>{HEAD_JS}</script><style>{CSS}{EXTRA_CSS}</style></head><body>{header}\
         <main><div id=\"dash\" class=\"view on\">{}</div><div id=\"manual\" class=\"view\">{}</div></main><script>{JS}</script></body></html>",
        dashboard(&projects), manual(&manual_path)
    );
    if let Err(e) = fs::write(&out, page) {
        eprintln!("uno-report: cannot write {}: {e}", out.display());
        std::process::exit(1);
    }
    println!("uno-report: {} projects → {}", projects.len(), out.display());
    if args.iter().any(|a| a == "--open") {
        let _ = Command::new("open").arg(&out).status();
    }
}
