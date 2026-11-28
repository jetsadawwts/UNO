#!/usr/bin/env python3
"""PreToolUse hook for Bash: a shell command may not read a file that the Read deny rules protect.

Claude Code's `Read(...)` deny rules stop the Read tool, not `cat`, `grep`, `head`, `python -c`
or `git show HEAD:path` run through Bash. This hook closes that gap: it collects the project's
`Read(...)` deny patterns (from .claude/settings.local.json and .claude/settings.json, plus a
built-in list), finds every path-like word in the command, and blocks the command when one of
them names a protected file. Git pathspec exclusions (':!path') are ignored, and example files
(.env.example, *.sample, *.template) are allowed.

Known limit: a recursive search with no file named (`grep -r KEY .`) is not caught; use the Grep
tool, which honours the Read deny rules. Active only in onboarded projects. Never blocks on its
own bug.
"""
import fnmatch
import glob
import json
import os
import re
import sys
from datetime import datetime, timezone

DEFAULT_PATTERNS = [
    "./.env", "./.env.*", "./**/.env", "./**/.env.*", "./**/secrets/**", "./**/*.pem", "./**/*.key",
    "./**/*.ini", "./**/*.p12", "./**/*credentials*", "./**/kubeconfig*", "./**/*.tfstate*",
    "./**/*.tfvars", "~/.kube/**", "~/.aws/**", "~/.docker/config.json", "~/.ssh/**",
]
EXAMPLE_SUFFIXES = (".example", ".sample", ".template", ".dist")
EXCLUDE_PATHSPEC = re.compile(r"""['"]?:(?:!|\^|\(exclude\))[^\s'"]*['"]?""")
WORD = re.compile(r"""[^\s'"`;|&<>()=,]+""")


def deny_patterns(root):
    pats = list(DEFAULT_PATTERNS)
    for name in ("settings.local.json", "settings.json"):
        try:
            data = json.load(open(os.path.join(root, ".claude", name), encoding="utf-8"))
        except (OSError, ValueError):
            continue
        for rule in (data.get("permissions") or {}).get("deny") or []:
            m = re.fullmatch(r"Read\((.+)\)", str(rule).strip())
            if m:
                pats.append(m.group(1))
    return pats


def matches(path_abs, root, pattern):
    if pattern.startswith("./"):
        rel = os.path.relpath(path_abs, root)
        if rel.startswith(".."):
            return False
        rp = pattern[2:]
        cands = [rp] + ([rp[3:]] if rp.startswith("**/") else [])
        return any(fnmatch.fnmatch(rel, c) or fnmatch.fnmatch(os.path.basename(rel), c) and "/" not in c
                   for c in cands)
    if pattern.startswith("//"):
        ap = pattern[1:]
    elif pattern.startswith("~"):
        ap = os.path.expanduser(pattern)
    elif pattern.startswith("/"):
        ap = pattern
    else:
        return fnmatch.fnmatch(os.path.basename(path_abs), pattern)
    if ap.endswith("/**"):
        base = ap[:-3]
        return path_abs == base or path_abs.startswith(base + "/")
    return fnmatch.fnmatch(path_abs, ap)


def candidates(command, cwd):
    command = EXCLUDE_PATHSPEC.sub(" ", command)
    for word in WORD.findall(command):
        if word.startswith("-") and "=" not in word:
            continue
        parts = [word]
        if ":" in word and not word.startswith(("http://", "https://")):
            parts.append(word.split(":", 1)[1])
        for p in parts:
            if not p or p.startswith("$"):
                continue
            full = os.path.normpath(os.path.join(cwd, os.path.expanduser(p)))
            yield p, full
            if any(ch in p for ch in "*?["):
                for hit in glob.glob(full)[:200]:
                    yield hit, os.path.normpath(hit)


def check(root, cwd, command):
    pats = deny_patterns(root)
    for shown, full in candidates(command, cwd):
        if full.endswith(EXAMPLE_SUFFIXES):
            continue
        for pat in pats:
            if matches(full, root, pat):
                return (f"UNO: blocked — this command reads '{shown}', which the project protects "
                        f"(Read deny rule {pat}). Secrets never go into the chat or a model. "
                        f"If the owner needs a value checked, ask them to look themselves.")
    return None


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
    if event.get("tool_name") != "Bash":
        return 0
    root = os.environ.get("CLAUDE_PROJECT_DIR") or event.get("cwd") or os.getcwd()
    if not os.path.exists(os.path.join(root, ".uno", "zones")):
        return 0
    cwd = event.get("cwd") or root
    command = (event.get("tool_input") or {}).get("command", "")
    try:
        reason = check(root, cwd, command)
    except Exception:
        return 0
    if not reason:
        return 0
    try:
        with open(os.path.join(root, ".uno", "audit.log"), "a", encoding="utf-8") as fh:
            fh.write(json.dumps({"time": datetime.now(timezone.utc).isoformat(timespec="seconds"),
                                 "session": event.get("session_id"), **who(event), "tool": "Bash", "command": command[:200],
                                 "action": "blocked", "rule": "secrets"}, ensure_ascii=False) + "\n")
    except OSError:
        pass
    print(reason, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main())
