/// One line of `audit.log`, as plain strings. Missing fields are empty.
#[derive(Default, Clone, Debug, PartialEq)]
pub struct AuditRecord {
    pub time: String,
    pub event: String,
    pub action: String,
    pub backend: String,
    pub task: String,
    pub rule: String,
    pub zone: String,
    pub tool: String,
    pub command: String,
    pub file: String,
    pub name: String,
    pub description: String,
    /// Claude Code session id, when the hook knew it.
    pub session: String,
    /// Heartbeat only: running, between, waiting, done or error, and since when.
    pub state: String,
    pub since: String,
}

/// What one session is doing now, from its heartbeat file.
#[derive(Clone, Debug, PartialEq)]
pub struct Live {
    pub session: String,
    pub time: String,
    pub since: String,
    pub state: String,
    pub who: String,
    pub tool: String,
    pub task: String,
}

/// What an activity-feed entry says happened. Text for people is made by the views.
#[derive(Clone, Debug, PartialEq)]
pub enum EventKind {
    PlanApproved { task: String },
    Smoke,
    Blocked { rule: String },
    AskedOwner { command: String },
    Edited { file: String },
    Agent { who: String, description: String },
    Skill { name: String },
    /// Latest tool use, from the heartbeat hook.
    Tool { who: String, tool: String },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    pub time: String,
    pub kind: EventKind,
}
