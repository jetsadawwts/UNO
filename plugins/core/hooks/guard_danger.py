#!/usr/bin/env python3
"""PreToolUse hook for Bash: deleting is fine as long as it can be undone; the system and the OS are the owner's.

Inside the project, destructive commands run without a prompt, but first this hook keeps a way
back and writes it to `.uno/recovery.md`:
  - files about to be deleted, discarded or cleaned are copied to `.uno/trash/<time>/`
  - the commit, branch, stash or remote ref about to be moved or deleted is recorded by sha
Commands that reach the machine outside the project (files outside the project and /tmp, sudo,
global package removal, OS settings, services) or data no snapshot can hold (SQL DROP/DELETE,
migration reset) always ask the owner. Files outside the project are copied to the trash first
when they fit, so the prompt can say how to restore them. If a snapshot cannot be made, the
command asks instead of running. Shared and production systems stay blocked by guard_infra and
the deny list. Trash folders older than 30 days are removed. Active only in onboarded projects. Never blocks on its own bug.
"""
import json
import os
import re
import shlex
import shutil
import subprocess
import sys
from datetime import datetime, timezone

MAX_SNAPSHOT = 200 * 1024 * 1024
KEEP_DAYS = 30
SCRATCH = ("/tmp/", "/private/tmp/")
CACHE_DIRS = ("node_modules", "dist", "build", ".next", "coverage", "target", "__pycache__",
              ".pytest_cache", ".turbo", ".cache", ".parcel-cache", "out")
DELETE_TOOLS = ("rm", "rmdir", "unlink", "trash")

ASK_ALWAYS = [
    (re.compile(r"(^|[;&|]\s*)sudo\b"), "ใช้ sudo แก้ระดับระบบ"),
    (re.compile(r"\bshred\b"), "shred ลบแบบกู้คืนไม่ได้"),
    (re.compile(r"\bbrew\s+(uninstall|remove|rm|cleanup|untap)\b"), "ถอนโปรแกรมออกจากเครื่อง (brew)"),
    (re.compile(r"\b(npm|pnpm|yarn)\s+(uninstall|remove|rm|un)\s+(.*\s)?(-g|--global)\b|\byarn\s+global\s+remove\b"),
     "ถอน package ระดับเครื่อง"),
    (re.compile(r"\b(pip3?|python3?\s+-m\s+pip)\s+uninstall\b"), "ถอน Python package (อาจเป็นของทั้งเครื่อง)"),
    (re.compile(r"\b(launchctl\s+(unload|remove|bootout|disable)|systemctl\s+(stop|disable|mask)|"
                r"defaults\s+(delete|write)|crontab\s+-r|diskutil|csrutil|spctl|tmutil\s+delete)\b"),
     "แก้ service หรือการตั้งค่าของ OS"),
    (re.compile(r"\b(chmod|chown)\s+-R\s+\S+\s+(/|~)"), "เปลี่ยนสิทธิ์ไฟล์นอกโปรเจกต์"),
    (re.compile(r"\b(DROP|TRUNCATE)\s+(TABLE|DATABASE|SCHEMA)\b|\bDELETE\s+FROM\b", re.I),
     "ลบข้อมูลในฐานข้อมูล (snapshot ให้ไม่ได้)"),
    (re.compile(r"\b(prisma\s+migrate\s+reset|db:drop|db:reset|migrate:fresh|migrate:reset|alembic\s+downgrade|"
                r"rails\s+db:rollback|sequelize\s+db:migrate:undo|knex\s+migrate:rollback|dotnet\s+ef\s+database\s+drop)\b"),
     "ล้างหรือย้อนฐานข้อมูล (snapshot ให้ไม่ได้)"),
    (re.compile(r"\b(npm|pnpm|yarn)\s+(unpublish|deprecate)\b|\bcargo\s+yank\b"), "ถอน package ที่เผยแพร่แล้ว"),
    (re.compile(r"\bgit\s+clean\s+.*-\w*x"), "git clean -x ลบไฟล์ที่ ignore ด้วย (อาจใหญ่เกิน snapshot)"),
]


