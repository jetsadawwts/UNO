//! Hand-off timeline: each agent call from Uno matched with what that member did afterwards,
//! from `office.log` (written by `trace_heartbeat.py`: start, tool, stop, return; names only).
use std::collections::BTreeMap;

use crate::{agent_name, parse_time};

/// Uno calling a team member, from `audit.log` (`action: called`, tool Agent or Task).
#[derive(Clone, Debug, PartialEq)]
pub struct Call {
    pub time: String,
    pub session: String,
    /// Team member name, e.g. `Epsilon`.
    pub who: String,
    pub description: String,
}

/// One line of `office.log`.
#[derive(Clone, Debug, PartialEq)]
pub struct Step {
    pub time: String,
    pub session: String,
    /// Claude Code agent id; empty for `return` (the main thread got an answer).
    pub agent: String,
    /// Team member name.
    pub who: String,
    /// `start`, `tool`, `stop` or `return`.
    pub kind: String,
    pub tool: String,
}

/// A call and, when it can be matched for sure, the member's work after it.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Handoff {
    pub time: String,
    pub session: String,
    pub who: String,
    pub description: String,
    /// When the member started; empty when no start could be matched.
    pub started: String,
    /// When the member finished; empty while running or when unknown.
    pub ended: String,
    /// Latest step seen for this member's work.
    pub last: String,
    /// Tool name → times used.
    pub tools: BTreeMap<String, usize>,
}

/// A start belongs to a call when the call is at most this many seconds before it (or 10 after).
const MATCH_WINDOW_SECS: i64 = 120;

/// Newest first. A start is matched to the latest unmatched call of the same member in the same
/// session within [`MATCH_WINDOW_SECS`]; when two such calls are open at once (parallel calls of
/// one member) the start is left unmatched rather than guessed.
pub fn handoffs(calls: &[Call], steps: &[Step]) -> Vec<Handoff> {
    let t = |s: &str| parse_time(s).unwrap_or(i64::MIN);
    let mut calls: Vec<&Call> = calls.iter().collect();
    calls.sort_by_key(|c| t(&c.time));
    let mut steps: Vec<&Step> = steps.iter().collect();
    steps.sort_by_key(|s| t(&s.time));

    // First step of each agent id is its start (SubagentStart, or its first tool if that is missing).
    let mut first: Vec<&Step> = Vec::new();
    for s in &steps {
        if !s.agent.is_empty() && s.kind != "return" && !first.iter().any(|f| f.session == s.session && f.agent == s.agent) {
            first.push(s);
        }
    }
    let mut owner: BTreeMap<usize, (String, String)> = BTreeMap::new(); // call index → (session, agent id)
    for s in &first {
        let st = t(&s.time);
        let open: Vec<usize> = calls.iter().enumerate()
            .filter(|(i, c)| !owner.contains_key(i) && c.session == s.session && c.who == s.who
                    && t(&c.time) <= st + 10 && st - t(&c.time) <= MATCH_WINDOW_SECS)
            .map(|(i, _)| i)
            .collect();
        if open.len() == 1 {
            owner.insert(open[0], (s.session.clone(), s.agent.clone()));
        }
    }
    let mut out: Vec<Handoff> = calls.iter().enumerate().map(|(i, c)| {
        let mut h = Handoff { time: c.time.clone(), session: c.session.clone(), who: c.who.clone(), description: c.description.clone(), ..Default::default() };
        if let Some((session, agent)) = owner.get(&i) {
            let mine: Vec<&&Step> = steps.iter().filter(|s| &s.session == session && &s.agent == agent).collect();
            h.started = mine.first().map(|s| s.time.clone()).unwrap_or_default();
            h.last = mine.last().map(|s| s.time.clone()).unwrap_or_default();
            for s in &mine {
                if s.kind == "tool" && !s.tool.is_empty() {
                    *h.tools.entry(s.tool.clone()).or_default() += 1;
                }
            }
            h.ended = mine.iter().find(|s| s.kind == "stop").map(|s| s.time.clone())
                .or_else(|| steps.iter()
                    .find(|s| s.kind == "return" && &s.session == session && s.who == h.who && t(&s.time) >= t(&h.started))
                    .map(|s| s.time.clone()))
                .unwrap_or_default();
            if !h.ended.is_empty() && t(&h.ended) > t(&h.last) {
                h.last = h.ended.clone();
            }
        }
        h
    }).collect();
    out.reverse();
    out
}

