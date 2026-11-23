#!/usr/bin/env python3
"""UserPromptSubmit hook: the owner approves a plan by typing `approve T7` (or `อนุมัติ T7`).

Only a human can submit a prompt, so an agent cannot trigger this. The hook appends
`Approved-by: <git user.name> <time> via prompt sha256:<hash>` to .uno/plans/<task>.md
(the same format guard_plan checks) and tells Claude the result. Other prompts pass through.
Active only in onboarded projects (`.uno/zones` exists).
"""
import hashlib
import json
import os
import re
import subprocess
import sys
from datetime import datetime, timezone

APPROVAL = "Approved-by:"
PATTERN = re.compile(r"^\s*(?:approve|อนุมัติ)\s+(T\d+)\s*$", re.IGNORECASE)
REQUIRED = ["## Goal", "## UI design", "## Dependencies", "## Files to change", "## Steps",
            "## Verification", "## Rollback"]


def say(text):
    print(json.dumps({"hookSpecificOutput": {"hookEventName": "UserPromptSubmit",
                                             "additionalContext": text}}, ensure_ascii=False))


def main():
    try:
        event = json.load(sys.stdin)
    except json.JSONDecodeError:
        return 0
    m = PATTERN.match(event.get("prompt", ""))
    if not m:
        return 0
    root = os.environ.get("CLAUDE_PROJECT_DIR") or event.get("cwd") or os.getcwd()
    if not os.path.exists(os.path.join(root, ".uno", "zones")):
        return 0
    task = m.group(1).upper()
    path = os.path.join(root, ".uno", "plans", f"{task}.md")
    if not os.path.exists(path):
        say(f"UNO: no plan at .uno/plans/{task}.md, nothing approved. Write the plan first.")
        return 0
    text = open(path, encoding="utf-8").read()
    body = "\n".join(l for l in text.splitlines() if not l.startswith(APPROVAL)).strip()
    missing = [h[3:] for h in REQUIRED if h.lower() not in body.lower()]
    if missing:
        say(f"UNO: plan {task} NOT approved — missing sections: {', '.join(missing)}. "
            f"Add them, show the plan again and wait for the owner.")
        return 0
    digest = hashlib.sha256(body.encode("utf-8")).hexdigest()[:12]
    try:
        name = subprocess.run(["git", "-C", root, "config", "user.name"], capture_output=True,
                              text=True, timeout=5).stdout.strip() or "owner"
    except (OSError, subprocess.SubprocessError):
        name = "owner"
    stamp = datetime.now(timezone.utc).isoformat(timespec="minutes")
    with open(path, "w", encoding="utf-8") as fh:
        fh.write(body + f"\n\n{APPROVAL} {name} {stamp} via prompt sha256:{digest}\n")
    try:
        with open(os.path.join(root, ".uno", "audit.log"), "a", encoding="utf-8") as fh:
            fh.write(json.dumps({"time": stamp, "session": event.get("session_id"),
                                 "event": "plan-approved", "task": task, "sha256": digest},
                                ensure_ascii=False) + "\n")
    except OSError:
        pass
    say(f"UNO: the owner approved plan {task} (sha256:{digest}). Continue the task now, inside the plan.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
