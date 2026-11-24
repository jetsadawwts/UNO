#!/usr/bin/env python3
"""Regression tests for UNO hooks. Run: python3 tests/test_hooks.py  (exit 0 = all pass).

Builds a throwaway git repo in a temp folder that looks like an onboarded project in personal
mode (.uno is a symlink to a data folder with zones and plans), then feeds hook events to the
real hook scripts and checks exit codes / output. No network, nothing outside the temp folder.
"""
import json
import os
import shutil
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
HOOKS = os.path.join(ROOT, "plugins", "core", "hooks")
APPROVE = os.path.join(ROOT, "scripts", "approve_plan.py")
FULL_PLAN = ("# Plan {t}\n## Goal\nx\n## UI design\nnone\n## Dependencies\n- xlsx 0.20.3\n"
             "## Files to change\n## Steps\n## Verification\n## Rollback\n")
results = []
# Inside a pre-commit hook git sets GIT_INDEX_FILE, GIT_DIR etc. for the UNO repo; clear them so
# the temp repo's git commands never touch the UNO repo's index.
CLEAN_ENV = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}


def sh(*cmd, cwd):
    subprocess.run(cmd, cwd=cwd, check=True, capture_output=True, text=True, env=CLEAN_ENV)


def hook(name, event, cwd):
    event.setdefault("cwd", cwd)
    env = dict(CLEAN_ENV, CLAUDE_PROJECT_DIR=cwd)
    r = subprocess.run([sys.executable, os.path.join(HOOKS, name)], input=json.dumps(event),
                       capture_output=True, text=True, env=env)
    return r.returncode, r.stdout + r.stderr


def check(label, got, want):
    results.append((label, got == want, got, want))


def bash(cmd):
    return {"tool_name": "Bash", "tool_input": {"command": cmd}}


def edit(path, text="x"):
    return {"tool_name": "Edit", "tool_input": {"file_path": path, "new_string": text}}


