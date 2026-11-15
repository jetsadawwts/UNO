#!/usr/bin/env python3
"""PreToolUse hook for Bash: keep agents off the default branch in onboarded projects.

Blocks (exit 2):
  - `git commit` while the current branch is the default branch
  - `git push` that targets the default branch, or any force push without --force-with-lease
  - `git merge` into the default branch
Only active when `.uno/zones` exists (project onboarded). Never blocks on its own errors.
"""
import json
import os
import re
import shlex
import subprocess
import sys
from datetime import datetime, timezone


def git(root, *args):
    try:
        out = subprocess.run(["git", "-C", root, *args], capture_output=True, text=True, timeout=5)
        return out.stdout.strip() if out.returncode == 0 else ""
    except (OSError, subprocess.SubprocessError):
        return ""


def default_branch(root):
    ref = git(root, "symbolic-ref", "--short", "refs/remotes/origin/HEAD")
    if ref.startswith("origin/"):
        return ref[len("origin/"):]
    for name in ("main", "master"):
        if git(root, "rev-parse", "--verify", "--quiet", f"refs/heads/{name}"):
            return name
    return "main"


def split_commands(command):
    # split on shell separators so "cd x && git commit" is checked
    return [part.strip() for part in re.split(r"&&|\|\||;|\n|\|", command) if part.strip()]


def tokens(part):
    try:
        return shlex.split(part)
    except ValueError:
        return part.split()


def git_args(tok):
    """Return args after `git` (skipping -C <dir> and -c k=v), or None if not a git call."""
    if not tok or os.path.basename(tok[0]) != "git":
        return None
    args, i = [], 1
    while i < len(tok):
        if tok[i] in ("-C", "-c") and i + 1 < len(tok):
            i += 2
            continue
        args = tok[i:]
        break
    return args


def check(root, command):
    base = default_branch(root)
    current = git(root, "rev-parse", "--abbrev-ref", "HEAD")
    for part in split_commands(command):
        args = git_args(tokens(part))
        if not args:
            continue
        sub, rest = args[0], args[1:]
        if sub == "commit" and current == base:
            return (f"UNO: you are on the default branch '{base}'. Create a task branch from "
                    f"origin/{base} first (see /core:work), then commit there.")
        if sub == "merge" and current == base:
            return f"UNO: agents do not merge into '{base}'. Open a PR; a human merges."
        if sub == "push":
            if any(a in ("--force", "-f") for a in rest) or any(a.startswith("+") for a in rest):
                return "UNO: plain force push is not allowed. Use --force-with-lease on your own task branch."
            targets = [a for a in rest if not a.startswith("-")]
            refspecs = targets[1:] if len(targets) > 1 else []
            if not refspecs and current == base:
                return f"UNO: this push would update '{base}'. Push a task branch and open a PR."
            for spec in refspecs:
                dest = spec.split(":")[-1].replace("refs/heads/", "")
                if dest in (base, "HEAD") and (dest != "HEAD" or current == base):
                    return f"UNO: pushing to '{base}' is not allowed. Push a task branch and open a PR."
    return None


def main():
    try:
        event = json.load(sys.stdin)
    except json.JSONDecodeError:
        return 0
    if event.get("tool_name") != "Bash":
        return 0
    command = (event.get("tool_input") or {}).get("command", "")
    if "git" not in command:
        return 0
    root = os.environ.get("CLAUDE_PROJECT_DIR") or event.get("cwd") or os.getcwd()
    if not os.path.exists(os.path.join(root, ".uno", "zones")):
        return 0
    reason = check(root, command)
    if not reason:
        return 0
    try:
        with open(os.path.join(root, ".uno", "audit.log"), "a", encoding="utf-8") as fh:
            fh.write(json.dumps({"time": datetime.now(timezone.utc).isoformat(timespec="seconds"),
                                 "session": event.get("session_id"), "tool": "Bash",
                                 "command": command[:200], "action": "blocked",
                                 "rule": "git-guard"}, ensure_ascii=False) + "\n")
    except OSError:
        pass
    print(reason, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main())
