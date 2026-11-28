#!/usr/bin/env python3
"""PreToolUse hook for Bash: keep agents off the default branch in onboarded projects.

Blocks (exit 2):
  - `git commit` while the current branch is the default branch
  - `git push` that targets the default branch, or any force push without --force-with-lease
  - `git merge` into the default branch
  - `git checkout` / `git switch` to a ref whose tree still tracks `.uno/` or `CLAUDE.md`
    while those are local symlinks (personal mode): git would replace the symlinks with old files
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


PERSONAL_PATHS = (".uno", "CLAUDE.md")


def checkout_target(sub, rest):
    """Ref whose tree a checkout/switch would write into the working tree, or None."""
    if "--" in rest:
        return None  # file restore, not a branch switch
    create = ("-b", "-B") if sub == "checkout" else ("-c", "-C", "--create", "--force-create")
    args, i = [], 0
    while i < len(rest):
        a = rest[i]
        if a in create:
            i += 2  # skip the new branch name; a start point may follow
            continue
        if not a.startswith("-"):
            args.append(a)
        i += 1
    return args[0] if args else None


def tracks_personal_files(root, ref):
    if not any(os.path.islink(os.path.join(root, p)) for p in PERSONAL_PATHS):
        return False
    listed = git(root, "ls-tree", "-r", "--name-only", ref, "--", *PERSONAL_PATHS)
    return bool(listed)


# Dependency manifests and lock files of any ecosystem. Language-agnostic rule: a commit that
# changes one of these needs an approved plan whose "Dependencies" section is not "none".
MANIFESTS = re.compile(r"(^|/)(package\.json|package-lock\.json|npm-shrinkwrap\.json|yarn\.lock|pnpm-lock\.yaml|bun\.lockb?"
    r"|deno\.json|deno\.lock|Cargo\.toml|Cargo\.lock|pyproject\.toml|poetry\.lock|uv\.lock|Pipfile(\.lock)?"
    r"|requirements[^/]*\.(txt|in)|setup\.(py|cfg)|environment\.ya?ml|composer\.(json|lock)|go\.(mod|sum)"
    r"|pom\.xml|build\.gradle(\.kts)?|settings\.gradle(\.kts)?|gradle\.lockfile|libs\.versions\.toml"
    r"|Gemfile(\.lock)?|.+\.gemspec|pubspec\.(yaml|lock)|Podfile(\.lock)?|Package\.(swift|resolved)|Cartfile(\.resolved)?"
    r"|mix\.(exs|lock)|.+\.(csproj|fsproj|vbproj)|packages\.lock\.json|Directory\.Packages\.props|paket\.(dependencies|lock)"
    r"|conanfile\.(txt|py)|vcpkg\.json|CMakeLists\.txt|stack\.yaml|cabal\.project|.+\.cabal|build\.sbt|project\.clj|deps\.edn"
    r"|Chart\.(yaml|lock)|requirements\.yaml|\.terraform\.lock\.hcl)$")
TASK_RE = re.compile(r"(?:^|[/_-])(T\d+)(?:[/_-]|$)", re.IGNORECASE)


def plan_dependencies(root, branch):
    """(task, text of the approved plan's Dependencies section) or (task, None) if no plan."""
    m = TASK_RE.search(branch or "")
    if not m:
        return None, None
    task = m.group(1).upper()
    try:
        text = open(os.path.join(root, ".uno", "plans", f"{task}.md"), encoding="utf-8").read()
    except OSError:
        return task, None
    sec = re.search(r"^## Dependencies[^\n]*\n(.*?)(?=^## |\Z)", text, re.M | re.S)
    body = re.sub(r"<[^>]*>", "", sec.group(1)).strip() if sec else ""
    return task, body


def manifest_check(root, current):
    staged = git(root, "diff", "--cached", "--name-only").splitlines()
    changed = [p for p in staged if MANIFESTS.search(p)]
    if not changed:
        return None
    task, deps = plan_dependencies(root, current)
    if deps is None or deps.lower() in ("", "none", "-", "n/a"):
        return (f"UNO: this commit changes dependency files ({', '.join(changed[:5])}) but the plan"
                f"{' for ' + task if task else ''} lists no dependencies. That is a PLAN CHANGE: ask the owner, "
                f"add the packages to the plan's Dependencies, get it re-approved, then commit.")
    return None


def check(root, command):
    base = default_branch(root)
    current = git(root, "rev-parse", "--abbrev-ref", "HEAD")
    for part in split_commands(command):
        args = git_args(tokens(part))
        if not args:
            continue
        sub, rest = args[0], args[1:]
        if sub == "commit" and current != base:
            why = manifest_check(root, current)
            if why:
                return why
        if sub in ("checkout", "switch"):
            ref = checkout_target(sub, rest)
            if ref and tracks_personal_files(root, ref):
                return (f"UNO: '{ref}' still tracks .uno/ or CLAUDE.md from before personal mode. "
                        f"Checking it out would replace your local symlinks with old files. "
                        f"Instead: git switch -c <new-branch> origin/{base} && git cherry-pick <commits of {ref}>.")
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
                                 "session": event.get("session_id"), **who(event), "tool": "Bash",
                                 "command": command[:200], "action": "blocked",
                                 "rule": "git-guard"}, ensure_ascii=False) + "\n")
    except OSError:
        pass
    print(reason, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main())