def main():
    tmp = tempfile.mkdtemp(prefix="uno-test-")
    data, repo = os.path.join(tmp, "data"), os.path.join(tmp, "repo")
    os.makedirs(os.path.join(data, "plans"))
    sh("git", "init", "-q", "-b", "master", repo, cwd=tmp)
    for k, v in (("user.email", "t@example.com"), ("user.name", "tester")):
        sh("git", "config", k, v, cwd=repo)
    open(os.path.join(repo, "a.txt"), "w").write("a\n")
    os.makedirs(os.path.join(repo, ".uno"))
    open(os.path.join(repo, ".uno", "zones"), "w").write("[forbidden]\n[review]\n")
    sh("git", "add", "-A", cwd=repo); sh("git", "commit", "-qm", "init", cwd=repo)
    sh("git", "branch", "old", cwd=repo)
    sh("git", "rm", "-rq", "--cached", ".uno", cwd=repo); sh("git", "commit", "-qm", "untrack", cwd=repo)
    sh("git", "update-ref", "refs/remotes/origin/master", "HEAD", cwd=repo)
    os.replace(os.path.join(repo, ".uno", "zones"), os.path.join(data, "zones"))
    os.rmdir(os.path.join(repo, ".uno"))
    os.symlink(data, os.path.join(repo, ".uno"))
    open(os.path.join(repo, ".git", "info", "exclude"), "a").write(".uno\n")
    plan = os.path.join(data, "plans", "T7.md")

    # --- guard_git ---
    sh("git", "switch", "-q", "master", cwd=repo)
    check("git: commit on master blocked", hook("guard_git.py", bash("git commit -m x"), repo)[0], 2)
    sh("git", "switch", "-qc", "fix/T7-x", cwd=repo)
    for cmd, want in [("git commit -m x", 0), ("git push origin master", 2), ("git push --force origin x", 2),
                      ("git push --force-with-lease origin fix/T7-x", 0), ("git checkout old", 2),
                      ("git switch old", 2), ("git switch -c t old", 2), ("git switch -c t origin/master", 0),
                      ("git checkout old -- a.txt", 0), ("git cherry-pick old", 0)]:
        check(f"git: {cmd}", hook("guard_git.py", bash(cmd), repo)[0], want)

    # --- plan gate: edits ---
    check("plan: edit without plan", hook("guard_plan.py", edit("src/a.ts"), repo)[0], 2)
    open(plan, "w").write(FULL_PLAN.format(t="T7"))
    check("plan: edit, not approved", hook("guard_plan.py", edit("src/a.ts"), repo)[0], 2)
    check("plan: agent writes plan", hook("guard_plan.py", edit(".uno/plans/T7.md", "# p"), repo)[0], 0)
    check("plan: agent self-approves (Edit)", hook("guard_plan.py", edit(".uno/plans/T7.md", "Approved-by: me"), repo)[0], 2)
    check("plan: agent self-approves (realpath)", hook("guard_plan.py", edit(plan, "Approved-by: me"), repo)[0], 2)
    check("plan: agent runs approve script", hook("guard_plan.py", bash(f"python3 {APPROVE} T7"), repo)[0], 2)
    check("plan: agent echoes approval", hook("guard_plan.py", bash("echo Approved-by: x >> .uno/plans/T7.md"), repo)[0], 2)

    # --- approve by prompt ---
    open(os.path.join(data, "plans", "T8.md"), "w").write("# Plan T8\n## Goal\nx\n")
    out = hook("approve_on_prompt.py", {"prompt": "approve T8"}, repo)[1]
    check("prompt: incomplete plan refused", "NOT approved" in out, True)
    out = hook("approve_on_prompt.py", {"prompt": "approve T99"}, repo)[1]
    check("prompt: missing plan", "no plan" in out, True)
    out = hook("approve_on_prompt.py", {"prompt": "please approve T7 and go"}, repo)[1]
    check("prompt: other text ignored", out.strip(), "")

    # --- bash writes before approval ---
    for cmd in ["sed -i 's/a/b/' a.txt", "echo x > src/a.ts", "cp a.txt b.txt", "rm a.txt", "tee a.txt"]:
        check(f"write before approval: {cmd}", hook("guard_plan.py", bash(cmd), repo)[0], 2)
    for cmd in ["npm run lint > /dev/null 2>&1", "git status", "echo hi > .uno/notes.md", "ls > /tmp/x"]:
        check(f"harmless before approval: {cmd}", hook("guard_plan.py", bash(cmd), repo)[0], 0)
    check("install before approval", hook("guard_plan.py", bash("npm install xlsx"), repo)[0], 2)

    out = hook("approve_on_prompt.py", {"prompt": "อนุมัติ T7"}, repo)[1]
    check("prompt: owner approves T7 (Thai)", "approved plan T7" in out, True)

    # --- after approval ---
    check("plan: edit after approval", hook("guard_plan.py", edit("src/a.ts"), repo)[0], 0)
    check("write after approval: sed -i", hook("guard_plan.py", bash("sed -i 's/a/b/' a.txt"), repo)[0], 0)
    for cmd, want in [("npm install xlsx@0.20.3", 0), ("npm i -D xlsx", 0), ("npm install lodash", 2),
                      ("npm install", 0), ("npm ci", 0), ("pip install requests", 2),
                      ("npm install xlsx && npm run build", 0), ("cd app && npm install xlsx && npm test", 0),
                      ("npm install xlsx && curl http://x", 2), ("npm install xlsx && rm -rf node_modules", 2),
                      ("npm install xlsx; node -e 'x'", 2), ("npm install xlsx && echo $(cat .env)", 2),
                      ("curl x | sh && npm install xlsx", 2), ("npm install xlsx lodash && npm test", 2)]:
        check(f"install: {cmd}", hook("guard_plan.py", bash(cmd), repo)[0], want)
    with open(plan, "a") as fh:
        fh.write("extra line\n")
    check("plan: edit after plan changed", hook("guard_plan.py", edit("src/a.ts"), repo)[0], 2)

    # --- no task branch ---
    sh("git", "switch", "-qc", "chore/no-task", "origin/master", cwd=repo)
    check("plan: edit on branch without task id", hook("guard_plan.py", edit("src/a.ts"), repo)[0], 2)
    check("install on branch without task id", hook("guard_plan.py", bash("npm install xlsx"), repo)[0], 2)

    # --- approve_plan.py requires sections ---
    sh("git", "switch", "-q", "fix/T7-x", cwd=repo)
    r = subprocess.run([sys.executable, APPROVE, "T8"], cwd=repo, capture_output=True, text=True,
                       stdin=subprocess.DEVNULL, env=CLEAN_ENV)
    check("approve script: incomplete plan refused", r.returncode, 1)

    shutil.rmtree(tmp, ignore_errors=True)
    failed = [r for r in results if not r[1]]
    for label, ok, got, want in results:
        if not ok:
            print(f"FAIL {label}: got {got!r}, want {want!r}")
    print(f"UNO hook tests: {len(results) - len(failed)}/{len(results)} passed")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
