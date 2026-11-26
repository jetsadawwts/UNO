use std::collections::BTreeMap;

use crate::{agent_name, clip, is_open, AuditRecord, Event, EventKind, Task, RECENT_KEEP};

/// Everything the dashboard shows about one project folder in `~/UNO/data`.
#[derive(Default)]
pub struct Project {
    pub name: String,
    pub events: usize,
    pub first: String,
    pub last: String,
    pub approvals: usize,
    pub approvals_local: usize,
    pub smoke: usize,
    pub asked_owner: usize,
    pub edits_normal: usize,
    pub edits_review: usize,
    pub blocked: BTreeMap<String, usize>,
    pub agents: BTreeMap<String, usize>,
    pub per_day: BTreeMap<String, usize>,
    pub approvals_by_day: BTreeMap<String, usize>,
    pub recent: Vec<Event>,
    pub sessions: usize,
    pub tasks: Vec<Task>,
}

impl Project {
    pub fn count_status(&self, wanted: &[&str]) -> usize {
        self.tasks.iter().filter(|t| wanted.contains(&t.status.as_str())).count()
    }
    pub fn blocked_total(&self) -> usize {
        self.blocked.values().sum()
    }
    pub fn reviews(&self) -> usize {
        *self.agents.get("Epsilon").unwrap_or(&0)
    }
    pub fn open(&self) -> usize {
        self.tasks.iter().filter(|t| is_open(t)).count()
    }

    /// Count one audit record. Call [`Project::finish`] after the last one.
    pub fn apply(&mut self, r: &AuditRecord) {
        self.events += 1;
        let time = &r.time;
        let day: String = time.chars().take(10).collect();
        if !time.is_empty() {
            if self.first.is_empty() || *time < self.first {
                self.first = time.clone();
            }
            if *time > self.last {
                self.last = time.clone();
            }
            *self.per_day.entry(day.clone()).or_default() += 1;
        }
        let kind = match (r.event.as_str(), r.action.as_str()) {
            ("plan-approved", _) => {
                self.approvals += 1;
                *self.approvals_by_day.entry(day).or_default() += 1;
                if !r.backend.is_empty() && r.backend != "anthropic" {
                    self.approvals_local += 1;
                }
                Some(EventKind::PlanApproved { task: r.task.clone() })
            }
            ("smoke", _) => {
                self.smoke += 1;
                Some(EventKind::Smoke)
            }
            (_, "blocked") => {
                let rule = [&r.rule, &r.zone, &r.tool].into_iter().find(|x| !x.is_empty()).cloned().unwrap_or_default();
                *self.blocked.entry(rule.clone()).or_default() += 1;
                Some(EventKind::Blocked { rule })
            }
            (_, "asked-owner") => {
                self.asked_owner += 1;
                Some(EventKind::AskedOwner { command: clip(&r.command, 60) })
            }
            (_, "allowed") => {
                match r.zone.as_str() {
                    "review" => self.edits_review += 1,
                    _ => self.edits_normal += 1,
                }
                Some(EventKind::Edited { file: clip(&r.file, 60) })
            }
            (_, "called") if matches!(r.tool.as_str(), "Agent" | "Task") => {
                let who = agent_name(&r.name);
                *self.agents.entry(who.clone()).or_default() += 1;
                Some(EventKind::Agent { who, description: clip(&r.description, 60) })
            }
            (_, "called") => Some(EventKind::Skill { name: r.name.clone() }),
            _ => None,
        };
        if let (Some(kind), false) = (kind, time.is_empty()) {
            self.recent.push(Event { time: time.clone(), kind });
        }
    }

