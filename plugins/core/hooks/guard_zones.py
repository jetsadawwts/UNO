#!/usr/bin/env python3
"""PreToolUse hook: block agent edits in forbidden zones and log edits in review zones.

Reads `.uno/zones` in the project root. No zones file → allow everything (project not onboarded).
Exit 2 blocks the tool call and shows stderr to Claude; exit 0 allows it.
"""
import json
import os
import sys
from datetime import datetime, timezone
from fnmatch import fnmatch

EDIT_TOOLS = {"Edit", "Write", "MultiEdit", "NotebookEdit"}


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
             "session": event.get("session_id"), "tool": tool, "file": rel}

    for pattern in zones["forbidden"]:
        if matches(rel, pattern):
            audit(root, {**entry, "zone": "forbidden", "action": "blocked", "rule": pattern})
            print(f"UNO: '{rel}' is in a forbidden zone ({pattern}). Agents may not edit it. "
                  "Describe the change needed and ask a human to make it.", file=sys.stderr)
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