def git(root, *args):
    r = subprocess.run(["git", *args], cwd=root, capture_output=True, text=True, timeout=10)
    return r.stdout.strip() if r.returncode == 0 else ""


def size_of(path):
    if os.path.isfile(path) or os.path.islink(path):
        return os.path.getsize(path)
    total = 0
    for d, _, files in os.walk(path):
        for f in files:
            try:
                total += os.path.getsize(os.path.join(d, f))
            except OSError:
                pass
            if total > MAX_SNAPSHOT:
                return total
    return total


class Snapshot:
    def __init__(self, root):
        self.root = root
        self.stamp = datetime.now().strftime("%Y%m%d-%H%M%S")
        self.dir = os.path.join(root, ".uno", "trash", self.stamp)
        self.files, self.notes = [], []

    def copy(self, path):
        """Copy a file or folder into the trash. Returns False when it cannot be kept."""
        if not os.path.lexists(path):
            return True
        if size_of(path) > MAX_SNAPSHOT:
            return False
        rel = os.path.relpath(path, self.root)
        dest = os.path.join(self.dir, rel if not rel.startswith("..") else os.path.join("_outside", path.lstrip("/")))
        os.makedirs(os.path.dirname(dest), exist_ok=True)
        if os.path.isdir(path) and not os.path.islink(path):
            shutil.copytree(path, dest, symlinks=True, dirs_exist_ok=True)
        else:
            shutil.copy2(path, dest, follow_symlinks=False)
        self.files.append((path, dest))
        return True

    def prune(self):
        """Drop trash folders older than KEEP_DAYS (their recovery.md entries stay as history)."""
        base = os.path.dirname(self.dir)
        cutoff = datetime.now().timestamp() - KEEP_DAYS * 86400
        for name in os.listdir(base) if os.path.isdir(base) else []:
            path = os.path.join(base, name)
            try:
                if os.path.getmtime(path) < cutoff:
                    shutil.rmtree(path, ignore_errors=True)
            except OSError:
                pass

    def write(self, command):
        if not self.files and not self.notes:
            return
        self.prune()
        lines = [f"\n## {self.stamp}", f"- command: `{command[:200]}`"]
        for src, dest in self.files:
            lines.append(f"- kept `{os.path.relpath(src, self.root)}` → restore: `cp -R \"{dest}\" \"{src}\"`")
        lines += [f"- {n}" for n in self.notes]
        path = os.path.join(self.root, ".uno", "recovery.md")
        head = "" if os.path.exists(path) else "# Recovery log\nEvery destructive command UNO ran, and how to undo it.\n"
        with open(path, "a", encoding="utf-8") as fh:
            fh.write(head + "\n".join(lines) + "\n")


def parts(command):
    for part in re.split(r"&&|\|\||;|\n|\|", command):
        try:
            tok = shlex.split(part.strip())
        except ValueError:
            tok = part.split()
        if tok:
            yield tok


def is_scratch(path):
    return path.startswith(SCRATCH)


def under(path, root):
    return path == root or path.startswith(root.rstrip("/") + "/")


def dirty_paths(root, include_tracked=True):
    out = git(root, "status", "--porcelain", "-uall", "-z")
    paths = []
    for entry in filter(None, out.split("\0")):
        code, p = entry[:2], entry[3:]
        if code == "??" or include_tracked:
            if not p.startswith(".uno"):
                paths.append(os.path.join(root, p))
    return paths


