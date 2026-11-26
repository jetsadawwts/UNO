use domain::Task;

fn cells(line: &str) -> Vec<String> {
    let t = line.trim().trim_start_matches('|').trim_end_matches('|');
    t.split('|').map(|c| c.trim().to_string()).collect()
}

/// Rows of Markdown tables whose header row starts with `ID` and has a `Status` column.
/// IDs must be `T` + digits; the first row of a duplicated ID wins. Status is lower-cased,
/// stripped of `*` and backticks, and cut to its first word.
pub fn parse_tasks(text: &str) -> Vec<Task> {
    let mut tasks: Vec<Task> = Vec::new();
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
        if tasks.iter().any(|t| t.id == id) {
            continue;
        }
        let status = c.get(s).map(|x| x.to_lowercase()).unwrap_or_default();
        let status = status.trim_matches(|ch: char| ch == '*' || ch == '`').split_whitespace().next().unwrap_or("").to_string();
        let title = task_col.and_then(|t| c.get(t)).cloned().unwrap_or_default();
        tasks.push(Task { id, title, status });
    }
    tasks
}