/// Team member name for a raw agent type; `Uno` for none.
pub fn member(raw: &str) -> String {
    if raw.is_empty() { "Uno".into() } else { agent_name(raw) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(time: &str, who: &str, d: &str) -> Call {
        Call { time: format!("2026-01-02T03:{time}+00:00"), session: "s".into(), who: who.into(), description: d.into() }
    }
    fn step(time: &str, agent: &str, who: &str, kind: &str, tool: &str) -> Step {
        Step { time: format!("2026-01-02T03:{time}+00:00"), session: "s".into(), agent: agent.into(), who: who.into(), kind: kind.into(), tool: tool.into() }
    }

    #[test]
    fn a_call_gets_start_tools_and_end() {
        let h = handoffs(&[call("06:00", "Epsilon", "Review T2")],
            &[step("06:02", "a1", "Epsilon", "start", ""), step("06:10", "a1", "Epsilon", "tool", "Read"),
              step("07:00", "a1", "Epsilon", "tool", "Read"), step("08:00", "a1", "Epsilon", "tool", "Bash"),
              step("09:00", "a1", "Epsilon", "stop", "")]);
        assert_eq!(h.len(), 1);
        assert_eq!((h[0].started.as_str(), h[0].ended.as_str()), ("2026-01-02T03:06:02+00:00", "2026-01-02T03:09:00+00:00"));
        assert_eq!(h[0].tools.get("Read"), Some(&2));
        assert_eq!(h[0].tools.get("Bash"), Some(&1));
    }

    #[test]
    fn return_ends_work_when_stop_is_missing_and_first_tool_is_the_start() {
        let h = handoffs(&[call("06:00", "Beta", "List")],
            &[step("06:05", "b1", "Beta", "tool", "Grep"), step("07:00", "", "Beta", "return", "")]);
        assert_eq!((h[0].started.as_str(), h[0].ended.as_str()), ("2026-01-02T03:06:05+00:00", "2026-01-02T03:07:00+00:00"));
    }

    #[test]
    fn running_work_has_no_end() {
        let h = handoffs(&[call("06:00", "Delta", "Tests")], &[step("06:01", "d1", "Delta", "start", "")]);
        assert!(!h[0].started.is_empty() && h[0].ended.is_empty());
    }

    #[test]
    fn two_open_calls_of_one_member_are_not_guessed() {
        let h = handoffs(&[call("06:00", "Beta", "A"), call("06:01", "Beta", "B")],
            &[step("06:02", "x", "Beta", "start", ""), step("06:03", "y", "Beta", "start", "")]);
        assert!(h.iter().all(|x| x.started.is_empty()), "{h:?}");
    }

    #[test]
    fn calls_in_sequence_match_one_by_one_newest_first() {
        let h = handoffs(&[call("06:00", "Beta", "A"), call("10:00", "Beta", "B")],
            &[step("06:01", "x", "Beta", "start", ""), step("06:30", "x", "Beta", "stop", ""), step("10:01", "y", "Beta", "start", "")]);
        assert_eq!((h[0].description.as_str(), h[0].ended.is_empty()), ("B", true));
        assert_eq!((h[1].description.as_str(), h[1].ended.as_str()), ("A", "2026-01-02T03:06:30+00:00"));
    }

    #[test]
    fn a_start_long_after_the_call_or_in_another_session_is_not_matched() {
        let mut other = step("06:01", "z", "Beta", "start", "");
        other.session = "other".into();
        let h = handoffs(&[call("06:00", "Beta", "A")], &[step("20:00", "x", "Beta", "start", ""), other]);
        assert!(h[0].started.is_empty());
    }
}
