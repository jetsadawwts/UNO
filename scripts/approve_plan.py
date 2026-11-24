#!/usr/bin/env python3
"""Owner approves a task plan: appends `Approved-by: <name> <time> sha256:<hash>` to .uno/plans/<task>.md.

Run from the project folder, in a terminal or in Claude Code with the `!` prefix:
    ! python3 ~/UNO/uno/scripts/approve_plan.py T7
The hash covers the plan text, so any later change to the plan voids this approval.
"""
import hashlib
import os
import subprocess
import sys
from datetime import datetime

APPROVAL = "Approved-by:"


def main():
    if len(sys.argv) != 2:
        print("usage: approve_plan.py <task-id>", file=sys.stderr)
        return 1
    task = sys.argv[1].upper()
    root = subprocess.run(["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True).stdout.strip() or os.getcwd()
    path = os.path.join(root, ".uno", "plans", f"{task}.md")
    if not os.path.exists(path):
        print(f"no plan at {path}", file=sys.stderr)
        return 1
    text = open(path, encoding="utf-8").read()
    body = "\n".join(l for l in text.splitlines() if not l.startswith(APPROVAL)).strip()
    required = ["Goal", "UI design", "Dependencies", "Files to change", "Steps", "Verification", "Rollback"]
    missing = [h for h in required if f"## {h}".lower() not in body.lower()]
    if missing:
        print(f"not approved: plan is missing sections: {', '.join(missing)}", file=sys.stderr)
        return 1
    print(body)
    print("\n" + "-" * 60)
    if sys.stdin.isatty() and input(f"Approve this plan for {task}? [y/N] ").strip().lower() != "y":
        print("not approved")
        return 1
    digest = hashlib.sha256(body.encode("utf-8")).hexdigest()[:12]
    name = subprocess.run(["git", "config", "user.name"], capture_output=True, text=True, cwd=root).stdout.strip() or "owner"
    with open(path, "w", encoding="utf-8") as fh:
        fh.write(body + f"\n\n{APPROVAL} {name} {datetime.now().isoformat(timespec='minutes')} sha256:{digest} via terminal backend:unknown\n")
    print(f"approved {task} (sha256:{digest})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
