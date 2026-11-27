#!/usr/bin/env python3
"""Hook for the dashboard's live team activity: what is each session doing right now?

On every state change it overwrites .uno/heartbeat/<session>.json with the time, session id,
state, since when, tool name, agent id, the task id from the branch name and the plugin version. It never stores
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


def record(event):
    session = event.get("session_id")
    if not isinstance(session, str) or not SAFE_ID.fullmatch(session):
        return
    root = os.environ.get("CLAUDE_PROJECT_DIR") or event.get("cwd") or os.getcwd()
    uno = os.path.join(root, ".uno")
    if not os.path.isdir(uno):
        return
    folder = os.path.join(uno, "heartbeat")
    path = os.path.join(folder, session + ".json")
    name = event.get("hook_event_name")
    if name == "SessionEnd":
        if os.path.exists(path):
            os.remove(path)
        return
    tool = str(event.get("tool_name") or "")[:80]
    state = state_of(name, tool)
    if state is None:
        return
    old = previous(path)
    if not tool and state in ("between", "waiting"):
        tool = old.get("tool", "")
    if state in ("done", "error"):
        tool, agent = "", ""
    elif tool in ("Agent", "Task"):
        agent = (event.get("tool_input") or {}).get("subagent_type")
    elif name == "Notification":
        agent = old.get("name")
    else:
        agent = event.get("agent_type")
    task = branch_task(root) if name in READ_TASK_ON else old.get("task", "")
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
