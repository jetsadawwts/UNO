//! Dashboard sections and the whole page.
use std::collections::BTreeMap;

use domain::{is_open, Event, EventKind, Project, Task, DONE, DROPPED};

use crate::*;

/// English and Thai text for an activity-feed entry.
pub(crate) fn event_text(k: &EventKind) -> (String, String) {
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
        EventKind::Tool { who, tool } => (format!("{who}: used {tool}"), format!("{who}: ใช้ {tool}")),
    }
}

pub(crate) fn ev_t(e: &Event) -> String {
    let (en, th) = event_text(&e.kind);
    t(&en, &th)
}


// ---------------------------------------------------------------- sections

pub(crate) fn task_table(tasks: &[&Task]) -> String {
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

pub(crate) fn tiles_for(projects: &[&Project], show_count: bool) -> String {
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
pub(crate) fn activity_panel(projects: &[&Project], sessions_known: bool) -> String {
    let mut out = format!(
        "<h2>{}{} <span class=\"muted hint\">{}</span></h2><div class=\"live\">",
        icon("live"), t("Team activity", "สถานะการทำงานของทีม"),
        t("From open Claude Code sessions and the latest audit.log events",
          "จาก session ของ Claude Code ที่เปิดอยู่ และเหตุการณ์ล่าสุดใน audit.log"));
    for p in projects {
        let last = p.activity();
        let at = last.map(|e| e.time.as_str()).unwrap_or("");
        let sess = if sessions_known { p.sessions.to_string() } else { "?".into() };
        out += &format!(
            "<div class=\"livecard\" data-last=\"{}\" data-sessions=\"{sess}\"><div class=\"lc-top\"><span class=\"dot\"></span><b>{}</b>\
             <span class=\"st st-working\">{}</span><span class=\"st st-open\">{}</span><span class=\"st st-idle\">{}</span></div>\
             <div class=\"lc-last\">{}</div><div class=\"lc-meta\"><span class=\"ago\" data-time=\"{}\"></span> · {}</div></div>",
            esc(at), esc(&p.name),
            t("Working", "กำลังทำงาน"), t("Session open, idle", "เปิด session อยู่ ยังไม่มีกิจกรรม"), t("Not running", "ไม่ได้ทำงาน"),
            last.map(ev_t).unwrap_or_else(|| t("No activity yet", "ยังไม่มีกิจกรรม")),
            esc(at),
            t(&format!("{sess} open session(s)"), &format!("เปิดอยู่ {sess} session")));
    }
    out + "</div>"
}

pub(crate) fn feed(events: &[(&str, &Event)]) -> String {
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

pub(crate) fn overview(projects: &[Project], sessions_known: bool) -> String {
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

pub(crate) fn project_view(p: &Project, sessions_known: bool) -> String {
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

/// `<option>`s of the project picker. `<option>` cannot hold markup, so options carry both
/// texts and the script picks one.
pub(crate) fn psel_options(projects: &[Project]) -> String {
    let mut o = String::from("<option value=\"all\" data-en=\"All projects (overview)\" data-th=\"ภาพรวมทุกโครงการ\">All projects (overview)</option>");
    for (i, p) in projects.iter().enumerate() {
        let open = p.open();
        let (en, th) = (format!("{} · {open} in progress", p.name), format!("{} · ระหว่างดำเนินการ {open} งาน", p.name));
        o += &format!("<option value=\"p{i}\" data-en=\"{}\" data-th=\"{}\">{}</option>", esc(&en), esc(&th), esc(&en));
    }
    o
}

pub(crate) fn dashboard(projects: &[Project], sessions_known: bool) -> String {
    let mut out = format!(
        "<div class=\"pbar\"><button class=\"back\" data-p=\"all\">{}{}</button><select id=\"psel\" aria-label=\"Select a project\">{}</select></div>",
        icon("back"), t("Back to overview", "กลับสู่ภาพรวม"), psel_options(projects));
    out += &format!("<div class=\"pview on\" id=\"all\">{}</div>", overview(projects, sessions_known));
    for (i, p) in projects.iter().enumerate() {
        out += &format!("<div class=\"pview\" id=\"p{i}\">{}</div>", project_view(p, sessions_known));
    }
    out
}

/// The header's one-line note: when, how many projects, where from.
pub(crate) fn meta_html(d: &app::Dashboard, data_label: &str, watch: Option<u64>) -> String {
    let (n, stamp) = (d.projects.len(), &d.generated_at);
    t(&format!("Generated {stamp} · {n} projects · Source {}{}", data_label,
               watch.map(|m| format!(" · regenerated every {m} min")).unwrap_or_default()),
      &format!("จัดทำเมื่อ {stamp} · {n} โครงการ · แหล่งข้อมูล {}{}", data_label,
               watch.map(|m| format!(" · สร้างใหม่ทุก {m} นาที")).unwrap_or_default()))
}


pub(crate) fn settings_dialog(watch: Option<u64>) -> String {
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


/// How the page will be delivered: a file (optionally regenerated every N minutes) or the live server.
pub enum PageMode {
    File { watch: Option<u64> },
    Live,
}

/// The whole page: header, dashboard, manual and settings. Byte-identical to v0.5.12.
pub fn render_page(d: &app::Dashboard, m: &app::ports::ManualText, data_label: &str, mode: &PageMode) -> String {
    let watch = match mode { PageMode::File { watch } => *watch, PageMode::Live => None };
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
        meta_html(d, data_label, watch));
    let page = format!(
        "<!DOCTYPE html><html lang=\"en\" data-lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\
         <title>UNO — Dashboard and Manual</title><script>{}{HEAD_JS}</script><style>{CSS}</style></head><body>{header}\
         <main><div id=\"dash\" class=\"view on\">{}</div><div id=\"manual\" class=\"view\">{}</div></main>{}<script>{JS}</script>{}</body></html>",
        if matches!(mode, PageMode::Live) { "window.UNO_LIVE=true;" } else { "" },
        dashboard(&d.projects, d.sessions_known), manual_html(m), settings_dialog(watch),
        if matches!(mode, PageMode::Live) { format!("<script>{LIVE_JS}</script>") } else { String::new() });
    page
}

