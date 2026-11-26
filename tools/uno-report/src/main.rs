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

use pulldown_cmark::{html, Options, Parser};
use serde_json::Value;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

use domain::{clip, is_open, AuditRecord, Event, EventKind, Project, Task, DONE, DROPPED};

// ---------------------------------------------------------------- data

fn read_audit(p: &mut Project, path: &Path) {
    let Ok(text) = fs::read_to_string(path) else { return };
    for line in text.lines() {
        let Ok(v) = serde_json::from_str::<Value>(line) else { continue };
        let s = |k: &str| v.get(k).and_then(Value::as_str).unwrap_or("").to_string();
        p.apply(&AuditRecord {
            time: s("time"), event: s("event"), action: s("action"), backend: s("backend"), task: s("task"),
            rule: s("rule"), zone: s("zone"), tool: s("tool"), command: s("command"), file: s("file"),
            name: s("name"), description: s("description"),
        });
    }
    p.finish();
}

/// English and Thai text for an activity-feed entry.
fn event_text(k: &EventKind) -> (String, String) {
    match k {
        EventKind::PlanApproved { task } => (format!("Plan {task} approved"), format!("อนุมัติแผน {task}")),
        EventKind::Smoke => ("Smoke test".into(), "ทดสอบระบบ (smoke test)".into()),
        EventKind::Blocked { rule } => {
            let (en, th) = rule_names(rule);
            (format!("Blocked: {en}"), format!("ระงับ: {th}"))
        }
        EventKind::AskedOwner { command } => (format!("Asked the owner: {command}"), format!("ขอการยืนยัน: {command}")),
        EventKind::Edited { file } => (format!("Edited {file}"), format!("แก้ไขไฟล์ {file}")),
        EventKind::Agent { who, description } => (format!("{who}: {description}"), format!("{who}: {description}")),
        EventKind::Skill { name } => (format!("Skill {name}"), format!("เรียกใช้ skill {name}")),
    }
}

fn ev_t(e: &Event) -> String {
    let (en, th) = event_text(&e.kind);
    t(&en, &th)
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
        if !id.starts_with('T') || id.len() < 2 || !id[1..].chars().all(|ch| ch.is_ascii_digit()) {
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

/// Claude Code sessions open right now, by project folder name (macOS `lsof` + `ps`).
/// Counts only top-level `claude` processes: a process whose parent is also `claude`
/// (a worker or tool process of the same session) is not a separate session.
/// Returns None when `lsof` is not available.
fn open_sessions() -> Option<BTreeMap<String, usize>> {
    if let Ok(v) = env::var("UNO_REPORT_FAKE_SESSIONS") {
        return fake_sessions(&v);
    }
    let out = Command::new("lsof").args(["-a", "-c", "claude", "-d", "cwd", "-Fpn"]).output().ok()?;
    // pid → project
    let mut by_pid: Vec<(String, String)> = Vec::new();
    let mut pid = String::new();
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        if let Some(p) = line.strip_prefix('p') {
            pid = p.to_string();
        } else if let Some(path) = line.strip_prefix('n') {
            if let Some(rest) = path.split("/UNO/projects/").nth(1) {
                let name = rest.split('/').next().unwrap_or("").to_string();
                if !name.is_empty() {
                    by_pid.push((pid.clone(), name));
                }
            }
        }
    }
    // pid → parent's command name
    let ps = Command::new("ps").args(["-axo", "pid=,ppid=,comm="]).output().ok();
    let mut comm: BTreeMap<String, String> = BTreeMap::new();
    let mut parent: BTreeMap<String, String> = BTreeMap::new();
    if let Some(ps) = ps {
        for line in String::from_utf8_lossy(&ps.stdout).lines() {
            let mut it = line.split_whitespace();
            if let (Some(p), Some(pp)) = (it.next(), it.next()) {
                let c: String = it.collect::<Vec<_>>().join(" ");
                comm.insert(p.to_string(), c.rsplit('/').next().unwrap_or("").to_string());
                parent.insert(p.to_string(), pp.to_string());
            }
        }
    }
    let mut m = BTreeMap::new();
    for (p, name) in by_pid {
        let child_of_claude = parent.get(&p).and_then(|pp| comm.get(pp)).map(|c| c.starts_with("claude")).unwrap_or(false);
        if !child_of_claude {
            *m.entry(name).or_default() += 1;
        }
    }
    Some(m)
}

/// Test seam: `UNO_REPORT_FAKE_SESSIONS="alpha=1,beta=0"` → fixed counts, `"none"` → lsof unavailable.
fn fake_sessions(v: &str) -> Option<BTreeMap<String, usize>> {
    if v == "none" {
        return None;
    }
    Some(v.split(',').filter_map(|kv| kv.split_once('=')).filter_map(|(k, n)| Some((k.to_string(), n.parse().ok()?))).collect())
}

// ---------------------------------------------------------------- html helpers

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// Bilingual text: both languages are in the page, CSS shows the one in `html[data-lang]`.
fn t(en: &str, th: &str) -> String {
    format!("<span class=\"l-en\">{}</span><span class=\"l-th\">{}</span>", esc(en), esc(th))
}

/// Both languages inside an SVG: two <text> elements at the same spot.
fn svg_t(attrs: &str, en: &str, th: &str) -> String {
    format!("<text {attrs} class=\"l-en\">{}</text><text {attrs} class=\"l-th\">{}</text>", esc(en), esc(th))
}

fn short(s: &str, n: usize) -> String {
    clip(&s.replace('`', "").replace("**", ""), n)
}

const PALETTE: &[&str] = &["var(--c1)", "var(--c2)", "var(--c3)", "var(--c4)", "var(--c5)", "var(--c6)"];

fn color_of(i: usize) -> &'static str {
    PALETTE[i % PALETTE.len()]
}

fn figure(ic: &str, title: (&str, &str), wide: bool, body: &str) -> String {
    format!("<figure class=\"chart{}\"><figcaption>{}{}</figcaption>{body}</figure>",
            if wide { " wide" } else { "" }, icon(ic), t(title.0, title.1))
}

fn no_data() -> String {
    format!("<p class=\"muted\">{}</p>", t("No data", "ไม่มีข้อมูล"))
}

fn legend(items: &[(String, String, &str)]) -> String {
    let mut s = String::from("<div class=\"legend\">");
    for (en, th, c) in items {
        s += &format!("<span><i style=\"background:{c}\"></i>{}</span>", t(en, th));
    }
    s + "</div>"
}

struct Bar {
    en: String,
    th: String,
    value: usize,
}

/// Horizontal bars, one colour.
fn hbars(ic: &str, title: (&str, &str), items: &[Bar], color: &str) -> String {
    if items.is_empty() {
        return figure(ic, title, false, &no_data());
    }
    let max = items.iter().map(|x| x.value).max().unwrap_or(1).max(1) as f64;
    let (label_w, bar_w, row) = (215.0, 300.0, 26.0);
    let h = items.len() as f64 * row + 8.0;
    let mut svg = format!("<svg viewBox=\"0 0 560 {h}\" role=\"img\" aria-label=\"{}\">", esc(title.0));
    for (i, b) in items.iter().enumerate() {
        let y = i as f64 * row + 4.0;
        let w = (b.value as f64 / max * bar_w).max(2.0);
        svg += &svg_t(&format!("x=\"{}\" y=\"{}\" text-anchor=\"end\" fill=\"var(--muted)\" font-size=\"12\"", label_w - 8.0, y + 13.5), &b.en, &b.th);
        svg += &format!("<rect x=\"{label_w}\" y=\"{y}\" width=\"{w:.1}\" height=\"18\" rx=\"3\" fill=\"{color}\"/>\
                         <text x=\"{:.1}\" y=\"{}\" class=\"val\">{}</text>", label_w + w + 6.0, y + 13.5, b.value);
    }
    figure(ic, title, false, &(svg + "</svg>"))
}

