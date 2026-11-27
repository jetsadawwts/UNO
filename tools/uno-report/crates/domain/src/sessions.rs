//! Open Claude Code sessions, counted from audit events (no looking at other processes).
use std::collections::BTreeMap;

use crate::AuditRecord;

/// A session with no event for this long counts as closed (Claude Code may exit without
/// running its SessionEnd hook, e.g. when the terminal is closed).
pub const SESSION_IDLE_SECS: i64 = 30 * 60;

/// Seconds since 1970 for `YYYY-MM-DDTHH:MM:SS[.frac](Z|±HH:MM)`; None for anything else.
pub fn parse_time(s: &str) -> Option<i64> {
    let b = s.as_bytes();
    if b.len() < 20 || b[4] != b'-' || b[7] != b'-' || b[10] != b'T' || b[13] != b':' || b[16] != b':' {
        return None;
    }
    let num = |r: std::ops::Range<usize>| -> Option<i64> { s.get(r)?.parse::<i64>().ok() };
    let (y, mo, d, h, mi, se) = (num(0..4)?, num(5..7)?, num(8..10)?, num(11..13)?, num(14..16)?, num(17..19)?);
    if !(1..=12).contains(&mo) || !(1..=31).contains(&d) || h > 23 || mi > 59 || se > 60 {
        return None;
    }
    let mut rest = &s[19..];
    if let Some(frac) = rest.strip_prefix('.') {
        let n = frac.find(|c: char| !c.is_ascii_digit()).unwrap_or(frac.len());
        rest = &frac[n..];
    }
    let offset = match rest {
        "Z" => 0,
        _ if rest.len() == 6 && (rest.starts_with('+') || rest.starts_with('-')) && rest.get(3..4) == Some(":") => {
            let v = rest.get(1..3)?.parse::<i64>().ok()? * 3600 + rest.get(4..6)?.parse::<i64>().ok()? * 60;
            if rest.starts_with('-') { -v } else { v }
        }
        _ => return None,
    };
    // days from civil (Howard Hinnant)
    let y2 = if mo <= 2 { y - 1 } else { y };
    let era = y2.div_euclid(400);
    let yoe = y2 - era * 400;
    let doy = (153 * (if mo > 2 { mo - 3 } else { mo + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe - 719468;
    Some(days * 86400 + h * 3600 + mi * 60 + se - offset)
}

/// Sessions whose latest event is not `session-end` and is at most `idle_secs` before `now`.
pub fn open_sessions(records: &[AuditRecord], now: i64, idle_secs: i64) -> usize {
    let mut latest: BTreeMap<&str, (i64, bool)> = BTreeMap::new(); // session → (time, ended)
    for r in records {
        let (Some(t), false) = (parse_time(&r.time), r.session.is_empty()) else { continue };
        let e = latest.entry(r.session.as_str()).or_insert((i64::MIN, false));
        if t >= e.0 {
            *e = (t, r.action == "session-end");
        }
    }
    latest.values().filter(|(t, ended)| !ended && now - t <= idle_secs).count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AuditRecord;

    fn r(time: &str, session: &str, action: &str) -> AuditRecord {
        AuditRecord { time: time.into(), session: session.into(), action: action.into(), ..Default::default() }
    }
    const NOW: &str = "2026-11-27T03:00:00+00:00";
    fn now() -> i64 { parse_time(NOW).unwrap() }

    #[test]
    fn parse_time_handles_offsets_and_z() {
        assert_eq!(parse_time("1970-01-01T00:00:00+00:00"), Some(0));
        assert_eq!(parse_time("1970-01-01T07:00:00+07:00"), Some(0));
        assert_eq!(parse_time("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(parse_time("2026-11-27T10:00:00+07:00"), parse_time("2026-11-27T03:00:00+00:00"));
        assert_eq!(parse_time("2024-02-29T00:00:00+00:00").map(|t| t % 86400), Some(0));
        assert_eq!(parse_time("2026-11-27T03:00:00.123+00:00"), parse_time(NOW));
        for bad in ["", "2026", "2026-11-27", "2026-13-01T00:00:00Z", "2026-11-27T25:00:00Z", "garbage", "2026-11-27T03:00:00+กข", "2026-11-27T03:00:00+0ก0", "2026-11-27T03:00:00-ก:0", "2026-11-27T03:00:00ก", "2026-11-27T03:00:0ก+00:00"] {
            assert_eq!(parse_time(bad), None, "{bad}");
        }
    }

    #[test]
    fn started_session_with_recent_activity_is_open() {
        let recs = [r("2026-11-27T02:50:00+00:00", "s1", "session-start")];
        assert_eq!(open_sessions(&recs, now(), SESSION_IDLE_SECS), 1);
    }

    #[test]
    fn ended_session_is_closed() {
        let recs = [r("2026-11-27T02:50:00+00:00", "s1", "session-start"), r("2026-11-27T02:55:00+00:00", "s1", "session-end")];
        assert_eq!(open_sessions(&recs, now(), SESSION_IDLE_SECS), 0);
    }

    #[test]
    fn idle_past_limit_is_closed() {
        let recs = [r("2026-11-27T02:29:00+00:00", "s1", "session-start")];
        assert_eq!(open_sessions(&recs, now(), SESSION_IDLE_SECS), 0);
    }

    #[test]
    fn any_event_keeps_a_session_alive() {
        let recs = [r("2026-11-27T01:00:00+00:00", "s1", "session-start"), r("2026-11-27T02:59:00+07:00", "s1", "allowed"),
                    r("2026-11-27T09:58:00+07:00", "s1", "called")];
        assert_eq!(open_sessions(&recs, now(), SESSION_IDLE_SECS), 1);
    }

    #[test]
    fn events_without_a_session_or_time_are_ignored() {
        let recs = [r("2026-11-27T02:59:00+00:00", "", "allowed"), r("", "s1", "session-start"), r("bad", "s2", "allowed")];
        assert_eq!(open_sessions(&recs, now(), SESSION_IDLE_SECS), 0);
    }

    #[test]
    fn two_sessions_counted_separately() {
        let recs = [r("2026-11-27T02:50:00+00:00", "a", "session-start"), r("2026-11-27T02:51:00+00:00", "b", "session-start"),
                    r("2026-11-27T02:52:00+00:00", "b", "session-end"), r("2026-11-27T02:53:00+00:00", "c", "allowed")];
        assert_eq!(open_sessions(&recs, now(), SESSION_IDLE_SECS), 2);
    }

    #[test]
    fn activity_after_end_reopens() {
        let recs = [r("2026-11-27T02:50:00+00:00", "s1", "session-end"), r("2026-11-27T02:55:00+00:00", "s1", "session-start")];
        assert_eq!(open_sessions(&recs, now(), SESSION_IDLE_SECS), 1);
    }
}
