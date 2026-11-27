//! Operating-system adapters: local time.
use std::process::Command;

use app::ports::Clock;

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

    #[test]
    fn clock_has_date_and_minutes() {
        let n = SystemClock.now();
        assert_eq!(n.len(), 16, "{n}");
        assert_eq!((&n[4..5], &n[10..11], &n[13..14]), ("-", " ", ":"));
    }
}