    /// Newest events first, at most [`RECENT_KEEP`].
    pub fn finish(&mut self) {
        self.recent.sort_by(|a, b| b.time.cmp(&a.time));
        self.recent.truncate(RECENT_KEEP);
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    fn rec(f: impl FnOnce(&mut AuditRecord)) -> AuditRecord {
        let mut r = AuditRecord::default();
        f(&mut r);
        r
    }

    #[test]
    fn apply_plan_approved_local_backend() {
        let mut p = Project::default();
        p.apply(&rec(|r| { r.event = "plan-approved".into(); r.backend = "ollama".into(); r.task = "T3".into(); r.time = "2026-01-02T10:00".into(); }));
        assert_eq!((p.approvals, p.approvals_local), (1, 1));
        assert_eq!(p.approvals_by_day["2026-01-02"], 1);
        assert_eq!(p.recent[0].kind, EventKind::PlanApproved { task: "T3".into() });
    }

    #[test]
    fn apply_plan_approved_anthropic_is_not_local() {
        let mut p = Project::default();
        p.apply(&rec(|r| { r.event = "plan-approved".into(); r.backend = "anthropic".into(); r.time = "2026-01-02T10:00".into(); }));
        assert_eq!((p.approvals, p.approvals_local), (1, 0));
    }

    #[test]
    fn apply_blocked_falls_back_rule_zone_tool() {
        let mut p = Project::default();
        p.apply(&rec(|r| { r.action = "blocked".into(); r.zone = "forbidden".into(); r.tool = "Edit".into(); r.time = "2026-01-02T10:00".into(); }));
        assert_eq!(p.blocked["forbidden"], 1);
        assert_eq!(p.blocked_total(), 1);
        assert_eq!(p.recent[0].kind, EventKind::Blocked { rule: "forbidden".into() });
    }

    #[test]
    fn apply_without_time_counts_event_only() {
        let mut p = Project::default();
        p.apply(&rec(|r| r.event = "smoke".into()));
        assert_eq!((p.events, p.smoke), (1, 1));
        assert!(p.per_day.is_empty());
        assert!(p.recent.is_empty());
        assert!(p.first.is_empty() && p.last.is_empty());
    }

    #[test]
    fn apply_called_agent_maps_name() {
        let mut p = Project::default();
        p.apply(&rec(|r| { r.action = "called".into(); r.tool = "Agent".into(); r.name = "core:security-reviewer".into(); r.description = "Review".into(); r.time = "2026-01-02T10:00".into(); }));
        assert_eq!(p.agents["Epsilon"], 1);
        assert_eq!(p.reviews(), 1);
        assert_eq!(p.recent[0].kind, EventKind::Agent { who: "Epsilon".into(), description: "Review".into() });
    }

    #[test]
    fn apply_called_skill_and_edits_and_owner() {
        let mut p = Project::default();
        p.apply(&rec(|r| { r.action = "called".into(); r.tool = "Skill".into(); r.name = "core:work".into(); r.time = "t1".into(); }));
        p.apply(&rec(|r| { r.action = "allowed".into(); r.zone = "review".into(); r.file = "a".into(); r.time = "t2".into(); }));
        p.apply(&rec(|r| { r.action = "allowed".into(); r.file = "b".into(); r.time = "t3".into(); }));
        p.apply(&rec(|r| { r.action = "asked-owner".into(); r.command = "x".repeat(70); r.time = "t4".into(); }));
        assert_eq!((p.edits_review, p.edits_normal, p.asked_owner), (1, 1, 1));
        assert_eq!(p.recent[0].kind, EventKind::Skill { name: "core:work".into() });
        assert_eq!(p.recent[3].kind, EventKind::AskedOwner { command: format!("{}…", "x".repeat(60)) });
    }

    #[test]
    fn first_and_last_track_extremes() {
        let mut p = Project::default();
        for t in ["2026-01-05T00:00", "2026-01-01T00:00", "2026-01-09T00:00"] {
            p.apply(&rec(|r| { r.event = "smoke".into(); r.time = t.into(); }));
        }
        assert_eq!((p.first.as_str(), p.last.as_str()), ("2026-01-01T00:00", "2026-01-09T00:00"));
        assert_eq!(p.per_day.len(), 3);
    }

    #[test]
    fn finish_keeps_newest_40() {
        let mut p = Project::default();
        for i in 0..45 {
            p.apply(&rec(|r| { r.event = "smoke".into(); r.time = format!("2026-01-01T00:{i:02}"); }));
        }
        p.finish();
        assert_eq!(p.recent.len(), RECENT_KEEP);
        assert_eq!(p.recent[0].time, "2026-01-01T00:44");
    }

    #[test]
    fn is_open_excludes_done_merged_dropped_wont() {
        let t = |s: &str| Task { id: "T1".into(), title: String::new(), status: s.into() };
        for s in ["done", "merged", "dropped", "won't", "wont"] {
            assert!(!is_open(&t(s)), "{s}");
        }
        assert!(is_open(&t("open")) && is_open(&t("")));
    }

    #[test]
    fn count_status_and_open() {
        let t = |s: &str| Task { id: "T1".into(), title: String::new(), status: s.into() };
        let p = Project { tasks: vec![t("done"), t("merged"), t("open"), t("dropped")], ..Default::default() };
        assert_eq!(p.count_status(DONE), 2);
        assert_eq!(p.count_status(DROPPED), 1);
        assert_eq!(p.open(), 1);
    }

    #[test]
    fn agent_name_known_and_unknown() {
        assert_eq!(agent_name("core:architect"), "Lambda");
        assert_eq!(agent_name("x:foo"), "foo");
        assert_eq!(agent_name("plain"), "plain");
    }

    #[test]
    fn clip_counts_chars_not_bytes() {
        assert_eq!(clip("สวัสดีครับ", 3), "สวั…");
        assert_eq!(clip("abc", 3), "abc");
    }
}