def plan(root, command):
    """Return (ask_reason or None, Snapshot)."""
    for rx, why in ASK_ALWAYS:
        if rx.search(command):
            return why, None
    snap = Snapshot(root)
    real_root = os.path.realpath(root)
    for tok in parts(command):
        tool = os.path.basename(tok[0])
        args = [t for t in tok[1:] if not t.startswith("-")]
        if tool in DELETE_TOOLS:
            outside = []
            for t in args:
                full = os.path.realpath(os.path.join(root, os.path.expanduser(t)))
                if is_scratch(full) or is_scratch(os.path.expanduser(t)):
                    continue
                if under(full, real_root):
                    rel = os.path.relpath(full, real_root)
                    if rel.split("/")[0] in CACHE_DIRS or rel.startswith(".uno"):
                        continue
                    if not snap.copy(full):
                        return f"ลบ {t} ซึ่งใหญ่เกินกว่าจะสำรองไว้ได้", None
                else:
                    outside.append(t)
                    if not snap.copy(full):
                        return f"ลบไฟล์นอกโปรเจกต์ ({t}) และใหญ่เกินกว่าจะสำรองได้", None
            if outside:
                snap.write(command)
                return (f"ลบไฟล์นอกโปรเจกต์: {', '.join(outside[:5])} — สำรองไว้แล้วที่ .uno/trash/{snap.stamp} "
                        f"(วิธีกู้อยู่ใน .uno/recovery.md)"), None
        elif tool == "find" and "-delete" in tok:
            start = os.path.realpath(os.path.join(root, args[0])) if args else real_root
            if not under(start, real_root):
                return "find -delete นอกโปรเจกต์", None
            if not snap.copy(start):
                return "find -delete ในโฟลเดอร์ที่ใหญ่เกินกว่าจะสำรองได้", None
        elif tool == "git" and len(tok) > 1:
            sub = tok[1]
            head = git(root, "rev-parse", "HEAD")
            if sub == "reset" and "--hard" in tok or sub in ("checkout", "restore") and "." in tok \
                    or sub == "clean":
                for p in dirty_paths(root, include_tracked=(sub != "clean")):
                    if not snap.copy(p):
                        return "ทิ้งงานที่ยังไม่ commit ซึ่งใหญ่เกินกว่าจะสำรองได้", None
                snap.notes.append(f"HEAD was {head} → `git reset --hard {head}` then copy the kept files back")
            elif sub == "branch" and any(f in tok for f in ("-D", "-d", "--delete")):
                for b in args[1:] if args and args[0] == "branch" else args:
                    sha = git(root, "rev-parse", "--verify", b)
                    if sha:
                        snap.notes.append(f"branch `{b}` was {sha} → restore: `git branch {b} {sha}`")
            elif sub == "stash" and len(tok) > 2 and tok[2] in ("drop", "clear"):
                for sha in git(root, "stash", "list", "--format=%H").split():
                    snap.notes.append(f"stash {sha} → restore: `git stash apply {sha}`")
            elif sub in ("rebase", "filter-branch") or sub == "commit" and "--amend" in tok:
                snap.notes.append(f"HEAD was {head} → restore: `git reset --hard {head}`")
            elif sub == "push" and (any(f in tok for f in ("--force-with-lease", "--force-if-includes", "--delete"))
                                    or any(a.startswith(":") for a in args)):
                for a in args[2:]:
                    b = a.lstrip(":").split(":")[-1].lstrip("+")
                    sha = git(root, "rev-parse", "--verify", f"origin/{b}")
                    if sha:
                        snap.notes.append(f"origin/{b} was {sha} → restore: `git push --force-with-lease origin {sha}:{b}`")
    snap.write(command)
    return None, snap


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
    command = (event.get("tool_input") or {}).get("command", "")
    try:
        why, snap = plan(root, command)
    except Exception:
        return 0
    if not why:
        return 0
    try:
        with open(os.path.join(root, ".uno", "audit.log"), "a", encoding="utf-8") as fh:
            fh.write(json.dumps({"time": datetime.now(timezone.utc).isoformat(timespec="seconds"),
                                 "session": event.get("session_id"), **who(event), "tool": "Bash", "command": command[:200],
                                 "action": "asked-owner", "rule": "danger"}, ensure_ascii=False) + "\n")
    except OSError:
        pass
    print(json.dumps({"hookSpecificOutput": {
        "hookEventName": "PreToolUse",
        "permissionDecision": "ask",
        "permissionDecisionReason": f"UNO: คำสั่งนี้{why} — กระทบนอกโปรเจกต์ JWT ต้องยืนยันเอง",
    }}, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    sys.exit(main())
