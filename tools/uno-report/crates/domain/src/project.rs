use std::collections::BTreeMap;

use crate::{agent_name, clip, is_open, AuditRecord, Event, EventKind, Live, Task, RECENT_KEEP};

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
    /// Newest heartbeat, if any.
    pub beat: Option<Event>,
    /// One entry per session with a heartbeat state, newest first.
    pub live: Vec<Live>,
    /// Latest `session-end` time by session id.
    pub ended: BTreeMap<String, String>,
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

    /// What the team did last: the newest logged event or heartbeat. A logged event says more,
    /// so it wins a tie. Session start and end are not activity.
    pub fn activity(&self) -> Option<&Event> {
        match (self.recent.first(), self.beat.as_ref()) {
            (Some(e), Some(b)) if b.time > e.time => Some(b),
            (Some(e), _) => Some(e),
            (None, b) => b,
        }
    }

    /// Count one audit record. Call [`Project::finish`] after the last one.
    pub fn apply(&mut self, r: &AuditRecord) {
        if r.action == "heartbeat" {
            // Liveness only: not an event, so it changes no count and stays out of the feed.
            if !r.time.is_empty() && self.beat.as_ref().is_none_or(|b| r.time > b.time) {
                let who = if r.name.is_empty() { "Uno".to_string() } else { agent_name(&r.name) };
                self.beat = Some(Event { time: r.time.clone(), kind: EventKind::Tool { who, tool: clip(&r.tool, 60) } });
            }
            if !r.time.is_empty() && !r.session.is_empty() && !r.state.is_empty() {
                let who = if r.name.is_empty() { "Uno".to_string() } else { agent_name(&r.name) };
                self.live.push(Live { session: r.session.clone(), time: r.time.clone(), since: r.since.clone(), state: r.state.clone(), who, tool: clip(&r.tool, 60), task: clip(&r.task, 12) });
            }
            return;
        }
        self.events += 1;
        if r.action == "session-end" && r.time > *self.ended.entry(r.session.clone()).or_default() {
            self.ended.insert(r.session.clone(), r.time.clone());
        }
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
        let ended = &self.ended;
        self.live.retain(|l| ended.get(&l.session).is_none_or(|end| *end < l.time));
        self.live.sort_by(|a, b| b.time.cmp(&a.time));
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

    fn agent_call(time: &str) -> AuditRecord {
        rec(|r| { r.action = "called".into(); r.tool = "Agent".into(); r.name = "core:intake-triager".into(); r.description = "Write spec".into(); r.session = "s1".into(); r.time = time.into(); })
    }
    fn beat(time: &str, tool: &str, name: &str) -> AuditRecord {
        rec(|r| { r.action = "heartbeat".into(); r.tool = tool.into(); r.name = name.into(); r.session = "s1".into(); r.time = time.into(); })
    }
    fn built(records: &[AuditRecord]) -> Project {
        let mut p = Project::default();
        for r in records { p.apply(r); }
        p.finish();
        p
    }

    #[test]
    fn session_start_and_end_are_not_activity() {
        let p = built(&[
            agent_call("2026-01-02T03:05:49+00:00"),
            rec(|r| { r.action = "session-start".into(); r.session = "s2".into(); r.time = "2026-01-02T03:47:38+00:00".into(); }),
            rec(|r| { r.action = "session-end".into(); r.session = "s2".into(); r.time = "2026-01-02T03:47:56+00:00".into(); }),
        ]);
        assert_eq!(p.activity().map(|e| e.time.clone()), Some("2026-01-02T03:05:49+00:00".into()));
    }

    #[test]
    fn newer_heartbeat_is_the_activity() {
        let p = built(&[agent_call("2026-01-02T03:00:00+00:00"), beat("2026-01-02T03:10:00+00:00", "Read", "")]);
        let a = p.activity().unwrap();
        assert_eq!((a.time.as_str(), &a.kind), ("2026-01-02T03:10:00+00:00", &EventKind::Tool { who: "Uno".into(), tool: "Read".into() }));
    }

    #[test]
    fn heartbeat_names_the_subagent() {
        let p = built(&[beat("2026-01-02T03:10:00+00:00", "Grep", "core:codebase-explorer")]);
        assert_eq!(p.activity().unwrap().kind, EventKind::Tool { who: "Beta".into(), tool: "Grep".into() });
    }

    #[test]
    fn heartbeat_is_not_counted_or_listed() {
        let p = built(&[beat("2026-01-02T03:10:00+00:00", "Read", "")]);
        assert_eq!((p.events, p.per_day.len(), p.recent.len()), (0, 0, 0));
        assert!(p.first.is_empty() && p.last.is_empty() && p.agents.is_empty() && p.blocked.is_empty());
    }

    #[test]
    fn logged_event_wins_over_older_or_same_time_heartbeat() {
        for t in ["2026-01-02T03:00:00+00:00", "2026-01-02T02:59:00+00:00"] {
            let p = built(&[agent_call("2026-01-02T03:00:00+00:00"), beat(t, "Agent", "")]);
            assert!(matches!(p.activity().unwrap().kind, EventKind::Agent { .. }), "{t}");
        }
    }

    fn live(session: &str, time: &str, state: &str, task: &str) -> AuditRecord {
        rec(|r| { r.action = "heartbeat".into(); r.session = session.into(); r.time = time.into(); r.since = time.into(); r.state = state.into(); r.tool = "Bash".into(); r.task = task.into(); })
    }

    #[test]
    fn each_session_has_its_own_live_entry_newest_first() {
        let p = built(&[live("a", "2026-01-02T03:00:00+00:00", "running", "T38"), live("b", "2026-01-02T03:01:00+00:00", "waiting", "")]);
        let got: Vec<_> = p.live.iter().map(|l| (l.session.as_str(), l.state.as_str(), l.task.as_str(), l.who.as_str(), l.tool.as_str())).collect();
        assert_eq!(got, [("b", "waiting", "", "Uno", "Bash"), ("a", "running", "T38", "Uno", "Bash")]);
        assert_eq!((p.events, p.recent.len()), (0, 0));
    }

    #[test]
    fn ended_session_has_no_live_entry_unless_it_came_back() {
        let end = |t: &str| rec(|r| { r.action = "session-end".into(); r.session = "a".into(); r.time = t.into(); });
        let gone = built(&[end("2026-01-02T03:05:00+00:00"), live("a", "2026-01-02T03:00:00+00:00", "running", "")]);
        assert!(gone.live.is_empty());
        let back = built(&[end("2026-01-02T03:05:00+00:00"), live("a", "2026-01-02T03:06:00+00:00", "running", "")]);
        assert_eq!(back.live.len(), 1);
    }

    #[test]
    fn heartbeat_without_state_has_no_live_entry() {
        let p = built(&[beat("2026-01-02T03:10:00+00:00", "Read", "")]);
        assert!(p.live.is_empty() && p.beat.is_some());
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
