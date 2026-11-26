//! Small HTML helpers, icons and bilingual names.
use std::collections::BTreeMap;

use domain::{clip, Project, Task};


pub(crate) fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// Bilingual text: both languages are in the page, CSS shows the one in `html[data-lang]`.
pub(crate) fn t(en: &str, th: &str) -> String {
    format!("<span class=\"l-en\">{}</span><span class=\"l-th\">{}</span>", esc(en), esc(th))
}

/// Both languages inside an SVG: two <text> elements at the same spot.
pub(crate) fn svg_t(attrs: &str, en: &str, th: &str) -> String {
    format!("<text {attrs} class=\"l-en\">{}</text><text {attrs} class=\"l-th\">{}</text>", esc(en), esc(th))
}

pub(crate) fn short(s: &str, n: usize) -> String {
    clip(&s.replace('`', "").replace("**", ""), n)
}

pub(crate) const PALETTE: &[&str] = &["var(--c1)", "var(--c2)", "var(--c3)", "var(--c4)", "var(--c5)", "var(--c6)"];

pub(crate) fn color_of(i: usize) -> &'static str {
    PALETTE[i % PALETTE.len()]
}

pub(crate) fn figure(ic: &str, title: (&str, &str), wide: bool, body: &str) -> String {
    format!("<figure class=\"chart{}\"><figcaption>{}{}</figcaption>{body}</figure>",
            if wide { " wide" } else { "" }, icon(ic), t(title.0, title.1))
}

pub(crate) fn no_data() -> String {
    format!("<p class=\"muted\">{}</p>", t("No data", "ไม่มีข้อมูล"))
}

pub(crate) fn legend(items: &[(String, String, &str)]) -> String {
    let mut s = String::from("<div class=\"legend\">");
    for (en, th, c) in items {
        s += &format!("<span><i style=\"background:{c}\"></i>{}</span>", t(en, th));
    }
    s + "</div>"
}


pub(crate) fn tile(ic: &str, label: (&str, &str), value: usize, note: (String, String)) -> String {
    format!("<div class=\"tile\"><div class=\"top\"><div class=\"num\">{value}</div><span class=\"badge\">{}</span></div><div class=\"lbl\">{}</div><div class=\"note\">{}</div></div>",
            icon(ic), t(label.0, label.1), t(&note.0, &note.1))
}

/// UNO mark: a loop "U" with an arrow. The tile takes the text colour and the glyph the
/// background colour, so it is black on the light theme and white on the dark theme.
pub(crate) fn logo(size: u32) -> String {
    format!("<svg class=\"logo\" width=\"{size}\" height=\"{size}\" viewBox=\"0 0 64 64\" aria-hidden=\"true\">\
             <rect width=\"64\" height=\"64\" rx=\"14\" fill=\"var(--fg)\"/>\
             <path d=\"M18.5 18V33.5A11 11 0 0 0 40.5 33.5V20M35 25.5L40.5 20L46 25.5\" fill=\"none\" stroke=\"var(--bg)\" stroke-width=\"5\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/></svg>")
}

/// Small line icons (Lucide-style paths, drawn inline; no external files).
pub(crate) fn icon(name: &str) -> String {
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

pub(crate) fn status_color(code: &str) -> &'static str {
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
pub(crate) fn status_names(code: &str) -> (String, String) {
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
pub(crate) fn rule_names(code: &str) -> (String, String) {
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

pub(crate) fn sorted(m: BTreeMap<String, usize>) -> Vec<(String, usize)> {
    let mut v: Vec<_> = m.into_iter().collect();
    v.sort_by_key(|x| std::cmp::Reverse(x.1));
    v
}

pub(crate) fn status_map(tasks: &[&Task]) -> BTreeMap<String, usize> {
    let mut m = BTreeMap::new();
    for t in tasks {
        *m.entry(if t.status.is_empty() { "?".into() } else { t.status.clone() }).or_default() += 1;
    }
    m
}

pub(crate) fn period(p: &Project) -> String {
    format!("{} → {}", p.first.get(..10).unwrap_or("-"), p.last.get(..10).unwrap_or("-"))
}

