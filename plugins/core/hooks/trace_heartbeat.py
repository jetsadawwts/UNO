#!/usr/bin/env python3
"""PreToolUse hook (every tool): note that the session is still working.

Overwrites .uno/heartbeat/<session>.json at most once a minute, so uno-report can show a session
as working while it only reads, plans or edits .uno files. Stores the time, session id, tool name
and agent id, never commands, paths or content. Kept out of audit.log so that log does not grow.
Only active in onboarded projects (`.uno/` exists). Prints nothing and never blocks.
"""
import json
import os
import re
import sys
from datetime import datetime, timezone

EVERY_SECS = 60
KEEP_SECS = 24 * 60 * 60
SAFE_ID = re.compile(r"[A-Za-z0-9_-]{1,128}")


def last_time(path):
    try:
        with open(path, encoding="utf-8") as fh:
            return datetime.fromisoformat(json.load(fh)["time"])
    except (OSError, ValueError, KeyError, TypeError):
        return None


def prune(folder, now):
    for entry in os.scandir(folder):
        if entry.name.endswith((".json", ".tmp")) and now.timestamp() - entry.stat().st_mtime > KEEP_SECS:
            os.remove(entry.path)


def main():
    try:
        event = json.load(sys.stdin)
        session = event.get("session_id")
        if not isinstance(session, str) or not SAFE_ID.fullmatch(session):
            return 0
        root = os.environ.get("CLAUDE_PROJECT_DIR") or event.get("cwd") or os.getcwd()
        uno = os.path.join(root, ".uno")
        if not os.path.isdir(uno):
            return 0
        folder = os.path.join(uno, "heartbeat")
        path = os.path.join(folder, session + ".json")
        now = datetime.now(timezone.utc)
        before = last_time(path)
        if before and 0 <= (now - before).total_seconds() < EVERY_SECS:
            return 0
        tool = str(event.get("tool_name") or "")
        params = event.get("tool_input") or {}
        agent = params.get("subagent_type") if tool in ("Agent", "Task") else event.get("agent_type")
        entry = {"time": now.isoformat(timespec="seconds"), "session": session, "action": "heartbeat",
                 "tool": tool[:80], "name": str(agent or "")[:80]}
        os.makedirs(folder, exist_ok=True)
        tmp = f"{path}.{os.getpid()}.tmp"
        with open(tmp, "w", encoding="utf-8") as fh:
            json.dump(entry, fh)
        os.replace(tmp, path)
        prune(folder, now)
    except Exception:  # a heartbeat must never break a tool call
        return 0
    return 0


if __name__ == "__main__":
    sys.exit(main())
