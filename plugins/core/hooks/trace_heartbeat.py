#!/usr/bin/env python3
"""Hook for the dashboard's live team activity: what is each session doing right now?

On every state change it overwrites .uno/heartbeat/<session>.json (the main thread, Uno) or, for
work inside a subagent, .uno/heartbeat/<session>.<agent_id>.json, with the time, session id, state,
since when, tool name, agent type, the task id from the branch name and the plugin version, so the
dashboard sees each team member at once. SubagentStart creates a member's file, SubagentStop (or,
for a subagent the main thread waited for, the Agent tool's PostToolUse) removes it, SessionEnd
removes every file of the session. Subagent start, each tool, stop, and the main thread getting an
Agent answer are also appended to .uno/office.log (time, session, agent id, agent type, step, tool
name) for the dashboard's hand-off timeline; past 1 MB only the newest 1,000 lines are kept. It never stores
commands, paths, prompts, messages, branch names or content. Kept out of audit.log so that log
does not grow. Only active in onboarded projects (`.uno/` exists).

It always prints nothing and exits 0, so as a PermissionRequest or PreToolUse hook it never
decides a permission and never blocks. Notification matchers pass the state as an argument.
"""
import json
import os
import re
import subprocess
import sys
from datetime import datetime, timezone

KEEP_SECS = 24 * 60 * 60
SAFE_ID = re.compile(r"[A-Za-z0-9_-]{1,128}")
TASK_RE = re.compile(r"(?:^|[/_-])(T\d+)(?:[/_-]|$)", re.IGNORECASE)  # same rule as guard_plan
STATES = {"UserPromptSubmit": "between", "PostToolUse": "between", "PostToolUseFailure": "between",
          "PreToolUse": "running", "PermissionRequest": "waiting", "Stop": "done", "StopFailure": "error"}
READ_TASK_ON = ("UserPromptSubmit", "PreToolUse")


def state_of(name, tool):
    if name == "PreToolUse" and tool == "AskUserQuestion":
        return "waiting"
    if name == "Notification":
        arg = sys.argv[1] if len(sys.argv) > 1 else ""
        return arg if arg in ("waiting", "done") else None
    return STATES.get(name)


def branch_task(root):
    try:
        r = subprocess.run(["git", "-C", root, "rev-parse", "--abbrev-ref", "HEAD"],
                           capture_output=True, text=True, timeout=2)
    except (OSError, subprocess.SubprocessError):
        return ""
    m = TASK_RE.search(r.stdout.strip()) if r.returncode == 0 else None
    return m.group(1).upper() if m else ""


def plugin_version():
    manifest = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".claude-plugin", "plugin.json")
    return str(previous(manifest).get("version") or "")[:20]


def previous(path):
    try:
        with open(path, encoding="utf-8") as fh:
            data = json.load(fh)
        return data if isinstance(data, dict) else {}
    except (OSError, ValueError):
        return {}


def prune(folder, now):
    for entry in os.scandir(folder):
        if entry.name.endswith((".json", ".tmp")) and now.timestamp() - entry.stat().st_mtime > KEEP_SECS:
            os.remove(entry.path)


def remove(path):
    try:
        os.remove(path)
    except OSError:
        pass


LOG_MAX_BYTES = 1024 * 1024
LOG_KEEP_LINES = 1000


def trail(uno, session, step, agent="", kind="", tool=""):
    """Append one step to .uno/office.log (names only)."""
    path = os.path.join(uno, "office.log")
    line = json.dumps({"time": datetime.now(timezone.utc).isoformat(timespec="seconds"), "action": "office",
                       "event": step, "session": session, "agent": agent, "name": str(kind or "")[:80],
                       "tool": str(tool or "")[:80]})
    with open(path, "a", encoding="utf-8") as fh:
        fh.write(line + "\n")
    if os.path.getsize(path) > LOG_MAX_BYTES:
        with open(path, encoding="utf-8") as fh:
            keep = fh.readlines()[-LOG_KEEP_LINES:]
        tmp = f"{path}.{os.getpid()}.tmp"
        with open(tmp, "w", encoding="utf-8") as fh:
            fh.writelines(keep)
        os.replace(tmp, path)


