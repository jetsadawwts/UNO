use domain::AuditRecord;
use serde_json::Value;

/// One record per line that parses as JSON. Fields that are missing or not strings are empty.
pub fn parse_audit(text: &str) -> Vec<AuditRecord> {
    text.lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .map(|v| {
            let s = |k: &str| v.get(k).and_then(Value::as_str).unwrap_or("").to_string();
            AuditRecord {
                time: s("time"),
                event: s("event"),
                action: s("action"),
                backend: s("backend"),
                task: s("task"),
                rule: s("rule"),
                zone: s("zone"),
                tool: s("tool"),
                command: s("command"),
                file: s("file"),
                name: s("name"),
                description: s("description"),
                session: s("session"),
                state: s("state"),
                since: s("since"),
            }
        })
        .collect()
}
