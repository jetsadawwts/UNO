#!/usr/bin/env python3
"""PreToolUse hook: record every skill and subagent call to .uno/audit.log.

Gives an after-the-fact trace of which flow a task followed (UNO skills, Superpowers skills,
Gamma, Epsilon ...). Only active in onboarded projects (`.uno/` exists). Never blocks.
"""
import json
import os
import sys
from datetime import datetime, timezone


def main():
    try:
        event = json.load(sys.stdin)
    except json.JSONDecodeError:
        return 0
    tool = event.get("tool_name", "")
    params = event.get("tool_input") or {}
    root = os.environ.get("CLAUDE_PROJECT_DIR") or event.get("cwd") or os.getcwd()
    uno = os.path.join(root, ".uno")
    if not os.path.isdir(uno):
        return 0
    if tool == "Skill":
        kind, name = "skill", params.get("skill") or params.get("command") or params.get("name")
    elif tool in ("Task", "Agent"):
        kind, name = "agent", params.get("subagent_type") or params.get("agent") or "general-purpose"
    else:
        return 0
    entry = {"time": datetime.now(timezone.utc).isoformat(timespec="seconds"),
             "session": event.get("session_id"), "tool": tool, "kind": kind, "name": name,
             "description": (params.get("description") or "")[:120], "action": "called"}
    try:
        with open(os.path.join(uno, "audit.log"), "a", encoding="utf-8") as fh:
            fh.write(json.dumps(entry, ensure_ascii=False) + "\n")
    except OSError:
        pass
    return 0


if __name__ == "__main__":
    sys.exit(main())