def agent_files(folder, session):
    """The session's subagent files: (path, stored entry)."""
    out = []
    if os.path.isdir(folder):
        for entry in os.scandir(folder):
            if entry.name.startswith(session + ".") and entry.name.endswith(".json") and entry.name != session + ".json":
                out.append((entry.path, previous(entry.path)))
    return out


def record(event):
    session = event.get("session_id")
    if not isinstance(session, str) or not SAFE_ID.fullmatch(session):
        return
    root = os.environ.get("CLAUDE_PROJECT_DIR") or event.get("cwd") or os.getcwd()
    uno = os.path.join(root, ".uno")
    if not os.path.isdir(uno):
        return
    folder = os.path.join(uno, "heartbeat")
    main_path = os.path.join(folder, session + ".json")
    name = event.get("hook_event_name")
    agent_id = event.get("agent_id")
    sub = isinstance(agent_id, str) and bool(SAFE_ID.fullmatch(agent_id))
    path = os.path.join(folder, f"{session}.{agent_id}.json") if sub else main_path
    if name == "SessionEnd":
        remove(main_path)
        for p, _ in agent_files(folder, session):
            remove(p)
        return
    if name == "SubagentStop":
        if sub:
            remove(path)
            trail(uno, session, "stop", agent_id, event.get("agent_type"))
        return
    tool = str(event.get("tool_name") or "")[:80]
    if not sub and name == "PostToolUse" and tool in ("Agent", "Task"):
        # The main thread got the subagent's answer: that member is done, unless it was only launched.
        response = event.get("tool_response")
        launched = isinstance(response, dict) and response.get("status") == "async_launched"
        kind = (event.get("tool_input") or {}).get("subagent_type")
        if not launched and kind:
            trail(uno, session, "return", "", kind)
            for p, data in agent_files(folder, session):
                if data.get("name") == str(kind)[:80]:
                    remove(p)
    if sub and name == "SubagentStart":
        trail(uno, session, "start", agent_id, event.get("agent_type"))
    elif sub and name == "PreToolUse":
        trail(uno, session, "tool", agent_id, event.get("agent_type"), tool)
    state = "between" if name == "SubagentStart" and sub else state_of(name, tool)
    if state is None:
        return
    old = previous(path)
    if not tool and state in ("between", "waiting"):
        tool = old.get("tool", "")
    if sub:
        agent = event.get("agent_type") or old.get("name")
    elif state in ("done", "error"):
        tool, agent = "", ""
    elif name == "Notification":
        agent = old.get("name")
    else:
        # Main thread is Uno. Only an old Claude Code without agent_id marks subagent work here.
        agent = event.get("agent_type")
    if state in ("done", "error"):
        tool = ""
    task = branch_task(root) if name in READ_TASK_ON else (old.get("task") or previous(main_path).get("task", ""))
    now = datetime.now(timezone.utc)
    stamp = now.isoformat(timespec="seconds")
    same = (old.get("state"), old.get("tool")) == (state, tool)
    entry = {"time": stamp, "session": session, "action": "heartbeat", "state": state,
             "since": old.get("since", stamp) if same else stamp,
             "tool": tool, "name": str(agent or "")[:80], "task": task, "version": plugin_version()}
    os.makedirs(folder, exist_ok=True)
    tmp = f"{path}.{os.getpid()}.tmp"
    with open(tmp, "w", encoding="utf-8") as fh:
        json.dump(entry, fh)
    os.replace(tmp, path)
    prune(folder, now)


def main():
    try:
        record(json.load(sys.stdin))
    except Exception:  # the dashboard must never break a session
        pass
    return 0


if __name__ == "__main__":
    sys.exit(main())
