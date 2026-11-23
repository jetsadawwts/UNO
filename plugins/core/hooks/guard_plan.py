#!/usr/bin/env python3
"""PreToolUse hook: no code edit without a plan the owner approved.

Active only in onboarded projects (`.uno/zones` exists). For Edit/Write/MultiEdit/NotebookEdit:
  - edits inside `.uno/` are allowed, except adding an `Approved-by:` line to a plan
    (only the owner approves, with scripts/approve_plan.py)
  - any other edit needs the current branch to carry a task id (e.g. fix/T7-slug) and
    `.uno/plans/<task-id>.md` to end with an `Approved-by:` line whose sha256 matches the plan text.
    Changing the plan after approval voids the approval.
Exit 2 blocks and tells Claude what to do. Never blocks on its own errors.
"""
import hashlib
import json
import os
import re
import subprocess
import sys
from datetime import datetime, timezone

EDIT_TOOLS = {"Edit", "Write", "MultiEdit", "NotebookEdit"}
TASK_RE = re.compile(r"(?:^|[/_-])(T\d+)(?:[/_-]|$)", re.IGNORECASE)
APPROVAL = "Approved-by:"


def plan_hash(text):
    body = "\n".join(l for l in text.splitlines() if not l.startswith(APPROVAL)).strip()
    return hashlib.sha256(body.encode("utf-8")).hexdigest()[:12]


def approval_status(plan_path):
    """Return None if approved and current, else a reason."""
    try:
        text = open(plan_path, encoding="utf-8").read()
    except OSError:
        return "missing"
    lines = [l for l in text.splitlines() if l.startswith(APPROVAL)]
    if not lines:
        return "not approved"
    m = re.search(r"sha256:([0-9a-f]{12})", lines[-1])
    if not m or m.group(1) != plan_hash(text):
        return "changed after approval"
    return None


def branch(root):
    try:
        out = subprocess.run(["git", "-C", root, "rev-parse", "--abbrev-ref", "HEAD"],
                             capture_output=True, text=True, timeout=5)
        return out.stdout.strip()
    except (OSError, subprocess.SubprocessError):
        return ""


def inside(path, folder):
    try:
        return os.path.commonpath([os.path.realpath(path), os.path.realpath(folder)]) == os.path.realpath(folder)
    except ValueError:
        return False


def new_text(tool_input):
    parts = [tool_input.get(k, "") for k in ("content", "new_string", "new_source")]
    for e in tool_input.get("edits", []) or []:
        parts.append(e.get("new_string", ""))
    return "\n".join(p for p in parts if isinstance(p, str))


def check(root, tool_input):
    target = tool_input.get("file_path") or tool_input.get("notebook_path") or ""
    if not target:
        return None
    if not os.path.isabs(target):
        target = os.path.join(root, target)
    uno = os.path.join(root, ".uno")
    if inside(target, uno):
        if inside(target, os.path.join(uno, "plans")) and APPROVAL in new_text(tool_input):
            return ("UNO: only the owner approves a plan. Show the plan and ask the owner to run "
                    "`! python3 ~/UNO/uno/scripts/approve_plan.py <task-id>`.")
        return None
    if not inside(target, root):
        return None
    br = branch(root)
    m = TASK_RE.search(br)
    if not m:
        return (f"UNO: branch '{br}' has no task id. Start the change with /core:work <task-id> "
                f"on a branch like fix/T7-short-name, write .uno/plans/<task-id>.md, and get it approved.")
    task = m.group(1).upper()
    status = approval_status(os.path.join(uno, "plans", f"{task}.md"))
    if status is None:
        return None
    return (f"UNO: plan for {task} is {status}. Write or update .uno/plans/{task}.md "
            f"(template: work/templates/plan.md), show it to the owner, and stop until they run "
            f"`! python3 ~/UNO/uno/scripts/approve_plan.py {task}`. No code edits before that.")


def main():
    try:
        event = json.load(sys.stdin)
    except json.JSONDecodeError:
        return 0
    if event.get("tool_name") not in EDIT_TOOLS:
        return 0
    root = os.environ.get("CLAUDE_PROJECT_DIR") or event.get("cwd") or os.getcwd()
    if not os.path.exists(os.path.join(root, ".uno", "zones")):
        return 0
    try:
        reason = check(root, event.get("tool_input") or {})
    except Exception:  # never block on our own bug
        return 0
    if not reason:
        return 0
    try:
        with open(os.path.join(root, ".uno", "audit.log"), "a", encoding="utf-8") as fh:
            fh.write(json.dumps({"time": datetime.now(timezone.utc).isoformat(timespec="seconds"),
                                 "session": event.get("session_id"), "tool": event.get("tool_name"),
                                 "file": (event.get("tool_input") or {}).get("file_path", ""),
                                 "action": "blocked", "rule": "plan-gate"}, ensure_ascii=False) + "\n")
    except OSError:
        pass
    print(reason, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main())
