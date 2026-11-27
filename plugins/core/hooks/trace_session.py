#!/usr/bin/env python3
"""SessionStart / SessionEnd hook: record when a Claude Code session opens and closes.

Writes one line to .uno/audit.log so uno-report can show which projects have a session open
without looking at other processes on the machine. Only active in onboarded projects
(`.uno/` exists). Never blocks.
"""
import json
import os
import sys
from datetime import datetime, timezone

ACTIONS = {"SessionStart": "session-start", "SessionEnd": "session-end"}


def main():
    try:
        event = json.load(sys.stdin)
    except (json.JSONDecodeError, ValueError):
        return 0
    action = ACTIONS.get(event.get("hook_event_name", ""))
    if not action:
        return 0
    root = os.environ.get("CLAUDE_PROJECT_DIR") or event.get("cwd") or os.getcwd()
    uno = os.path.join(root, ".uno")
    if not os.path.isdir(uno):
        return 0
    entry = {"time": datetime.now(timezone.utc).isoformat(timespec="seconds"),
             "session": event.get("session_id"), "action": action}
    if action == "session-start":
        entry["source"] = event.get("source", "")
    else:
        entry["reason"] = event.get("reason", "")
    try:
        with open(os.path.join(uno, "audit.log"), "a", encoding="utf-8") as fh:
            fh.write(json.dumps(entry, ensure_ascii=False) + "\n")
    except OSError:
        pass
    return 0


if __name__ == "__main__":
    sys.exit(main())
