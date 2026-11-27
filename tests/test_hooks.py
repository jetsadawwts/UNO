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
from datetime import datetime, timedelta, timezone

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
HOOKS = os.path.join(ROOT, "plugins", "core", "hooks")
APPROVE = os.path.join(ROOT, "scripts", "approve_plan.py")
FULL_PLAN = ("# Plan {t}\n## Goal\nx\n## UI design\nnone\n## Dependencies\n- xlsx 0.20.3\n- tokio (cargo add tokio)\n"
             "## Files to change\n## Steps\n## Verification\n## Rollback\n")
results = []
# Inside a pre-commit hook git sets GIT_INDEX_FILE, GIT_DIR etc. for the UNO repo; clear them so
# the temp repo's git commands never touch the UNO repo's index.
CLEAN_ENV = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}


def sh(*cmd, cwd):
    subprocess.run(cmd, cwd=cwd, check=True, capture_output=True, text=True, env=CLEAN_ENV)


def hook(name, event, cwd, extra_env=None):
    event.setdefault("cwd", cwd)
    env = dict({k: v for k, v in CLEAN_ENV.items() if not k.startswith("ANTHROPIC_")},
               CLAUDE_PROJECT_DIR=cwd, **(extra_env or {}))
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
    out = hook("approve_on_prompt.py", {"prompt": "uno-smoke smoke-123-9"}, repo)[1]
    check("smoke prompt blocked before the model", '"decision": "block"' in out and "smoke-123-9" in out, True)
    check("smoke prompt logged", '"nonce": "smoke-123-9"' in open(os.path.join(data, "audit.log")).read(), True)
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
    check("approval records backend: anthropic", "backend:anthropic" in open(plan).read(), True)
    t9 = os.path.join(data, "plans", "T9.md")
    open(t9, "w").write(FULL_PLAN.format(t="T9"))
    hook("approve_on_prompt.py", {"prompt": "approve T9"}, repo,
         {"ANTHROPIC_BASE_URL": "http://ollama.example:11434", "ANTHROPIC_DEFAULT_SONNET_MODEL": "qwen3-coder:30b"})
    line = [l for l in open(t9).read().splitlines() if l.startswith("Approved-by:")][-1]
    check("approval records local backend and model",
          "backend:ollama.example:11434" in line and "model:qwen3-coder:30b" in line, True)

    # --- after approval ---
    check("plan: edit after approval", hook("guard_plan.py", edit("src/a.ts"), repo)[0], 0)
    check("write after approval: sed -i", hook("guard_plan.py", bash("sed -i 's/a/b/' a.txt"), repo)[0], 0)
    for cmd, want in [("npm install xlsx@0.20.3", 0), ("npm i -D xlsx", 0), ("npm install lodash", 2),
                      ("npm install", 0), ("npm ci", 0), ("pip install requests", 2),
                      ("npm install xlsx && npm run build", 0), ("cd app && npm install xlsx && npm test", 0),
                      ("npm install xlsx && curl http://x", 2), ("npm install xlsx && rm -rf node_modules", 2),
                      ("npm install xlsx; node -e 'x'", 2), ("npm install xlsx && echo $(cat .env)", 2),
                      ("curl x | sh && npm install xlsx", 2), ("npm install xlsx lodash && npm test", 2),
                      ("cargo add serde", 2), ("composer require monolog/monolog:^3", 2), ("poetry add requests", 2),
                      ("uv add httpx", 2), ("go get example.com/lib@v1.2.0", 2),
                      ("pip install -r requirements.txt", 0), ("composer install", 0), ("cargo build && cargo test", 0),
                      ("uv sync", 0), ("pip install xlsx==0.20.3", 0), ("cargo add tokio", 0)]:
        check(f"install: {cmd}", hook("guard_plan.py", bash(cmd), repo)[0], want)
    with open(plan, "a") as fh:
        fh.write("extra line\n")
    check("plan: edit after plan changed", hook("guard_plan.py", edit("src/a.ts"), repo)[0], 2)

    # --- browser hosts ---
    def browse(url, tool="mcp__Claude_Browser__navigate"):
        return hook("guard_browser.py", {"tool_name": tool, "tool_input": {"url": url}}, repo)[0]
    profile = os.path.join(data, "project-profile.md")
    check("browser: no preview section -> blocked", browse("http://localhost:5173/log"), 2)
    open(profile, "w").write("# P\n## Preview\n- URL: http://localhost:5173\n- Allowed hosts: uat.example.org\n- Data: none\n")
    check("browser: Data none -> blocked", browse("http://localhost:5173/log"), 2)
    open(profile, "w").write("# P\n## Preview\n- URL: http://localhost:5173\n- Allowed hosts: uat.example.org, 10.0.0.5:8080\n- Data: fake\n## Other\n")
    check("browser: localhost ok", browse("http://localhost:5173/log"), 0)
    check("browser: allowed UAT host ok", browse("https://uat.example.org/his"), 0)
    check("browser: allowed host:port ok", browse("http://10.0.0.5:8080/x"), 0)
    check("browser: production host blocked", browse("https://his.example.org/"), 2)
    check("browser: chrome tool, other host blocked", browse("https://mail.example.com", "mcp__claude-in-chrome__navigate"), 2)
    check("browser: no url (screenshot) ok", hook("guard_browser.py", {"tool_name": "mcp__Claude_Browser__computer", "tool_input": {"action": "screenshot"}}, repo)[0], 0)
    check("browser: non-browser tool ignored", hook("guard_browser.py", bash("curl https://his.example.org"), repo)[0], 0)

    # --- language-agnostic: dependency files at commit time ---
    t10 = os.path.join(data, "plans", "T10.md")
    open(t10, "w").write("# Plan T10\n## Goal\nx\n## UI design\nnone\n## Dependencies\nnone\n## Files to change\n## Steps\n## Verification\n## Rollback\n")
    sh("git", "switch", "-qc", "feat/T10-x", "origin/master", cwd=repo)
    for name in ("pom.xml", "app/build.gradle.kts", "pubspec.yaml", "Package.swift", "go.mod"):
        os.makedirs(os.path.dirname(os.path.join(repo, name)) or repo, exist_ok=True)
        open(os.path.join(repo, name), "w").write("x\n")
        sh("git", "add", name, cwd=repo)
        check(f"commit with {name}, plan says none -> blocked", hook("guard_git.py", bash("git commit -m deps"), repo)[0], 2)
        sh("git", "reset", "-q", name, cwd=repo)
    open(t10, "w").write("# Plan T10\n## Goal\nx\n## UI design\nnone\n## Dependencies\n- com.squareup.okhttp3:okhttp 4.12.0\n## Files to change\n## Steps\n## Verification\n## Rollback\n")
    sh("git", "add", "pom.xml", cwd=repo)
    check("commit with pom.xml, plan lists deps -> ok", hook("guard_git.py", bash("git commit -m deps"), repo)[0], 0)
    sh("git", "reset", "-q", "pom.xml", cwd=repo)
    open(os.path.join(repo, "src.txt"), "w").write("x\n"); sh("git", "add", "src.txt", cwd=repo)
    check("commit without dependency files -> ok", hook("guard_git.py", bash("git commit -m code"), repo)[0], 0)
    sh("git", "reset", "-q", "src.txt", cwd=repo)
    sh("git", "switch", "-q", "fix/T7-x", cwd=repo)
    for cmd in ("flutter pub add http", "dart pub add http", "deno add jsr:@std/path", "npx expo install expo-camera"):
        check(f"install outside plan: {cmd}", hook("guard_plan.py", bash(cmd), repo)[0], 2)

    # --- infrastructure ---
    for cmd, want in [("terraform apply -auto-approve", 2), ("terraform plan", 0), ("terraform validate", 0),
                      ("docker push registry.example.com/app:1", 2), ("docker build -t app:dev .", 0),
                      ("docker compose down -v", 2), ("docker compose up -d", 0),
                      ("helm upgrade app ./chart --kube-context prod-cluster", 2), ("helm template app ./chart", 0),
                      ("helm upgrade app ./chart --kube-context kind-kind", 0),
                      ("kubectl apply -f k8s/ --context prod-cluster", 2), ("kubectl apply -f k8s/ --dry-run=client --context prod-cluster", 0),
                      ("kubectl get pods --context prod-cluster", 0), ("kubectl delete ns x --context minikube", 0),
                      ("kafka-topics --bootstrap-server b:9092 --delete --topic orders", 2),
                      ("kafka-topics --bootstrap-server b:9092 --list", 0),
                      ("aws s3 rm s3://bucket/x --recursive", 2), ("aws s3 ls s3://bucket", 0),
                      ("aws ec2 terminate-instances --instance-ids i-1", 2)]:
        check(f"infra: {cmd}", hook("guard_infra.py", bash(cmd), repo)[0], want)

    # --- secrets through Bash ---
    os.makedirs(os.path.join(repo, "public"), exist_ok=True)
    open(os.path.join(repo, "public", "config.ini"), "w").write("SECRET_KEY=x\n")
    open(os.path.join(repo, ".env"), "w").write("K=1\n")
    open(os.path.join(repo, ".env.example"), "w").write("K=\n")
    for cmd, want in [("cat public/config.ini", 2), ("grep SECRET public/config.ini", 2), ("head -5 .env", 2),
                      ("tail -n1 ./public/config.ini", 2), ("git show HEAD:public/config.ini", 2),
                      ("cat public/*.ini", 2), ("python3 -c \"print(open('public/config.ini').read())\"", 2),
                      ("cat ~/.aws/credentials", 2), ("cat .env.example", 0),
                      ("git add -A -- . ':!**/config.ini' ':!**/.env*'", 0), ("cat package.json", 0),
                      ("npm run lint", 0), ("grep -n TODO src/a.ts", 0), ("git status --short", 0),
                      ("echo .uno >> .git/info/exclude", 0)]:
        check(f"secrets: {cmd}", hook("guard_secrets.py", bash(cmd), repo)[0], want)
    for f in ("public/config.ini", ".env", ".env.example"):
        os.remove(os.path.join(repo, f))

    # --- deleting: snapshot inside the project, ask outside it ---
    def asks(cmd):
        code, out = hook("guard_danger.py", bash(cmd), repo)
        return code == 0 and '"permissionDecision": "ask"' in out
    open(os.path.join(repo, "u.txt"), "w").write("untracked\n")
    for cmd, want in [("rm a.txt", False), ("git clean -fd", False), ("git reset --hard", False),
                      ("git branch -D old", False), ("git stash drop", False), ("git commit --amend -m x", False),
                      ("docker compose down", False), ("kubectl delete ns x --context minikube", False),
                      ("rm -rf node_modules", False), ("rm /tmp/x.txt", False), ("npm run build", False),
                      ("git clean -fdx", True), ("rm /etc/hosts", True), ("rm ~/Library/Preferences/x.plist", True),
                      ("sudo rm -r /opt/x", True), ("brew uninstall node", True), ("npm uninstall -g typescript", True),
                      ("pip uninstall requests", True), ("shred a.txt", True), ("defaults delete com.apple.dock", True),
                      ("psql -c 'DROP TABLE users'", True), ("npx prisma migrate reset", True)]:
        check(f"danger asks={want}: {cmd}", asks(cmd), want)
    trash = os.path.join(data, "trash")
    old = os.path.join(data, "trash", "20000101-000000")
    os.makedirs(old, exist_ok=True); os.utime(old, (946684800, 946684800))
    hook("guard_danger.py", bash("rm a.txt"), repo)
    check("danger: trash older than 30 days pruned", os.path.exists(old), False)
    kept = [os.path.join(d, f) for d, _, fs in os.walk(trash) for f in fs]
    check("danger: rm a.txt kept a copy", any(p.endswith("/a.txt") for p in kept), True)
    check("danger: git clean kept the untracked file", any(p.endswith("/u.txt") for p in kept), True)
    rec = open(os.path.join(data, "recovery.md")).read()
    check("danger: branch -D recorded with restore command", "git branch old " in rec, True)
    check("danger: reset --hard recorded HEAD", "git reset --hard " in rec, True)
    os.remove(os.path.join(repo, "u.txt"))

    # --- no task branch ---
    sh("git", "switch", "-qc", "chore/no-task", "origin/master", cwd=repo)
    check("plan: edit on branch without task id", hook("guard_plan.py", edit("src/a.ts"), repo)[0], 2)
    check("install on branch without task id", hook("guard_plan.py", bash("npm install xlsx"), repo)[0], 2)

    # --- approve_plan.py requires sections ---
    sh("git", "switch", "-q", "fix/T7-x", cwd=repo)
    r = subprocess.run([sys.executable, APPROVE, "T8"], cwd=repo, capture_output=True, text=True,
                       stdin=subprocess.DEVNULL, env=CLEAN_ENV)
    check("approve script: incomplete plan refused", r.returncode, 1)

    # trace_session: SessionStart / SessionEnd become audit events the dashboard counts
    log = os.path.join(repo, ".uno", "audit.log")
    def last_event():
        with open(log, encoding="utf-8") as fh:
            return json.loads(fh.read().splitlines()[-1])
    rc, _ = hook("trace_session.py", {"hook_event_name": "SessionStart", "session_id": "s-1", "source": "startup"}, repo)
    e = last_event()
    check("trace_session: start exits 0", rc, 0)
    check("trace_session: start recorded", (e.get("action"), e.get("session"), e.get("source")), ("session-start", "s-1", "startup"))
    check("trace_session: start has a time", bool(e.get("time")), True)
    rc, _ = hook("trace_session.py", {"hook_event_name": "SessionEnd", "session_id": "s-1", "reason": "prompt_input_exit"}, repo)
    e = last_event()
    check("trace_session: end recorded", (rc, e.get("action"), e.get("session"), e.get("reason")), (0, "session-end", "s-1", "prompt_input_exit"))
    bare = os.path.join(tmp, "bare")
    os.makedirs(bare)
    rc, _ = hook("trace_session.py", {"hook_event_name": "SessionStart", "session_id": "s-2"}, bare)
    check("trace_session: no .uno → nothing written", (rc, os.path.exists(os.path.join(bare, ".uno"))), (0, False))
    rc, _ = hook("trace_session.py", {"hook_event_name": "Other", "session_id": "s-3"}, repo)
    check("trace_session: other events ignored", (rc, last_event().get("session")), (0, "s-1"))
    r = subprocess.run([sys.executable, os.path.join(HOOKS, "trace_session.py")], input="not json",
                       capture_output=True, text=True, env=CLEAN_ENV)
    check("trace_session: bad input exits 0", r.returncode, 0)

    # trace_heartbeat: one small file per session, at most once a minute, names only
    beats = os.path.join(repo, ".uno", "heartbeat")
    def beat_file(sid):
        try:
            with open(os.path.join(beats, sid + ".json"), encoding="utf-8") as fh:
                return json.load(fh)
        except (OSError, ValueError):
            return {}
    def beat(sid, tool, tool_input=None, **extra):
        return hook("trace_heartbeat.py", dict({"session_id": sid, "tool_name": tool, "tool_input": tool_input or {}}, **extra), repo)
    def age(sid, seconds):
        old = beat_file(sid)
        if old:
            old["time"] = (datetime.now(timezone.utc) - timedelta(seconds=seconds)).isoformat(timespec="seconds")
            with open(os.path.join(beats, sid + ".json"), "w", encoding="utf-8") as fh:
                json.dump(old, fh)
    log_before = open(log, encoding="utf-8").read()
    rc, out = beat("h-1", "Read", {"file_path": "/x/SECRET-PATH"})
    b = beat_file("h-1")
    check("heartbeat: exits 0 silently", (rc, out), (0, ""))
    check("heartbeat: fields", sorted(b), ["action", "name", "session", "time", "tool"])
    check("heartbeat: values", (b.get("action"), b.get("session"), b.get("tool"), b.get("name")), ("heartbeat", "h-1", "Read", ""))
    beat("h-1", "Bash", {"command": "echo SECRET-CMD"})
    check("heartbeat: throttled within a minute", beat_file("h-1").get("tool"), "Read")
    age("h-1", 61)
    beat("h-1", "Bash", {"command": "echo SECRET-CMD"})
    check("heartbeat: written again after a minute", beat_file("h-1").get("tool"), "Bash")
    check("heartbeat: no command or input stored", "SECRET" in json.dumps(beat_file("h-1")), False)
    beat("h-2", "Agent", {"subagent_type": "core:ui-designer", "prompt": "SECRET-PROMPT"})
    check("heartbeat: Agent call stores the agent", beat_file("h-2").get("name"), "core:ui-designer")
    beat("h-3", "Grep", agent_type="core:codebase-explorer")
    check("heartbeat: tool inside a subagent stores the agent", beat_file("h-3").get("name"), "core:codebase-explorer")
    check("heartbeat: audit.log untouched", open(log, encoding="utf-8").read(), log_before)
    for bad in ("../escape", "a/b", "", None):
        rc, _ = beat(bad, "Read")
        check(f"heartbeat: unsafe session id {bad!r} ignored", rc, 0)
    check("heartbeat: only the good sessions have files",
          sorted(os.listdir(beats)) if os.path.isdir(beats) else [], ["h-1.json", "h-2.json", "h-3.json"])
    check("heartbeat: nothing written outside", os.path.exists(os.path.join(repo, ".uno", "escape.json")), False)
    os.makedirs(beats, exist_ok=True)
    stale = os.path.join(beats, "gone.json")
    open(stale, "w").write("{}")
    os.utime(stale, (0, 0))
    beat("h-4", "Read")
    check("heartbeat: files older than a day removed", os.path.exists(stale), False)
    rc, _ = hook("trace_heartbeat.py", {"session_id": "h-5", "tool_name": "Read"}, bare)
    check("heartbeat: no .uno → nothing written", (rc, os.path.exists(os.path.join(bare, ".uno"))), (0, False))
    r = subprocess.run([sys.executable, os.path.join(HOOKS, "trace_heartbeat.py")], input="[1]",
                       capture_output=True, text=True, env=CLEAN_ENV)
    check("heartbeat: bad input exits 0", r.returncode, 0)

    shutil.rmtree(tmp, ignore_errors=True)
    failed = [r for r in results if not r[1]]
    for label, ok, got, want in results:
        if not ok:
            print(f"FAIL {label}: got {got!r}, want {want!r}")
    print(f"UNO hook tests: {len(results) - len(failed)}/{len(results)} passed")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
