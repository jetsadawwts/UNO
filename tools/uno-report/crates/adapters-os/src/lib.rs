//! Operating-system adapters: open Claude Code sessions and local time.
use std::collections::BTreeMap;
use std::process::Command;

use app::ports::{Clock, SessionProbe};

/// Claude Code sessions open right now, by project folder name (macOS `lsof` + `ps`).
pub struct LsofSessions;

impl SessionProbe for LsofSessions {
    /// None when `lsof` cannot run.
    fn open_sessions(&self) -> Option<BTreeMap<String, usize>> {
        let lsof = Command::new("lsof").args(["-a", "-c", "claude", "-d", "cwd", "-Fpn"]).output().ok()?;
        let ps = Command::new("ps").args(["-axo", "pid=,ppid=,comm="]).output().ok();
        let ps = ps.map(|o| String::from_utf8_lossy(&o.stdout).to_string()).unwrap_or_default();
        Some(count_sessions(&String::from_utf8_lossy(&lsof.stdout), &ps))
    }
}

/// Count sessions from `lsof -Fpn` output (cwd under `/UNO/projects/<name>`) and `ps -axo pid=,ppid=,comm=`.
/// Only top-level `claude` processes count: a process whose parent is also `claude`
/// (a worker or tool process of the same session) is not a separate session.
pub fn count_sessions(lsof_out: &str, ps_out: &str) -> BTreeMap<String, usize> {
    let mut by_pid: Vec<(String, String)> = Vec::new();
    let mut pid = String::new();
    for line in lsof_out.lines() {
        if let Some(p) = line.strip_prefix('p') {
            pid = p.to_string();
        } else if let Some(path) = line.strip_prefix('n') {
            if let Some(rest) = path.split("/UNO/projects/").nth(1) {
                let name = rest.split('/').next().unwrap_or("").to_string();
                if !name.is_empty() {
                    by_pid.push((pid.clone(), name));
                }
            }
        }
    }
    let mut comm: BTreeMap<String, String> = BTreeMap::new();
    let mut parent: BTreeMap<String, String> = BTreeMap::new();
    for line in ps_out.lines() {
        let mut it = line.split_whitespace();
        if let (Some(p), Some(pp)) = (it.next(), it.next()) {
            let c: String = it.collect::<Vec<_>>().join(" ");
            comm.insert(p.to_string(), c.rsplit('/').next().unwrap_or("").to_string());
            parent.insert(p.to_string(), pp.to_string());
        }
    }
    let mut m = BTreeMap::new();
    for (p, name) in by_pid {
        let child_of_claude = parent.get(&p).and_then(|pp| comm.get(pp)).map(|c| c.starts_with("claude")).unwrap_or(false);
        if !child_of_claude {
            *m.entry(name).or_default() += 1;
        }
    }
    m
}

/// Local time from `date`, as `YYYY-MM-DD HH:MM`; empty when `date` cannot run.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> String {
        Command::new("date").arg("+%Y-%m-%d %H:%M").output().ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PS: &str = "  101     1 /Applications/claude\n  102   101 claude\n  103     1 /opt/bin/claude\n  104     1 zsh\n  105   104 claude\n";

    #[test]
    fn counts_top_level_claude_per_project() {
        let lsof = "p101\nfcwd\nn/Users/x/UNO/projects/alpha/src\np103\nfcwd\nn/Users/x/UNO/projects/beta\n";
        assert_eq!(count_sessions(lsof, PS), BTreeMap::from([("alpha".into(), 1), ("beta".into(), 1)]));
    }

    #[test]
    fn child_of_claude_is_same_session() {
        let lsof = "p101\nn/Users/x/UNO/projects/alpha\np102\nn/Users/x/UNO/projects/alpha\n";
        assert_eq!(count_sessions(lsof, PS), BTreeMap::from([("alpha".into(), 1)]));
    }

    #[test]
    fn child_of_shell_is_its_own_session() {
        let lsof = "p101\nn/Users/x/UNO/projects/alpha\np105\nn/Users/x/UNO/projects/alpha\n";
        assert_eq!(count_sessions(lsof, PS), BTreeMap::from([("alpha".into(), 2)]));
    }

    #[test]
    fn cwd_outside_projects_ignored() {
        let lsof = "p101\nn/Users/x/code/alpha\np103\nn/Users/x/UNO/projects/\n";
        assert!(count_sessions(lsof, PS).is_empty());
    }

    #[test]
    fn empty_ps_counts_every_pid() {
        let lsof = "p101\nn/Users/x/UNO/projects/alpha\np102\nn/Users/x/UNO/projects/alpha\n";
        assert_eq!(count_sessions(lsof, ""), BTreeMap::from([("alpha".into(), 2)]));
    }
}
