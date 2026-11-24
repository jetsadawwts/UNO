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


INSTALL_RE = re.compile(r"\b(npm\s+(install|i|add|uninstall|remove)|pnpm\s+(add|install|remove)|yarn\s+(add|remove)|pip3?\s+install|dotnet\s+add\s+\S*\s*package)\b")


SAFE_WITH_INSTALL = (
    "cd", "pwd", "ls", "cat", "grep", "echo", "mkdir", "test", "true",
    "npm run", "npm test", "npm ci", "npm ls", "npm audit", "npm outdated", "npm view",
    "npx tsc", "npx vitest", "npx eslint", "npx prettier",
    "git status", "git diff", "git log", "git add", "git commit", "git show",
    "pip list", "pip show", "python3 -m pytest", "pytest", "dotnet build", "dotnet test", "dotnet restore",
)
HIDDEN = ("`", "$(", "<(", ">(", "| sh", "| bash", "|sh", "|bash", "eval ")


def _packages(part):
    import shlex
    try:
        tok = shlex.split(part)
    except ValueError:
        return None
    return [t for t in tok[2:] if not t.startswith("-") and t not in ("package", "add")]


def _plan_deps(root):
    m = TASK_RE.search(branch(root))
    if not m:
        return None, "UNO: package changes only happen inside a task (/core:work) with an approved plan."
    task = m.group(1).upper()
    plan = os.path.join(root, ".uno", "plans", f"{task}.md")
    status = approval_status(plan)
    if status is not None:
        return None, f"UNO: plan for {task} is {status}; no package changes before the owner approves it."
    deps, on = [], False
    for line in open(plan, encoding="utf-8").read().splitlines():
        if line.startswith("## "):
            on = line.lower().startswith("## dependencies")
        elif on:
            deps.append(line.lower())
    return (task, "\n".join(deps)), None


def _bare(p):
    n = p.rsplit("@", 1)[0] if p.rfind("@") > 0 else p
    return re.split(r"[=<>~!]", n)[0].lower()


def install_check(root, cmd):
    """Installs need an approved plan listing every package. A chained command is allowed
    only if every other part is a known, harmless command; anything unfamiliar is blocked."""
    if not INSTALL_RE.search(cmd):
        return None
    if any(h in cmd for h in HIDDEN):
        return "UNO: install chained with a hidden or piped command (backticks, $(), | sh, eval) is blocked."
    parts = [p.strip() for p in re.split(r"&&|\|\||;|\n", cmd) if p.strip()]
    if any("|" in p for p in parts):
        return "UNO: install chained with a pipe is blocked; run the install as its own command."
    info = None
    for part in parts:
        if INSTALL_RE.search(part):
            pkgs = _packages(part)
            if pkgs is None:
                return "UNO: could not read this install command; run it as a plain command."
            if not pkgs:
                continue  # npm install / npm ci from the lock file
            if info is None:
                info, err = _plan_deps(root)
                if err:
                    return err
            task, text = info
            missing = [p for p in pkgs if _bare(p) not in text]
            if missing:
                return (f"UNO: {', '.join(missing)} not in the Dependencies of the approved plan {task}. "
                        f"This is a PLAN CHANGE: ask the owner, update the plan, get it re-approved.")
        elif not part.startswith(SAFE_WITH_INSTALL):
            return (f"UNO: '{part[:60]}' is not a known safe command to chain with an install. "
                    f"Run the install on its own, and the other command separately.")
    return None


WRITE_CMDS = re.compile(r"(^|[;&|]\s*|\s)(sed\s+(-[a-zA-Z]*i|--in-place)|perl\s+-[a-zA-Z]*i|tee\b|cp\b|mv\b|rm\b|touch\b|truncate\b|git\s+apply\b|patch\b|git\s+restore\b|git\s+checkout\s+\S*\s*--\s)")
REDIRECT = re.compile(r"(?<![0-9&>])>{1,2}\s*([^\s;&|]+)")


def writes_files(cmd):
    """True if a shell command looks like it writes or deletes files outside .uno/ and /tmp."""
    if WRITE_CMDS.search(cmd):
        return True
    for target in REDIRECT.findall(cmd):
        if target in ("/dev/null", "/dev/stderr", "/dev/stdout"):
            continue
        if target.startswith((".uno/", "/tmp/", "$TMPDIR")) or "/.uno/" in target:
            continue
        return True
    return False


def bash_write_check(root, cmd):
    if not writes_files(cmd):
        return None
    m = TASK_RE.search(branch(root))
    if not m:
        return ("UNO: this shell command writes files, and the branch has no task id. "
                "Start with /core:work <task-id>; use the Edit/Write tools for code changes.")
    task = m.group(1).upper()
    status = approval_status(os.path.join(root, ".uno", "plans", f"{task}.md"))
    if status is not None:
        return (f"UNO: plan for {task} is {status}; no file changes (shell or editor) before the owner approves it.")
    return None


def main():
    try:
        event = json.load(sys.stdin)
    except json.JSONDecodeError:
        return 0
    tool = event.get("tool_name")
    if tool not in EDIT_TOOLS and tool != "Bash":
        return 0
    root = os.environ.get("CLAUDE_PROJECT_DIR") or event.get("cwd") or os.getcwd()
    if not os.path.exists(os.path.join(root, ".uno", "zones")):
        return 0
    if tool == "Bash":
        cmd = (event.get("tool_input") or {}).get("command", "")
        why = install_check(root, cmd) or bash_write_check(root, cmd)
        if why:
            print(why, file=sys.stderr)
            return 2
        if "approve_plan" in cmd or "Approved-by" in cmd:
            print("UNO: only the owner approves a plan. Show the plan, stop, and ask the owner to run "
                  "`! python3 ~/UNO/uno/scripts/approve_plan.py <task-id>` themselves. Do not run it.",
                  file=sys.stderr)
            try:
                with open(os.path.join(root, ".uno", "audit.log"), "a", encoding="utf-8") as fh:
                    fh.write(json.dumps({"time": datetime.now(timezone.utc).isoformat(timespec="seconds"),
                                         "session": event.get("session_id"), "tool": "Bash",
                                         "command": cmd[:200], "action": "blocked",
                                         "rule": "self-approval"}, ensure_ascii=False) + "\n")
            except OSError:
                pass
            return 2
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
