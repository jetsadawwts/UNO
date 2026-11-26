use std::time::Duration;

use crate::ports::{ChangeFeed, ProjectStore, SessionProbe};
use crate::signature;

/// Turns "files changed, or `tick` passed" into "what the page shows has changed".
#[derive(Default)]
pub struct Watch {
    last: Option<String>,
}

impl Watch {
    pub fn new() -> Self {
        Self::default()
    }

    /// One round: wait on the feed (up to `tick`), recompute the signature, and return it only
    /// when it differs from the last one returned. The first call always returns it.
    pub fn step(&mut self, feed: &(impl ChangeFeed + ?Sized), store: &impl ProjectStore, manual_stamp: &str,
                sessions: &impl SessionProbe, tick: Duration) -> Option<String> {
        feed.wait(tick);
        let sig = signature(store, manual_stamp, sessions);
        if self.last.as_deref() == Some(sig.as_str()) {
            return None;
        }
        self.last = Some(sig.clone());
        Some(sig)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::*;
    use std::cell::{Cell, RefCell};
    use std::collections::BTreeMap;
    use std::time::Duration;

    struct Feed { answers: RefCell<Vec<bool>>, asked: RefCell<Vec<Duration>> }
    impl ChangeFeed for Feed {
        fn wait(&self, timeout: Duration) -> bool {
            self.asked.borrow_mut().push(timeout);
            self.answers.borrow_mut().pop().unwrap_or(false)
        }
    }
    fn feed(a: &[bool]) -> Feed { Feed { answers: RefCell::new(a.iter().rev().copied().collect()), asked: RefCell::new(vec![]) } }
    struct Store(RefCell<String>);
    impl ProjectStore for Store {
        fn projects(&self) -> Result<Vec<ProjectFiles>, String> { Ok(vec![]) }
        fn stamp(&self) -> String { self.0.borrow().clone() }
    }
    struct Sess(Cell<usize>);
    impl SessionProbe for Sess {
        fn open_sessions(&self) -> Option<BTreeMap<String, usize>> { Some(BTreeMap::from([("a".into(), self.0.get())])) }
    }
    const TICK: Duration = Duration::from_secs(5);

    #[test]
    fn first_step_returns_signature() {
        let mut w = Watch::new();
        assert!(w.step(&feed(&[]), &Store(RefCell::new("s".into())), "m", &Sess(Cell::new(0)), TICK).is_some());
    }

    #[test]
    fn unchanged_returns_none() {
        let (mut w, s, p) = (Watch::new(), Store(RefCell::new("s".into())), Sess(Cell::new(0)));
        let f = feed(&[true, true]);
        assert!(w.step(&f, &s, "m", &p, TICK).is_some());
        assert_eq!(w.step(&f, &s, "m", &p, TICK), None);
    }

    #[test]
    fn store_change_returns_new_sig() {
        let (mut w, s, p) = (Watch::new(), Store(RefCell::new("s".into())), Sess(Cell::new(0)));
        let f = feed(&[true, true]);
        let first = w.step(&f, &s, "m", &p, TICK).unwrap();
        *s.0.borrow_mut() = "s2".into();
        let next = w.step(&f, &s, "m", &p, TICK).unwrap();
        assert_ne!(first, next);
    }

    #[test]
    fn session_change_without_file_event_returns_new_sig() {
        let (mut w, s, p) = (Watch::new(), Store(RefCell::new("s".into())), Sess(Cell::new(0)));
        let f = feed(&[false, false]);
        w.step(&f, &s, "m", &p, TICK).unwrap();
        p.0.set(1);
        assert!(w.step(&f, &s, "m", &p, TICK).is_some());
    }

    #[test]
    fn feed_timeout_is_passed_through() {
        let f = feed(&[]);
        Watch::new().step(&f, &Store(RefCell::new(String::new())), "m", &Sess(Cell::new(0)), Duration::from_millis(1234));
        assert_eq!(*f.asked.borrow(), vec![Duration::from_millis(1234)]);
    }
}
