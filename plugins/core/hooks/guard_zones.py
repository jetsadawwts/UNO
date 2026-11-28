#!/usr/bin/env python3
"""PreToolUse hook: block agent edits in forbidden zones and log edits in review zones.

Reads `.uno/zones` in the project root. No zones file → allow everything (project not onboarded).
In an onboarded project the main thread (Uno) may edit only LEAD_FILES; members are told apart by
the `agent_id` Claude Code sends for subagent calls.
Exit 2 blocks the tool call and shows stderr to Claude; exit 0 allows it.
"""
import json
import os
import sys
from datetime import datetime, timezone
from fnmatch import fnmatch

EDIT_TOOLS = {"Edit", "Write", "MultiEdit", "NotebookEdit"}

# Uno (the main thread) plans and delegates: it may edit only UNO's own files. Code, tests, UI,
# infrastructure and docs go to a team member (Mu for code). UNO_LEAD_EDITS=allow turns this off.
LEAD_FILES = (".uno/**", "CLAUDE.md", ".claude/settings.local.json",
              "docs/intent.md", "docs/spec.md", "docs/refactor/**")


def load_zones(root):
    zones = {"forbidden": [], "review": []}
    path = os.path.join(root, ".uno", "zones")
    if not os.path.exists(path):
        return None
    section = None
    with open(path, encoding="utf-8") as fh:
        for raw in fh:
            line = raw.strip()
            if not line or line.startswith("#"):
                continue
            if line.startswith("[") and line.endswith("]"):
                section = line[1:-1].strip().lower()
                continue
            if section in zones:
                zones[section].append(line)
    return zones


def matches(rel, pattern):
    if fnmatch(rel, pattern):
        return True
    # let "**/x/**" also match "x/..." at the project root
    return pattern.startswith("**/") and fnmatch(rel, pattern[3:])


def audit(root, entry):
    try:
        os.makedirs(os.path.join(root, ".uno"), exist_ok=True)
        with open(os.path.join(root, ".uno", "audit.log"), "a", encoding="utf-8") as fh:
            fh.write(json.dumps(entry, ensure_ascii=False) + "\n")
    except OSError:
        pass  # logging must never break the session


def who(event):
    """Who made this call, names only: the subagent type (e.g. core:developer), or "uno" for the main thread."""
    kind = event.get("agent_type")
    if event.get("agent_id") and isinstance(kind, str) and kind:
        return {"name": kind[:64]}
    return {"name": "uno"}


def main():
    try:
        event = json.load(sys.stdin)
    except json.JSONDecodeError:
        return 0
    tool = event.get("tool_name", "")
    if tool not in EDIT_TOOLS:
        return 0
    params = event.get("tool_input", {}) or {}
    target = params.get("file_path") or params.get("notebook_path")
    if not target:
        return 0

    root = os.environ.get("CLAUDE_PROJECT_DIR") or event.get("cwd") or os.getcwd()
    zones = load_zones(root)
    if zones is None:
        return 0

    abs_target = os.path.realpath(os.path.join(root, target))
    abs_root = os.path.realpath(root)
    if not abs_target.startswith(abs_root + os.sep):
        return 0  # outside the project; other policies handle that
    rel = os.path.relpath(abs_target, abs_root).replace(os.sep, "/")

    entry = {"time": datetime.now(timezone.utc).isoformat(timespec="seconds"),
             "session": event.get("session_id"), **who(event), "tool": tool, "file": rel}

    for pattern in zones["forbidden"]:
        if matches(rel, pattern):
            audit(root, {**entry, "zone": "forbidden", "action": "blocked", "rule": pattern})
            print(f"UNO: '{rel}' is in a forbidden zone ({pattern}). Agents may not edit it. "
                  "Describe the change needed and ask a human to make it.", file=sys.stderr)
            return 2

    lead = not event.get("agent_id") and os.environ.get("UNO_LEAD_EDITS", "") != "allow"
    if lead and not any(fnmatch(rel, p) for p in LEAD_FILES):
        audit(root, {**entry, "zone": "lead", "action": "blocked", "rule": "lead-delegates"})
        print(f"UNO: Uno plans and delegates; it does not edit '{rel}'. Send this step to "
              "Mu (core:developer), or to the member who owns this kind of file: Delta tests, "
              "Gamma UI, Iota infrastructure, Kappa database, Theta docs, Lambda ADRs.", file=sys.stderr)
        return 2

    zone = "normal"
    for pattern in zones["review"]:
        if matches(rel, pattern):
            zone = "review"
            break
    audit(root, {**entry, "zone": zone, "action": "allowed"})
    return 0


if __name__ == "__main__":
    sys.exit(main())