/// Donut with a legend; the total in the middle.
fn donut(ic: &str, title: (&str, &str), items: &[(String, String, usize, &'static str)]) -> String {
    let total: usize = items.iter().map(|x| x.2).sum();
    if total == 0 {
        return figure(ic, title, false, &no_data());
    }
    let (cx, cy, r, w) = (90.0, 90.0, 66.0, 26.0);
    let circ = 2.0 * std::f64::consts::PI * r;
    let mut svg = format!("<svg viewBox=\"0 0 180 180\" class=\"donut\" role=\"img\" aria-label=\"{}\">", esc(title.0));
    svg += &format!("<circle cx=\"{cx}\" cy=\"{cy}\" r=\"{r}\" fill=\"none\" stroke=\"var(--line)\" stroke-width=\"{w}\"/>");
    let mut offset = 0.0;
    let mut leg = Vec::new();
    for (en, th, v, c) in items.iter() {
        let c = *c;
        let len = *v as f64 / total as f64 * circ;
        svg += &format!("<circle cx=\"{cx}\" cy=\"{cy}\" r=\"{r}\" fill=\"none\" stroke=\"{c}\" stroke-width=\"{w}\" \
                          stroke-dasharray=\"{len:.2} {:.2}\" stroke-dashoffset=\"{:.2}\" transform=\"rotate(-90 {cx} {cy})\"/>",
                        circ - len, -offset);
        offset += len;
        leg.push((format!("{en} · {v} ({:.0}%)", *v as f64 * 100.0 / total as f64),
                  format!("{th} · {v} ({:.0}%)", *v as f64 * 100.0 / total as f64), c));
    }
    svg += &format!("<text x=\"{cx}\" y=\"{}\" text-anchor=\"middle\" font-size=\"28\" font-weight=\"650\" fill=\"var(--fg)\">{total}</text>", cy + 4.0);
    svg += &svg_t(&format!("x=\"{cx}\" y=\"{}\" text-anchor=\"middle\" font-size=\"11\" fill=\"var(--muted)\"", cy + 22.0), "tasks", "งาน");
    svg += "</svg>";
    figure(ic, title, false, &format!("<div class=\"donutwrap\">{svg}{}</div>", legend(&leg)))
}

/// One stacked horizontal bar per project: completed / in progress / cancelled.
fn stacked_projects(projects: &[&Project]) -> String {
    let title = ("Tasks by project", "งานแยกตามโครงการ");
    if projects.iter().all(|p| p.tasks.is_empty()) {
        return figure("bars", title, false, &no_data());
    }
    let max = projects.iter().map(|p| p.tasks.len()).max().unwrap_or(1).max(1) as f64;
    let (label_w, bar_w, row) = (150.0, 360.0, 30.0);
    let h = projects.len() as f64 * row + 8.0;
    let mut svg = format!("<svg viewBox=\"0 0 560 {h}\" role=\"img\" aria-label=\"{}\">", esc(title.0));
    for (i, p) in projects.iter().enumerate() {
        let y = i as f64 * row + 4.0;
        svg += &format!("<text x=\"{}\" y=\"{}\" text-anchor=\"end\" fill=\"var(--muted)\" font-size=\"12\">{}</text>",
                        label_w - 8.0, y + 15.0, esc(&clip(&p.name, 22)));
        let parts = [(p.count_status(DONE), "var(--c2)"), (p.open(), "var(--c1)"), (p.count_status(DROPPED), "var(--muted)")];
        let mut x = label_w;
        for (v, c) in parts {
            let w = v as f64 / max * bar_w;
            if w > 0.0 {
                svg += &format!("<rect x=\"{x:.1}\" y=\"{y}\" width=\"{w:.1}\" height=\"20\" fill=\"{c}\"/>");
                if w > 18.0 {
                    svg += &format!("<text x=\"{:.1}\" y=\"{}\" text-anchor=\"middle\" font-size=\"11\" fill=\"#fff\">{v}</text>", x + w / 2.0, y + 14.0);
                }
            }
            x += w;
        }
    }
    svg += "</svg>";
    let leg = legend(&[("Completed".into(), "เสร็จสิ้น".into(), "var(--c2)"),
                       ("In progress".into(), "ระหว่างดำเนินการ".into(), "var(--c1)"),
                       ("Cancelled".into(), "ยกเลิก".into(), "var(--muted)")]);
    figure("bars", title, false, &(svg + &leg))
}

/// Columns per day, stacked by project.
fn daily_by_project(projects: &[&Project]) -> String {
    let title = ("Daily activity by project (audit.log events)", "กิจกรรมรายวันแยกตามโครงการ (เหตุการณ์ใน audit.log)");
    let mut days: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (i, p) in projects.iter().enumerate() {
        for (d, v) in &p.per_day {
            let e = days.entry(d.clone()).or_insert_with(|| vec![0; projects.len()]);
            e[i] += v;
        }
    }
    if days.is_empty() {
        return figure("activity", title, true, &no_data());
    }
    let max = days.values().map(|v| v.iter().sum::<usize>()).max().unwrap_or(1).max(1) as f64;
    let (left, width, height) = (30.0, 940.0, 160.0);
    let step = width / days.len() as f64;
    let bw = (step * 0.6).min(48.0);
    let mut svg = format!("<svg viewBox=\"0 0 1000 {}\" role=\"img\" aria-label=\"{}\">", height + 30.0, esc(title.0));
    svg += &format!("<line x1=\"{left}\" y1=\"{height}\" x2=\"{}\" y2=\"{height}\" class=\"axis\"/>", left + width);
    for (i, (day, vals)) in days.iter().enumerate() {
        let x = left + i as f64 * step + (step - bw) / 2.0;
        let mut y = height;
        for (j, v) in vals.iter().enumerate() {
            let h = *v as f64 / max * (height - 18.0);
            if h > 0.0 {
                y -= h;
                svg += &format!("<rect x=\"{x:.1}\" y=\"{y:.1}\" width=\"{bw:.1}\" height=\"{h:.1}\" fill=\"{}\"/>", color_of(j));
            }
        }
        let total: usize = vals.iter().sum();
        svg += &format!("<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" class=\"val\">{total}</text>\
                         <text x=\"{:.1}\" y=\"{}\" text-anchor=\"middle\" class=\"lbl\">{}</text>",
                        x + bw / 2.0, y - 4.0, x + bw / 2.0, height + 16.0, &day[5..]);
    }
    svg += "</svg>";
    let leg: Vec<(String, String, &str)> = projects.iter().enumerate().map(|(i, p)| (p.name.clone(), p.name.clone(), color_of(i))).collect();
    figure("activity", title, true, &(svg + &legend(&leg)))
}

/// Cumulative approved plans over time, all projects together.
fn approvals_line(projects: &[&Project]) -> String {
    let title = ("Approved plans over time (cumulative)", "แผนงานที่ได้รับอนุมัติสะสม");
    let mut days: BTreeMap<String, usize> = BTreeMap::new();
    for p in projects {
        for (d, v) in &p.approvals_by_day {
            *days.entry(d.clone()).or_default() += v;
        }
    }
    if days.is_empty() {
        return figure("approve", title, false, &no_data());
    }
    let (left, width, top, height) = (36.0, 490.0, 26.0, 140.0);
    let mut acc = 0;
    let pts: Vec<(String, usize)> = days.into_iter().map(|(d, v)| { acc += v; (d, acc) }).collect();
    let max = acc.max(1) as f64;
    let n = pts.len();
    let xs = |i: usize| if n == 1 { left + width / 2.0 } else { left + i as f64 * width / (n - 1) as f64 };
    let ys = |v: usize| top + height - v as f64 / max * height;
    let mut svg = format!("<svg viewBox=\"0 0 560 {}\" role=\"img\" aria-label=\"{}\">", top + height + 28.0, esc(title.0));
    svg += &format!("<line x1=\"{left}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" class=\"axis\"/>", top + height, left + width, top + height);
    let path: Vec<String> = pts.iter().enumerate().map(|(i, (_, v))| format!("{:.1},{:.1}", xs(i), ys(*v))).collect();
    let area = format!("{:.1},{} {} {:.1},{}", xs(0), top + height, path.join(" "), xs(n - 1), top + height);
    svg += &format!("<polygon points=\"{area}\" fill=\"color-mix(in srgb,var(--c1) 14%,transparent)\"/>\
                     <polyline points=\"{}\" fill=\"none\" stroke=\"var(--c1)\" stroke-width=\"2.2\" stroke-linejoin=\"round\"/>", path.join(" "));
    for (i, (d, v)) in pts.iter().enumerate() {
        svg += &format!("<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"3.5\" fill=\"var(--c1)\"/>\
                         <text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" class=\"val\">{v}</text>\
                         <text x=\"{:.1}\" y=\"{}\" text-anchor=\"middle\" class=\"lbl\">{}</text>",
                        xs(i), ys(*v), xs(i), ys(*v) - 8.0, xs(i), top + height + 18.0, &d[5..]);
    }
    figure("approve", title, false, &(svg + "</svg>"))
}

/// Guard rule × project table, shaded by count.
fn heatmap(projects: &[&Project]) -> String {
    let title = ("Blocked actions: rule × project", "การดำเนินการที่ถูกระงับ: กฎ × โครงการ");
    let mut rules: BTreeMap<String, usize> = BTreeMap::new();
    for p in projects {
        for (k, v) in &p.blocked {
            *rules.entry(k.clone()).or_default() += v;
        }
    }
    if rules.is_empty() {
        return figure("ban", title, false, &no_data());
    }
    let max = projects.iter().flat_map(|p| p.blocked.values()).max().copied().unwrap_or(1).max(1) as f64;
    let mut h = String::from("<table class=\"heat\"><thead><tr><th></th>");
    for p in projects {
        h += &format!("<th>{}</th>", esc(&clip(&p.name, 18)));
    }
    h += "</tr></thead><tbody>";
    for (rule, _) in sorted(rules) {
        let (en, th) = rule_names(&rule);
        h += &format!("<tr><td>{}</td>", t(&en, &th));
        for p in projects {
            let v = *p.blocked.get(&rule).unwrap_or(&0);
            let pct = (v as f64 / max * 70.0).round();
            h += &format!("<td style=\"background:color-mix(in srgb,var(--c3) {pct}%,var(--card))\">{}</td>",
                          if v == 0 { "·".to_string() } else { v.to_string() });
        }
        h += "</tr>";
    }
    figure("ban", title, false, &(h + "</tbody></table>"))
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
        "pie" => "<path d='M21 12A9 9 0 1 1 12 3v9z'/><path d='M15 3.5A9 9 0 0 1 20.5 9H15z'/>",
        "users" => "<circle cx='9' cy='8' r='3.5'/><path d='M2.5 20a6.5 6.5 0 0 1 13 0'/><path d='M16 4.5a3.5 3.5 0 0 1 0 7M18 14a6 6 0 0 1 3.5 6'/>",
        "activity" => "<path d='M3 12h4l3 7 4-14 3 7h4'/>",
        "live" => "<circle cx='12' cy='12' r='3'/><path d='M6.3 6.3a8 8 0 0 0 0 11.4M17.7 6.3a8 8 0 0 1 0 11.4'/>",
        "list" => "<path d='M9 6h11M9 12h11M9 18h11'/><circle cx='4.5' cy='6' r='1'/><circle cx='4.5' cy='12' r='1'/><circle cx='4.5' cy='18' r='1'/>",
        "table" => "<rect x='3' y='4' width='18' height='16' rx='2'/><path d='M3 10h18M9 10v10'/>",
        "grid" => "<rect x='3' y='3' width='7' height='7' rx='1.5'/><rect x='14' y='3' width='7' height='7' rx='1.5'/><rect x='3' y='14' width='7' height='7' rx='1.5'/><rect x='14' y='14' width='7' height='7' rx='1.5'/>",
        "book" => "<path d='M4 5a2 2 0 0 1 2-2h13v16H6a2 2 0 0 0-2 2z'/><path d='M4 19V5'/>",
        "back" => "<path d='M19 12H5M11 6l-6 6 6 6'/>",
        "gear" => "<circle cx='12' cy='12' r='3'/><path d='M19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1.1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1.1 1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z'/>",
        "refresh" => "<path d='M20 11a8 8 0 0 0-14.8-4M4 4v4h4M4 13a8 8 0 0 0 14.8 4M20 20v-4h-4'/>",
        "x" => "<path d='M6 6l12 12M18 6 6 18'/>",
        "sun" => "<circle cx='12' cy='12' r='4'/><path d='M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4'/>",
        "moon" => "<path d='M20 14.5A8 8 0 1 1 9.5 4a6.5 6.5 0 0 0 10.5 10.5z'/>",
        "auto" => "<circle cx='12' cy='12' r='9'/><path d='M12 3a9 9 0 0 1 0 18z' fill='currentColor'/>",
        _ => "",
    };
    format!("<svg class='ic' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1.8' stroke-linecap='round' stroke-linejoin='round' aria-hidden='true'>{body}</svg>")
}

fn status_color(code: &str) -> &'static str {
    match code {
        "done" | "merged" => "var(--c2)",
        "in-progress" | "in-review" => "var(--c1)",
        "ready" => "var(--c6)",
        "proposed" => "var(--c5)",
        "blocked" => "var(--c3)",
        "deferred" => "var(--c4)",
        _ => "var(--muted)",
    }
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

fn period(p: &Project) -> String {
    format!("{} → {}", p.first.get(..10).unwrap_or("-"), p.last.get(..10).unwrap_or("-"))
}

// ---------------------------------------------------------------- sections

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

/// Live team status. The page decides Working / Idle / Offline from `data-last`, `data-sessions`
/// and the "active window" setting, so the status ages correctly between regenerations.
fn activity_panel(projects: &[&Project], sessions_known: bool) -> String {
    let mut out = format!(
        "<h2>{}{} <span class=\"muted hint\">{}</span></h2><div class=\"live\">",
        icon("live"), t("Team activity", "สถานะการทำงานของทีม"),
        t("From open Claude Code sessions and the latest audit.log events",
          "จาก session ของ Claude Code ที่เปิดอยู่ และเหตุการณ์ล่าสุดใน audit.log"));
    for p in projects {
        let last = p.recent.first();
        let sess = if sessions_known { p.sessions.to_string() } else { "?".into() };
        out += &format!(
            "<div class=\"livecard\" data-last=\"{}\" data-sessions=\"{sess}\"><div class=\"lc-top\"><span class=\"dot\"></span><b>{}</b>\
             <span class=\"st st-working\">{}</span><span class=\"st st-open\">{}</span><span class=\"st st-idle\">{}</span></div>\
             <div class=\"lc-last\">{}</div><div class=\"lc-meta\"><span class=\"ago\" data-time=\"{}\"></span> · {}</div></div>",
            esc(&p.last), esc(&p.name),
            t("Working", "กำลังทำงาน"), t("Session open, idle", "เปิด session อยู่ ยังไม่มีกิจกรรม"), t("Not running", "ไม่ได้ทำงาน"),
            last.map(|e| ev_t(e)).unwrap_or_else(|| t("No activity yet", "ยังไม่มีกิจกรรม")),
            esc(&p.last),
            t(&format!("{sess} open session(s)"), &format!("เปิดอยู่ {sess} session")));
    }
    out + "</div>"
}

fn feed(events: &[(&str, &Event)]) -> String {
    let mut out = String::from("<ul class=\"feed\">");
    if events.is_empty() {
        out += &format!("<li class=\"muted\">{}</li>", t("No activity yet", "ยังไม่มีกิจกรรม"));
    }
    for (project, e) in events {
        out += &format!("<li><span class=\"ago\" data-time=\"{}\"></span><span class=\"fp\">{}</span><span>{}</span></li>",
                        esc(&e.time), esc(project), ev_t(e));
    }
    out + "</ul>"
}

fn overview(projects: &[Project], sessions_known: bool) -> String {
    let all: Vec<&Project> = projects.iter().collect();
    let mut out = tiles_for(&all, true);
    out += &activity_panel(&all, sessions_known);

    out += &format!(
        "<h2>{}{} <span class=\"muted hint\">{}</span></h2><table><thead><tr><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th></tr></thead><tbody>",
        icon("table"), t("Project summary", "สรุปรายโครงการ"),
        t("Click a row for details · sorted by tasks in progress", "คลิกที่แถวเพื่อดูรายละเอียด · เรียงตามจำนวนงานที่อยู่ระหว่างดำเนินการ"),
        t("Project", "โครงการ"), t("Period", "ช่วงเวลา"), t("Total tasks", "งานทั้งหมด"), t("Completed", "เสร็จสิ้น"),
        t("In progress", "ระหว่างดำเนินการ"), t("Approved plans", "อนุมัติแผน"), t("Security reviews", "ตรวจสอบความปลอดภัย"),
        t("Blocked", "ถูกระงับ"), t("File edits (review zone)", "แก้ไขไฟล์ (เขตตรวจสอบ)"));
    let mut order: Vec<(usize, &Project)> = projects.iter().enumerate().collect();
    order.sort_by_key(|(_, p)| std::cmp::Reverse(p.open()));
    for (i, p) in order {
        out += &format!(
            "<tr class=\"go\" data-go=\"p{i}\"><td><a href=\"#\">{}</a></td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{} ({})</td></tr>",
            esc(&p.name), period(p), p.tasks.len(), p.count_status(DONE), p.open(), p.approvals, p.reviews(),
            p.blocked_total(), p.edits_normal + p.edits_review, p.edits_review);
    }
    out += "</tbody></table>";

    // Charts across all projects.
    let tasks: Vec<&Task> = projects.iter().flat_map(|p| p.tasks.iter()).collect();
    let status: Vec<(String, String, usize, &'static str)> = sorted(status_map(&tasks)).into_iter()
        .map(|(k, v)| { let (en, th) = status_names(&k); (en, th, v, status_color(&k)) }).collect();
    let mut agents: BTreeMap<String, usize> = BTreeMap::new();
    for p in projects {
        for (k, v) in &p.agents { *agents.entry(k.clone()).or_default() += v; }
    }
    let ag: Vec<Bar> = sorted(agents).into_iter().map(|(k, v)| Bar { en: k.clone(), th: k, value: v }).collect();
    out += &format!("<h2>{}{}</h2><section class=\"charts\">", icon("bars"), t("Charts — all projects", "กราฟรวมทุกโครงการ"));
    out += &donut("pie", ("Task status", "สถานะของงาน"), &status);
    out += &stacked_projects(&all);
    out += &daily_by_project(&all);
    out += &approvals_line(&all);
    out += &hbars("users", ("AI team usage", "การเรียกใช้ทีม AI"), &ag, "var(--c2)");
    out += &heatmap(&all);
    out += "</section>";

    let mut events: Vec<(&str, &Event)> = projects.iter().flat_map(|p| p.recent.iter().map(move |e| (p.name.as_str(), e))).collect();
    events.sort_by(|a, b| b.1.time.cmp(&a.1.time));
    events.truncate(25);
    out += &format!("<h2>{}{}</h2>", icon("activity"), t("Recent activity", "กิจกรรมล่าสุด"));
    out + &feed(&events)
}

fn project_view(p: &Project, sessions_known: bool) -> String {
    let mut out = format!("<h2 class=\"ptitle\">{}<span class=\"muted\"> · {}</span></h2>", esc(&p.name), period(p));
    out += &tiles_for(&[p], false);
    out += &activity_panel(&[p], sessions_known);
    let open: Vec<&Task> = p.tasks.iter().filter(|t| is_open(t)).collect();
    out += &format!("<h2>{}{}</h2>", icon("list"), t("Tasks in progress", "งานที่อยู่ระหว่างดำเนินการ"));
    out += &task_table(&open);

    let tasks: Vec<&Task> = p.tasks.iter().collect();
    let status: Vec<(String, String, usize, &'static str)> = sorted(status_map(&tasks)).into_iter()
        .map(|(k, v)| { let (en, th) = status_names(&k); (en, th, v, status_color(&k)) }).collect();
    let bl: Vec<Bar> = sorted(p.blocked.clone()).into_iter()
        .map(|(k, v)| { let (en, th) = rule_names(&k); Bar { en, th, value: v } }).collect();
    let ag: Vec<Bar> = sorted(p.agents.clone()).into_iter().map(|(k, v)| Bar { en: k.clone(), th: k, value: v }).collect();
    out += "<section class=\"charts\">";
    out += &donut("pie", ("Task status", "สถานะของงาน"), &status);
    out += &hbars("ban", ("Blocked actions by rule", "การดำเนินการที่ถูกระงับ แยกตามกฎ"), &bl, "var(--c3)");
    out += &hbars("users", ("AI team usage", "การเรียกใช้ทีม AI"), &ag, "var(--c2)");
    out += &approvals_line(&[p]);
    out += &daily_by_project(&[p]);
    out += "</section>";

    let events: Vec<(&str, &Event)> = p.recent.iter().take(20).map(|e| (p.name.as_str(), e)).collect();
    out += &format!("<h2>{}{}</h2>", icon("activity"), t("Recent activity", "กิจกรรมล่าสุด"));
    out += &feed(&events);

    let all: Vec<&Task> = p.tasks.iter().collect();
    out += &format!("<details><summary>{}</summary>{}</details>",
                    t(&format!("All tasks ({})", all.len()), &format!("งานทั้งหมด ({} รายการ)", all.len())), task_table(&all));
    out
}

fn dashboard(projects: &[Project], sessions_known: bool) -> String {
    // <option> cannot hold markup, so options carry both texts and the script picks one.
    let mut bar = format!(
        "<div class=\"pbar\"><button class=\"back\" data-p=\"all\">{}{}</button><select id=\"psel\" aria-label=\"Select a project\">\
         <option value=\"all\" data-en=\"All projects (overview)\" data-th=\"ภาพรวมทุกโครงการ\">All projects (overview)</option>",
        icon("back"), t("Back to overview", "กลับสู่ภาพรวม"));
    let mut views = format!("<div class=\"pview on\" id=\"all\">{}</div>", overview(projects, sessions_known));
    for (i, p) in projects.iter().enumerate() {
        let open = p.open();
        let (en, th) = (format!("{} · {open} in progress", p.name), format!("{} · ระหว่างดำเนินการ {open} งาน", p.name));
        bar += &format!("<option value=\"p{i}\" data-en=\"{}\" data-th=\"{}\">{}</option>", esc(&en), esc(&th), esc(&en));
        views += &format!("<div class=\"pview\" id=\"p{i}\">{}</div>", project_view(p, sessions_known));
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
    let (en, th) = match (render_md(en_path), render_md(&th_path)) {
        (Some(e), Some(t)) => (e, t),
        (Some(e), None) => (e.clone(), e),
        (None, Some(t)) => (t.clone(), t),
        (None, None) => {
            return format!("<p class=\"muted\">{}</p>",
                t(&format!("No manual found at {} — add manual.md there and run again.", en_path.display()),
                  &format!("ไม่พบคู่มือที่ {} — วางไฟล์ manual.md ไว้แล้วรันใหม่", en_path.display())));
        }
    };
    format!("<div class=\"l-en\">{en}</div><div class=\"l-th\">{th}</div>")
}

fn settings_dialog(watch: Option<u64>) -> String {
    let opt = |name: &str, vals: &[(&str, &str, &str)]| -> String {
        vals.iter().map(|(v, en, th)| format!("<label class=\"opt\"><input type=\"radio\" name=\"{name}\" value=\"{v}\">{}</label>", t(en, th))).collect()
    };
    let watch_note = match watch {
        Some(m) => (format!("This file is being regenerated every {m} min by uno-report --watch."),
                    format!("ไฟล์นี้ถูกสร้างใหม่ทุก {m} นาทีโดย uno-report --watch")),
        None => ("New data needs uno-report to run again. Keep it updating with: uno-report --watch 5".to_string(),
                 "ข้อมูลใหม่จะมาเมื่อรัน uno-report อีกครั้ง ให้อัปเดตต่อเนื่องด้วยคำสั่ง uno-report --watch 5".to_string()),
    };
    format!(
        "<dialog id=\"settings\"><form method=\"dialog\"><div class=\"dh\"><h2>{}{}</h2><button class=\"iconbtn\" value=\"close\" aria-label=\"Close\">{}</button></div>\
         <fieldset><legend>{}</legend>{}</fieldset>\
         <fieldset><legend>{}</legend>{}<div class=\"hours\"><label>{}<input type=\"number\" id=\"s-day\" min=\"0\" max=\"23\"></label><label>{}<input type=\"number\" id=\"s-night\" min=\"0\" max=\"23\"></label></div></fieldset>\
         <fieldset><legend>{}</legend>{}<p class=\"muted small\">{}</p></fieldset>\
         <fieldset><legend>{}</legend>{}<p class=\"muted small\">{}</p></fieldset>\
         <fieldset><legend>{}</legend>{}</fieldset>\
         <fieldset><legend>{}</legend>{}<p class=\"muted small\">{}</p></fieldset>\
         <div class=\"df\"><button type=\"button\" id=\"s-reset\" class=\"ghost\">{}</button><button value=\"close\" class=\"primary\">{}</button></div></form></dialog>",
        icon("gear"), t("Settings", "ตั้งค่า"), icon("x"),
        t("Language", "ภาษา"), opt("s-lang", &[("en", "English", "English"), ("th", "ไทย", "ไทย")]),
        t("Theme", "รูปแบบการแสดงผล"), opt("s-theme", &[("auto", "Auto (by time)", "อัตโนมัติ (ตามเวลา)"), ("light", "Light", "โหมดสว่าง"), ("dark", "Dark", "โหมดมืด")]),
        t("Light from (hour)", "โหมดสว่างตั้งแต่ (ชั่วโมง)"), t("Dark from (hour)", "โหมดมืดตั้งแต่ (ชั่วโมง)"),
        t("Reload the page", "โหลดหน้าใหม่อัตโนมัติ"),
        opt("s-reload", &[("0", "Off", "ปิด"), ("1", "Every 1 min", "ทุก 1 นาที"), ("5", "Every 5 min", "ทุก 5 นาที"), ("15", "Every 15 min", "ทุก 15 นาที"), ("30", "Every 30 min", "ทุก 30 นาที")]),
        t(&watch_note.0, &watch_note.1),
        t("Count the team as working when the last activity is within", "ถือว่าทีมกำลังทำงาน เมื่อมีกิจกรรมล่าสุดภายใน"),
        opt("s-window", &[("5", "5 min", "5 นาที"), ("15", "15 min", "15 นาที"), ("30", "30 min", "30 นาที"), ("60", "60 min", "60 นาที")]),
        t("A session counts as open when Claude Code runs in ~/UNO/projects/<project>.", "นับว่าเปิด session เมื่อ Claude Code ทำงานอยู่ใน ~/UNO/projects/<โครงการ>"),
        t("Start page", "หน้าเริ่มต้น"),
        opt("s-start", &[("last", "Last viewed", "หน้าที่ดูล่าสุด"), ("all", "Overview", "ภาพรวม")]),
        t("Live update (uno-report --serve)", "อัปเดตสด (uno-report --serve)"),
        opt("s-live", &[("2", "Every 2 s", "ทุก 2 วินาที"), ("3", "Every 3 s", "ทุก 3 วินาที"), ("5", "Every 5 s", "ทุก 5 วินาที"), ("10", "Every 10 s", "ทุก 10 วินาที")]),
        t("Only when opened from http://localhost:8765. The page checks for changes and updates in place.",
          "ใช้ได้เมื่อเปิดจาก http://localhost:8765 หน้าเว็บจะตรวจการเปลี่ยนแปลงและอัปเดตเองโดยไม่โหลดทั้งหน้า"),
        t("Reset to defaults", "คืนค่าเริ่มต้น"), t("Done", "เสร็จสิ้น"))
}

// ---------------------------------------------------------------- page

const CSS: &str = r#"
:root{--bg:#fbfaf8;--fg:#1f1e1c;--muted:#6b6862;--line:#e4e1da;--card:#fff;--c1:#2f6fbf;--c2:#2e8b6e;--c3:#c2593a;--c4:#8a5cc2;--c5:#b8860b;--c6:#3d8fa6}
:root[data-theme=dark]{--bg:#1b1a19;--fg:#ecebe8;--muted:#a19e97;--line:#34322f;--card:#242321;--c1:#6fa3e6;--c2:#5fc0a0;--c3:#e58a6c;--c4:#b393e0;--c5:#e0b44a;--c6:#6cc0d6;color-scheme:dark}
[hidden]{display:none!important}#liveflag .ld{width:8px;height:8px;border-radius:50%;background:var(--c2);box-shadow:0 0 0 3px color-mix(in srgb,var(--c2) 25%,transparent)}#liveflag.off .ld{background:var(--c3);box-shadow:none}#liveflag .l-off{display:none}#liveflag.off .l-on{display:none}#liveflag.off .l-off{display:inline}#liveflag.flash{border-color:var(--c2)}*{box-sizing:border-box}body{margin:0;background:var(--bg);color:var(--fg);font:15px/1.6 -apple-system,"Sukhumvit Set","Thonburi",system-ui,sans-serif}
header{position:sticky;top:0;background:var(--bg);border-bottom:1px solid var(--line);padding:12px 28px;display:flex;gap:18px;align-items:center;z-index:2;flex-wrap:wrap}
header h1{font-size:18px;margin:0;display:flex;align-items:center;gap:10px;letter-spacing:.02em}header .meta{color:var(--muted);font-size:13px}
.hright{margin-left:auto;display:flex;gap:10px;align-items:center}
nav button{background:none;border:0;border-bottom:2px solid transparent;color:var(--muted);font:inherit;padding:6px 2px;margin-right:14px;cursor:pointer}
nav button.on{color:var(--fg);border-color:var(--c1);font-weight:600}
main{max-width:1120px;margin:0 auto;padding:24px 28px 60px}.view{display:none}.view.on{display:block}
.ic{width:17px;height:17px;flex:none}
nav button,.pbar .back,#theme>span,.iconbtn,.pill-btn{display:inline-flex;align-items:center;gap:7px}
.pill-btn,#theme{background:var(--card);border:1px solid var(--line);border-radius:99px;padding:4px 12px;font:inherit;font-size:13px;color:var(--fg);cursor:pointer}
#theme>span{display:none}html[data-mode=auto] #theme .m-auto,html[data-mode=light] #theme .m-light,html[data-mode=dark] #theme .m-dark,html:not([data-mode]) #theme .m-auto{display:inline-flex}
#lang{display:inline-flex;border:1px solid var(--line);border-radius:99px;overflow:hidden}
#lang button{background:var(--card);border:0;padding:4px 11px;font:inherit;font-size:13px;color:var(--muted);cursor:pointer}
html[data-lang=en] #lang [data-l=en],html[data-lang=th] #lang [data-l=th]{background:var(--c1);color:#fff;font-weight:600}
html[data-lang=en] .l-th,html[data-lang=th] .l-en,html:not([data-lang]) .l-th{display:none}
.tiles{display:grid;grid-template-columns:repeat(auto-fit,minmax(160px,1fr));gap:12px;margin-bottom:8px}
.tile{background:var(--card);border:1px solid var(--line);border-radius:10px;padding:14px 16px}
.tile .top{display:flex;justify-content:space-between;align-items:flex-start}.tile .num{font-size:30px;font-weight:650;line-height:1.1}
.badge{display:inline-flex;width:34px;height:34px;border-radius:9px;align-items:center;justify-content:center;color:var(--c1);background:color-mix(in srgb,var(--c1) 12%,var(--card))}.badge .ic{width:19px;height:19px}
.tile .lbl{font-weight:600}.tile .note{color:var(--muted);font-size:12.5px}
figcaption,h2{display:flex;align-items:center;gap:8px;flex-wrap:wrap}figcaption .ic,h2 .ic{color:var(--c1)}
h2{font-size:17px;margin:30px 0 10px}table{width:100%;border-collapse:collapse;background:var(--card);border:1px solid var(--line);border-radius:10px;overflow:hidden}
th,td{text-align:left;padding:8px 12px;border-bottom:1px solid var(--line);vertical-align:top;font-size:14px}th{color:var(--muted);font-weight:600;font-size:13px}
tr:last-child td{border-bottom:0}td:first-child{white-space:nowrap}
.charts{display:grid;grid-template-columns:repeat(auto-fit,minmax(440px,1fr));gap:14px;margin-top:10px}
.chart{margin:0;background:var(--card);border:1px solid var(--line);border-radius:10px;padding:14px 16px;min-width:0}.chart.wide{grid-column:1/-1}
figcaption{font-weight:600;margin-bottom:8px}svg{width:100%;height:auto}svg .lbl{font-size:12px;fill:var(--muted)}svg .val{font-size:12px;fill:var(--fg)}
svg .axis{stroke:var(--line)}svg text.l-en,svg text.l-th{}html[data-lang=en] svg text.l-th,html[data-lang=th] svg text.l-en{display:none}
.donutwrap{display:flex;gap:18px;align-items:center}.donut{width:180px;flex:none}
.legend{display:flex;flex-wrap:wrap;gap:6px 16px;font-size:13px;color:var(--muted);margin-top:8px}.donutwrap .legend{flex-direction:column;margin:0}
.legend i{display:inline-block;width:10px;height:10px;border-radius:3px;margin-right:6px;vertical-align:-1px}
table.heat td,table.heat th{text-align:center}table.heat td:first-child,table.heat th:first-child{text-align:left}
.pill{background:var(--line);border-radius:99px;padding:1px 9px;font-size:12.5px;white-space:nowrap}
.s-done,.s-merged{background:color-mix(in srgb,var(--c2) 18%,var(--card))}.s-in-progress,.s-in-review,.s-ready{background:color-mix(in srgb,var(--c1) 18%,var(--card))}.s-blocked{background:color-mix(in srgb,var(--c3) 20%,var(--card))}
.muted{color:var(--muted)}.small{font-size:12.5px;margin:6px 0 0}.hint{font-size:13px;font-weight:400}
.pbar{display:flex;gap:10px;align-items:center;margin-bottom:22px}.pbar select,.pbar .back{background:var(--card);border:1px solid var(--line);border-radius:8px;padding:6px 12px;font:inherit;color:var(--fg)}.pbar select{min-width:300px}.pbar .back{cursor:pointer;display:none}.pbar.inp .back{display:inline-flex}
tr.go{cursor:pointer}tr.go:hover td{background:color-mix(in srgb,var(--c1) 7%,var(--card))}tr.go a{color:var(--c1);text-decoration:none}
.pview{display:none}.pview.on{display:block}.ptitle{font-size:20px;margin-top:0}details{margin-top:22px}summary{cursor:pointer;font-weight:600;margin-bottom:10px}
.live{display:grid;grid-template-columns:repeat(auto-fit,minmax(260px,1fr));gap:12px}
.livecard{background:var(--card);border:1px solid var(--line);border-radius:10px;padding:12px 14px}
.lc-top{display:flex;align-items:center;gap:8px}.lc-top b{font-weight:600}.lc-last{margin-top:6px;font-size:14px}.lc-meta{font-size:12.5px;color:var(--muted)}
.dot{width:10px;height:10px;border-radius:50%;background:var(--muted);flex:none}
.st{display:none;margin-left:auto;font-size:12px;border-radius:99px;padding:1px 9px;background:var(--line);white-space:nowrap}
.livecard.working .dot{background:var(--c2);box-shadow:0 0 0 4px color-mix(in srgb,var(--c2) 25%,transparent)}.livecard.working .st-working{display:inline-block;background:color-mix(in srgb,var(--c2) 20%,var(--card))}
.livecard.open .dot{background:var(--c5)}.livecard.open .st-open{display:inline-block;background:color-mix(in srgb,var(--c5) 20%,var(--card))}
.livecard.idle .st-idle{display:inline-block}
.feed{list-style:none;margin:0;padding:0;background:var(--card);border:1px solid var(--line);border-radius:10px}
.feed li{display:flex;gap:12px;padding:7px 14px;border-bottom:1px solid var(--line);font-size:14px}.feed li:last-child{border-bottom:0}
.feed .ago{color:var(--muted);width:110px;flex:none}.feed .fp{color:var(--c1);width:170px;flex:none;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.iconbtn{background:none;border:0;color:var(--fg);cursor:pointer;padding:4px;border-radius:8px}
dialog#settings{border:1px solid var(--line);border-radius:14px;background:var(--bg);color:var(--fg);padding:0;width:min(520px,92vw)}
dialog#settings::backdrop{background:rgba(0,0,0,.35)}dialog form{padding:18px 22px}
.dh{display:flex;align-items:center;justify-content:space-between}.dh h2{margin:0}
fieldset{border:0;border-top:1px solid var(--line);margin:14px 0 0;padding:12px 0 0}legend{font-weight:600;padding:0 0 6px;font-size:14px}
.opt{display:inline-flex;align-items:center;gap:6px;margin:4px 14px 4px 0;font-size:14px;cursor:pointer}
.hours{display:flex;gap:16px;margin-top:8px;font-size:13px;color:var(--muted)}.hours label{display:flex;align-items:center;gap:8px}
.hours input{width:64px;background:var(--card);color:var(--fg);border:1px solid var(--line);border-radius:6px;padding:4px 6px;font:inherit}
.df{display:flex;justify-content:flex-end;gap:10px;margin-top:18px}
.df button{font:inherit;border-radius:8px;padding:6px 14px;cursor:pointer;border:1px solid var(--line)}.primary{background:var(--c1);color:#fff;border-color:var(--c1)!important}.ghost{background:var(--card);color:var(--fg)}
#manual{max-width:860px}#manual table{margin:10px 0}#manual pre{background:var(--card);border:1px solid var(--line);border-radius:8px;padding:10px 14px;overflow:auto}
#manual code{font-size:13px}#manual h1{font-size:24px}#manual h2{border-top:1px solid var(--line);padding-top:18px}
@media (max-width:700px){.charts{grid-template-columns:1fr}.feed .fp{width:110px}.donutwrap{flex-direction:column}}
"#;

/// Runs before paint: settings, theme, language, favicon.
const HEAD_JS: &str = r#"(function(){var d=document.documentElement;
function S(k,def){try{var v=localStorage.getItem('uno-'+k);return v===null?def:v}catch(e){return def}}
window.unoS=S;
function pick(){var m=S('theme','auto'),day=+S('day','6'),night=+S('night','18');
var h=new Date().getHours();var lightNow=day<night?(h>=day&&h<night):(h>=day||h<night);
var th=m==='auto'?(lightNow?'light':'dark'):m;d.dataset.theme=th;d.dataset.mode=m;
var bg=th==='dark'?'%23FFFFFF':'%23111111',fg=th==='dark'?'%23111111':'%23FFFFFF';
var svg="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 64 64'%3E%3Crect width='64' height='64' rx='14' fill='"+bg+"'/%3E%3Cpath d='M18.5 18V33.5A11 11 0 0 0 40.5 33.5V20M35 25.5L40.5 20L46 25.5' fill='none' stroke='"+fg+"' stroke-width='5' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E";
var f=document.getElementById('fav');if(!f){f=document.createElement('link');f.id='fav';f.rel='icon';f.type='image/svg+xml';document.head.appendChild(f);}f.href=svg;}
window.unoTheme=pick;pick();setInterval(pick,60000);
var l=S('lang','en');d.dataset.lang=l;d.lang=l;})();"#;

const JS: &str = r#"
var D=document.documentElement,S=window.unoS;
function put(k,v){try{localStorage.setItem('uno-'+k,v)}catch(e){}}
function setLang(l){D.dataset.lang=l;D.lang=l;put('lang',l);
 document.querySelectorAll('option[data-en]').forEach(o=>o.textContent=o.dataset[l]);
 document.title=l==='th'?'UNO — แดชบอร์ดและคู่มือการใช้งาน':'UNO — Dashboard and Manual';ages();}
document.querySelectorAll('#lang button').forEach(b=>b.onclick=()=>{setLang(b.dataset.l);syncForm()});
document.querySelectorAll('nav button').forEach(b=>b.onclick=()=>{
 document.querySelectorAll('nav button,.view').forEach(e=>e.classList.remove('on'));
 b.classList.add('on');document.getElementById(b.dataset.v).classList.add('on');put('view',b.dataset.v)});
document.getElementById('theme').onclick=()=>{var o=['auto','light','dark'];var m=D.dataset.mode||'auto';
 put('theme',o[(o.indexOf(m)+1)%3]);window.unoTheme();syncForm()};
function showP(id,keep){if(!document.getElementById(id))id='all';
 document.querySelectorAll('.pview').forEach(e=>e.classList.remove('on'));document.getElementById(id).classList.add('on');
 document.getElementById('psel').value=id;document.querySelector('.pbar').classList.toggle('inp',id!=='all');
 if(!keep)window.scrollTo(0,0);put('proj',id)}
function bindDyn(){
 document.getElementById('psel').onchange=e=>showP(e.target.value);
 document.querySelector('.pbar .back').onclick=()=>showP('all');
 document.querySelectorAll('tr.go').forEach(r=>r.onclick=e=>{e.preventDefault();showP(r.dataset.go)});}
bindDyn();
// relative times + live status
function ago(iso){var t=Date.parse(iso);if(isNaN(t))return '';var m=Math.round((Date.now()-t)/60000),th=D.dataset.lang==='th';
 if(m<1)return th?'เมื่อสักครู่':'just now';if(m<60)return th?m+' นาทีที่แล้ว':m+' min ago';
 var h=Math.round(m/60);if(h<48)return th?h+' ชั่วโมงที่แล้ว':h+' h ago';var dd=Math.round(h/24);return th?dd+' วันที่แล้ว':dd+' days ago'}
function ages(){var win=+S('window','15');
 document.querySelectorAll('.ago').forEach(e=>e.textContent=ago(e.dataset.time));
 document.querySelectorAll('.livecard').forEach(c=>{var t=Date.parse(c.dataset.last),s=c.dataset.sessions,
  recent=!isNaN(t)&&(Date.now()-t)<win*60000,open=s!=='?'&&+s>0;
  c.classList.remove('working','open','idle');c.classList.add(recent?'working':(open?'open':'idle'))})}
setInterval(ages,30000);
// settings
var dlg=document.getElementById('settings');
document.getElementById('gear').onclick=()=>{syncForm();dlg.showModal()};
function syncForm(){[['s-lang','lang','en'],['s-theme','theme','auto'],['s-reload','reload','0'],['s-window','window','15'],['s-start','start','last'],['s-live','live','3']].forEach(([n,k,def])=>{
 var v=S(k,def);document.querySelectorAll('input[name="'+n+'"]').forEach(i=>i.checked=i.value===v)});
 document.getElementById('s-day').value=S('day','6');document.getElementById('s-night').value=S('night','18')}
dlg.addEventListener('change',e=>{var i=e.target,map={'s-lang':'lang','s-theme':'theme','s-reload':'reload','s-window':'window','s-start':'start','s-live':'live'};
 if(map[i.name]){put(map[i.name],i.value)}else if(i.id==='s-day'){put('day',Math.max(0,Math.min(23,+i.value||0)))}else if(i.id==='s-night'){put('night',Math.max(0,Math.min(23,+i.value||0)))}
 setLang(S('lang','en'));window.unoTheme();ages();armReload();if(window.unoLiveArm)window.unoLiveArm()});
document.getElementById('s-reset').onclick=()=>{['lang','theme','reload','window','start','day','night','proj','view','live'].forEach(k=>{try{localStorage.removeItem('uno-'+k)}catch(e){}});
 setLang('en');window.unoTheme();syncForm();ages();armReload()};
var rt=null;function armReload(){if(rt)clearTimeout(rt);var m=+S('reload','0');var b=document.getElementById('reloadflag');
 b.hidden=!m;if(m){b.querySelectorAll('.n').forEach(x=>x.textContent=m);rt=setTimeout(()=>location.reload(),m*60000)}}
setLang(S('lang','en'));ages();armReload();
// live mode (uno-report --serve): poll /version and swap in fresh content when anything changed
if(window.UNO_LIVE){var lf=document.getElementById('liveflag');lf.hidden=false;var sig=null,busy=false;
 function setLive(ok){lf.classList.toggle('off',!ok)}
 async function refresh(){var cur=document.getElementById('psel').value;
  var opened=[...document.querySelectorAll('#dash details')].map(d=>d.open);
  var h=await (await fetch('/',{cache:'no-store'})).text();var doc=new DOMParser().parseFromString(h,'text/html');
  ['dash','manual'].forEach(id=>{var n=doc.getElementById(id);if(n)document.getElementById(id).innerHTML=n.innerHTML});
  var m=doc.querySelector('header .meta');if(m)document.querySelector('header .meta').innerHTML=m.innerHTML;
  bindDyn();setLang(D.dataset.lang);showP(cur,true);
  document.querySelectorAll('#dash details').forEach((d,i)=>{if(opened[i])d.open=true});ages();
  lf.classList.add('flash');setTimeout(()=>lf.classList.remove('flash'),900)}
 async function tick(){if(busy)return;busy=true;
  try{var v=await (await fetch('/version',{cache:'no-store'})).text();setLive(true);
   if(sig!==null&&v!==sig)await refresh();sig=v}catch(e){setLive(false)}busy=false}
 var li=null;window.unoLiveArm=function(){if(li)clearInterval(li);li=setInterval(tick,(+S('live','3'))*1000)};tick();window.unoLiveArm()}

if(S('start','last')==='last'){try{var p=localStorage.getItem('uno-proj');if(p)showP(p)}catch(e){}
 try{var v=localStorage.getItem('uno-view');if(v)document.querySelector('nav button[data-v="'+v+'"]').click()}catch(e){}}
"#;

fn now() -> String {
    if let Ok(v) = env::var("UNO_REPORT_FAKE_NOW") {
        return v;
    }
    Command::new("date").arg("+%Y-%m-%d %H:%M").output().ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default()
}

fn render(data: &Path, manual_path: &Path, watch: Option<u64>, live: bool) -> Result<(String, usize), String> {
    let entries = fs::read_dir(data).map_err(|e| format!("cannot read {}: {e}", data.display()))?;
    let sessions = open_sessions();
    let mut dirs: Vec<PathBuf> = entries.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect();
    dirs.sort();
    let mut projects = Vec::new();
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
        p.sessions = sessions.as_ref().and_then(|m| m.get(&p.name)).copied().unwrap_or(0);
        projects.push(p);
    }
    let n = projects.len();
    let stamp = now();
    let header = format!(
        "<header><h1>{}UNO</h1><nav><button class=\"on\" data-v=\"dash\">{}{}</button><button data-v=\"manual\">{}{}</button></nav>\
         <div class=\"hright\"><span id=\"liveflag\" class=\"pill-btn\" hidden title=\"uno-report --serve\"><i class=\"ld\"></i><span class=\"l-on\">{}</span><span class=\"l-off\">{}</span></span><span id=\"reloadflag\" class=\"pill-btn\" hidden>{}{}</span>\
         <div id=\"lang\" role=\"group\" aria-label=\"Language\"><button data-l=\"en\">EN</button><button data-l=\"th\">TH</button></div>\
         <button id=\"theme\" title=\"Theme\"><span class=\"m-auto\">{}{}</span><span class=\"m-light\">{}{}</span><span class=\"m-dark\">{}{}</span></button>\
         <button id=\"gear\" class=\"pill-btn\">{}{}</button></div>\
         <div class=\"meta\" style=\"flex-basis:100%\">{}</div></header>",
        logo(26), icon("grid"), t("Dashboard", "แดชบอร์ด"), icon("book"), t("Manual", "คู่มือการใช้งาน"),
        t("Live", "สด"), t("Disconnected", "ขาดการเชื่อมต่อ"),
        icon("refresh"), "<span class=\"l-en\">Reload every <b class=\"n\"></b> min</span><span class=\"l-th\">โหลดใหม่ทุก <b class=\"n\"></b> นาที</span>",
        icon("auto"), t("Auto (by time)", "อัตโนมัติ (ตามเวลา)"), icon("sun"), t("Light", "โหมดสว่าง"), icon("moon"), t("Dark", "โหมดมืด"),
        icon("gear"), t("Settings", "ตั้งค่า"),
        t(&format!("Generated {stamp} · {n} projects · Source {}{}", data.display(),
                   watch.map(|m| format!(" · regenerated every {m} min")).unwrap_or_default()),
          &format!("จัดทำเมื่อ {stamp} · {n} โครงการ · แหล่งข้อมูล {}{}", data.display(),
                   watch.map(|m| format!(" · สร้างใหม่ทุก {m} นาที")).unwrap_or_default())));
    let page = format!(
        "<!DOCTYPE html><html lang=\"en\" data-lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\
         <title>UNO — Dashboard and Manual</title><script>{}{HEAD_JS}</script><style>{CSS}</style></head><body>{header}\
         <main><div id=\"dash\" class=\"view on\">{}</div><div id=\"manual\" class=\"view\">{}</div></main>{}<script>{JS}</script></body></html>",
        if live { "window.UNO_LIVE=true;" } else { "" },
        dashboard(&projects, sessions.is_some()), manual(manual_path), settings_dialog(watch));
    Ok((page, n))
}

fn build(data: &Path, out: &Path, manual_path: &Path, watch: Option<u64>) -> Result<usize, String> {
    let (page, n) = render(data, manual_path, watch, false)?;
    // Write to a temporary file, then rename, so a reload never sees a half-written page.
    let tmp = out.with_extension("html.tmp");
    fs::write(&tmp, page).map_err(|e| format!("cannot write {}: {e}", tmp.display()))?;
    fs::rename(&tmp, out).map_err(|e| format!("cannot write {}: {e}", out.display()))?;
    Ok(n)
}

/// Cheap fingerprint of everything the page shows: sizes and times of the data files and
/// manuals, plus the open sessions. The page polls it and reloads its content when it changes.
fn signature(data: &Path, manual_path: &Path) -> String {
    let mut sig = String::new();
    let stamp = |p: &Path| fs::metadata(p).ok()
        .map(|m| format!("{}:{:?};", m.len(), m.modified().ok()))
        .unwrap_or_default();
    sig += &stamp(manual_path);
    sig += &stamp(&manual_path.with_file_name("manual.th.md"));
    if let Ok(entries) = fs::read_dir(data) {
        let mut dirs: Vec<PathBuf> = entries.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect();
        dirs.sort();
        for d in dirs {
            sig += &stamp(&d.join("audit.log"));
            sig += &stamp(&d.join("tasks.md"));
        }
    }
    if let Some(s) = open_sessions() {
        sig += &format!("{s:?}");
    }
    let mut h: u64 = 1469598103934665603;
    for b in sig.bytes() {
        h = (h ^ b as u64).wrapping_mul(0x0100_0000_01b3);
    }
    format!("{h:016x}")
}

fn respond(stream: &mut std::net::TcpStream, status: &str, ctype: &str, body: &[u8]) {
    use std::io::Write;
    let head = format!("HTTP/1.1 {status}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nCache-Control: no-store\r\n\
                        X-Content-Type-Options: nosniff\r\nReferrer-Policy: no-referrer\r\nConnection: close\r\n\r\n", body.len());
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(body);
}

/// Local live server. Binds 127.0.0.1 only and refuses requests whose Host header is not this
/// machine (so another site cannot read the data through DNS rebinding).
fn serve(port: u16, data: PathBuf, manual_path: PathBuf, open: bool) {
    use std::io::{BufRead, BufReader};
    let listener = match std::net::TcpListener::bind(("127.0.0.1", port)) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("uno-report: cannot listen on 127.0.0.1:{port}: {e}");
            std::process::exit(1);
        }
    };
    let url = format!("http://localhost:{port}/");
    println!("uno-report: live dashboard at {url} (Ctrl+C to stop)");
    if open {
        let _ = Command::new("open").arg(&url).status();
    }
    let allowed = [format!("localhost:{port}"), format!("127.0.0.1:{port}")];
    for conn in listener.incoming() {
        let Ok(mut stream) = conn else { continue };
        let (data, manual_path, allowed) = (data.clone(), manual_path.clone(), allowed.clone());
        thread::spawn(move || {
            let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
            let mut reader = BufReader::new(match stream.try_clone() { Ok(s) => s, Err(_) => return });
            let mut first = String::new();
            if reader.read_line(&mut first).is_err() {
                return;
            }
            let mut host = String::new();
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" || line == "\n" {
                    break;
                }
                if let Some(v) = line.split_once(':').filter(|(k, _)| k.eq_ignore_ascii_case("host")).map(|(_, v)| v.trim().to_string()) {
                    host = v;
                }
            }
            let mut parts = first.split_whitespace();
            let (method, path) = (parts.next().unwrap_or(""), parts.next().unwrap_or(""));
            if !allowed.contains(&host) {
                return respond(&mut stream, "403 Forbidden", "text/plain", b"forbidden");
            }
            if method != "GET" {
                return respond(&mut stream, "405 Method Not Allowed", "text/plain", b"GET only");
            }
            match path.split('?').next().unwrap_or("") {
                "/" => match render(&data, &manual_path, None, true) {
                    Ok((page, _)) => respond(&mut stream, "200 OK", "text/html; charset=utf-8", page.as_bytes()),
                    Err(e) => respond(&mut stream, "500 Internal Server Error", "text/plain; charset=utf-8", e.as_bytes()),
                },
                "/version" => respond(&mut stream, "200 OK", "text/plain", signature(&data, &manual_path).as_bytes()),
                _ => respond(&mut stream, "404 Not Found", "text/plain", b"not found"),
            }
        });
    }
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
        serve(port, data, manual_path, args.iter().any(|a| a == "--open"));
        return;
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
