//! Charts drawn as inline SVG.
use std::collections::BTreeMap;

use domain::{clip, Project, DONE, DROPPED};

use crate::*;

pub(crate) struct Bar {
    pub(crate) en: String,
    pub(crate) th: String,
    pub(crate) value: usize,
}

/// Horizontal bars, one colour.
pub(crate) fn hbars(ic: &str, title: (&str, &str), items: &[Bar], color: &str) -> String {
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
pub(crate) fn donut(ic: &str, title: (&str, &str), items: &[(String, String, usize, &'static str)]) -> String {
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
pub(crate) fn stacked_projects(projects: &[&Project]) -> String {
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
pub(crate) fn daily_by_project(projects: &[&Project]) -> String {
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
                        x + bw / 2.0, y - 4.0, x + bw / 2.0, height + 16.0, day.get(5..).unwrap_or(""));
    }
    svg += "</svg>";
    let leg: Vec<(String, String, &str)> = projects.iter().enumerate().map(|(i, p)| (p.name.clone(), p.name.clone(), color_of(i))).collect();
    figure("activity", title, true, &(svg + &legend(&leg)))
}

/// Cumulative approved plans over time, all projects together.
pub(crate) fn approvals_line(projects: &[&Project]) -> String {
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
                        xs(i), ys(*v), xs(i), ys(*v) - 8.0, xs(i), top + height + 18.0, d.get(5..).unwrap_or(""));
    }
    figure("approve", title, false, &(svg + "</svg>"))
}

/// Guard rule × project table, shaded by count.
pub(crate) fn heatmap(projects: &[&Project]) -> String {
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

