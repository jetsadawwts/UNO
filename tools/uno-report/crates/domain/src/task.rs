/// A row of a project's `tasks.md` table.
#[derive(Clone, Debug, PartialEq)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub status: String,
}

pub const DONE: &[&str] = &["done", "merged"];
pub const DROPPED: &[&str] = &["dropped", "won't", "wont"];
/// How many recent events each project keeps for the activity feed.
pub const RECENT_KEEP: usize = 40;

pub fn is_open(t: &Task) -> bool {
    !DONE.contains(&t.status.as_str()) && !DROPPED.contains(&t.status.as_str())
}

/// Shorten to `n` characters (not bytes), adding `…` when cut.
pub fn clip(s: &str, n: usize) -> String {
    if s.chars().count() <= n { s.to_string() } else { format!("{}…", s.chars().take(n).collect::<String>()) }
}
