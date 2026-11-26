use domain::Project;

use crate::ports::{Clock, ProjectStore, SessionProbe};

/// Everything a view needs to draw the dashboard.
pub struct Dashboard {
    pub projects: Vec<Project>,
    pub sessions_known: bool,
    pub generated_at: String,
}

pub fn build_dashboard(store: &impl ProjectStore, sessions: &impl SessionProbe, clock: &impl Clock) -> Result<Dashboard, String> {
    let files = store.projects()?;
    let open = sessions.open_sessions();
    let projects = files
        .into_iter()
        .map(|f| {
            let mut p = Project { name: f.name, tasks: f.tasks, ..Default::default() };
            for r in &f.audit {
                p.apply(r);
            }
            p.finish();
            p.sessions = open.as_ref().and_then(|m| m.get(&p.name)).copied().unwrap_or(0);
            p
        })
        .collect();
    Ok(Dashboard { projects, sessions_known: open.is_some(), generated_at: clock.now() })
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::ports::*;
    use crate::*;
    use domain::{AuditRecord, Task};
    use std::cell::Cell;
    use std::collections::BTreeMap;

    pub struct FakeStore { pub projects: Result<Vec<ProjectFiles>, String>, pub stamp: String }
    impl ProjectStore for FakeStore {
        fn projects(&self) -> Result<Vec<ProjectFiles>, String> {
            match &self.projects {
                Ok(v) => Ok(v.iter().map(|p| ProjectFiles { name: p.name.clone(), audit: p.audit.clone(), tasks: p.tasks.clone() }).collect()),
                Err(e) => Err(e.clone()),
            }
        }
        fn stamp(&self) -> String { self.stamp.clone() }
    }
    pub struct FakeSessions(pub Option<BTreeMap<String, usize>>);
    impl SessionProbe for FakeSessions {
        fn open_sessions(&self) -> Option<BTreeMap<String, usize>> { self.0.clone() }
    }
    pub struct FakeClock(pub &'static str, pub Cell<usize>);
    impl Clock for FakeClock {
        fn now(&self) -> String { self.1.set(self.1.get() + 1); self.0.to_string() }
    }

    pub fn store() -> FakeStore {
        let smoke = AuditRecord { event: "smoke".into(), time: "2026-01-01T00:00".into(), ..Default::default() };
        let task = Task { id: "T1".into(), title: "x".into(), status: "open".into() };
        FakeStore {
            projects: Ok(vec![
                ProjectFiles { name: "alpha".into(), audit: vec![smoke.clone(), smoke], tasks: vec![task.clone()] },
                ProjectFiles { name: "beta".into(), audit: vec![], tasks: vec![task] },
            ]),
            stamp: "s1".into(),
        }
    }

    #[test]
    fn builds_projects_from_store() {
        let d = build_dashboard(&store(), &FakeSessions(Some(BTreeMap::new())), &FakeClock("2026-01-15 09:30", Cell::new(0))).unwrap();
        let names: Vec<_> = d.projects.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, ["alpha", "beta"]);
        assert_eq!((d.projects[0].smoke, d.projects[0].recent.len(), d.projects[0].tasks.len()), (2, 2, 1));
        assert_eq!(d.projects[1].events, 0);
    }

    #[test]
    fn sessions_known_false_when_probe_none() {
        let d = build_dashboard(&store(), &FakeSessions(None), &FakeClock("n", Cell::new(0))).unwrap();
        assert!(!d.sessions_known);
        assert!(d.projects.iter().all(|p| p.sessions == 0));
    }

    #[test]
    fn sessions_mapped_by_project_name() {
        let m = BTreeMap::from([("alpha".to_string(), 2)]);
        let d = build_dashboard(&store(), &FakeSessions(Some(m)), &FakeClock("n", Cell::new(0))).unwrap();
        assert!(d.sessions_known);
        assert_eq!((d.projects[0].sessions, d.projects[1].sessions), (2, 0));
    }

    #[test]
    fn store_error_is_returned() {
        let s = FakeStore { projects: Err("cannot read /x: gone".into()), stamp: String::new() };
        let e = build_dashboard(&s, &FakeSessions(None), &FakeClock("n", Cell::new(0))).err();
        assert_eq!(e.as_deref(), Some("cannot read /x: gone"));
    }

    #[test]
    fn generated_at_comes_from_clock_once() {
        let c = FakeClock("2026-01-15 09:30", Cell::new(0));
        let d = build_dashboard(&store(), &FakeSessions(None), &c).unwrap();
        assert_eq!(d.generated_at, "2026-01-15 09:30");
        assert_eq!(c.1.get(), 1);
    }

    #[test]
    fn signature_is_16_hex_chars() {
        let s = signature(&store(), "m", &FakeSessions(None));
        assert_eq!(s.len(), 16);
        assert!(s.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn signature_changes_when_stamp_manual_or_sessions_change() {
        let base = signature(&store(), "m", &FakeSessions(None));
        let mut other = store();
        other.stamp = "s2".into();
        assert_ne!(base, signature(&other, "m", &FakeSessions(None)));
        assert_ne!(base, signature(&store(), "m2", &FakeSessions(None)));
        assert_ne!(base, signature(&store(), "m", &FakeSessions(Some(BTreeMap::from([("alpha".into(), 1)])))));
        assert_eq!(base, signature(&store(), "m", &FakeSessions(None)));
    }
}
