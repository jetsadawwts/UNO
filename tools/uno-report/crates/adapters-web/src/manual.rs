//! The manual tab: Markdown → HTML, English and Thai.
use app::ports::ManualText;
use pulldown_cmark::{html, Options, Parser};

use crate::t;

fn render_md(md: &str) -> String {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_TASKLISTS);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    let mut s = String::new();
    html::push_html(&mut s, Parser::new_ext(md, opts));
    s
}

/// Both languages, each shown by CSS for its `data-lang`; a missing one falls back to the other.
pub fn manual_html(m: &ManualText) -> String {
    let (en, th) = match (m.en.as_deref().map(render_md), m.th.as_deref().map(render_md)) {
        (Some(e), Some(t)) => (e, t),
        (Some(e), None) => (e.clone(), e),
        (None, Some(t)) => (t.clone(), t),
        (None, None) => {
            return format!("<p class=\"muted\">{}</p>",
                t(&format!("No manual found at {} — add manual.md there and run again.", m.location),
                  &format!("ไม่พบคู่มือที่ {} — วางไฟล์ manual.md ไว้แล้วรันใหม่", m.location)));
        }
    };
    format!("<div class=\"l-en\">{en}</div><div class=\"l-th\">{th}</div>")
}
