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

    # --- guard_zones: Uno (main thread) delegates; members are named in the log ---
    zlog = os.path.join(data, "audit.log")
    def last_zone():
        with open(zlog, encoding="utf-8") as fh:
            return json.loads(fh.read().splitlines()[-1])
    member = {"agent_id": "a9", "agent_type": "core:developer", "session_id": "z-1"}
    check("lead: Uno editing code blocked", hook("guard_zones.py", edit("src/a.ts"), repo)[0], 2)
    e = last_zone()
    check("lead: block logged as uno", (e.get("action"), e.get("rule"), e.get("name")), ("blocked", "lead-delegates", "uno"))
    for f in ("CLAUDE.md", ".claude/settings.local.json", "docs/spec.md", "docs/refactor/x-plan.md"):
        check(f"lead: Uno may edit {f}", hook("guard_zones.py", edit(f), repo)[0], 0)
    check("lead: Uno may edit .uno (symlink)", hook("guard_zones.py", edit(".uno/plans/T7.md"), repo)[0], 0)
    check("lead: docs/guide.md is Theta's", hook("guard_zones.py", edit("docs/guide.md"), repo)[0], 2)
    check("lead: member may edit code", hook("guard_zones.py", {**edit("src/a.ts"), **member}, repo)[0], 0)
    e = last_zone()
    check("lead: member edit logged with its name only", (e.get("action"), e.get("name"), e.get("file")), ("allowed", "core:developer", "src/a.ts"))
    check("lead: switch off with UNO_LEAD_EDITS=allow",
          hook("guard_zones.py", edit("src/a.ts"), repo, {"UNO_LEAD_EDITS": "allow"})[0], 0)
    check("lead: agent_type without agent_id is Uno", hook("guard_zones.py", {**edit("src/a.ts"), "agent_type": "core:developer"}, repo)[0], 2)
    open(os.path.join(data, "zones"), "w").write("[forbidden]\nsecret/**\n[review]\n")
    check("lead: forbidden still blocks members", hook("guard_zones.py", {**edit("secret/k"), **member}, repo)[0], 2)
    check("lead: forbidden block names the member", last_zone().get("name"), "core:developer")
    open(os.path.join(data, "zones"), "w").write("[forbidden]\n[review]\n")
    sh("git", "switch", "-q", "master", cwd=repo)
    hook("guard_git.py", {**bash("git commit -m x"), **member}, repo)
    check("git: block names the member", last_zone().get("name"), "core:developer")
    hook("guard_git.py", bash("git commit -m x"), repo)
    check("git: Uno's block is named uno", last_zone().get("name"), "uno")

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

    # trace_heartbeat: one small file per session holding its current state, names only
    beats = os.path.join(repo, ".uno", "heartbeat")
    def beat_file(sid):
        try:
            with open(os.path.join(beats, sid + ".json"), encoding="utf-8") as fh:
                return json.load(fh)
        except (OSError, ValueError):
            return {}
    def beat(sid, ev, *args, **fields):
        event = dict({"session_id": sid, "hook_event_name": ev}, **fields)
        env = dict(CLEAN_ENV, CLAUDE_PROJECT_DIR=repo)
        r = subprocess.run([sys.executable, os.path.join(HOOKS, "trace_heartbeat.py"), *args],
                           input=json.dumps(event), capture_output=True, text=True, env=env)
        return r.returncode, r.stdout + r.stderr
    def seen(sid, *keys):
        b = beat_file(sid)
        return tuple(b.get(k) for k in keys)
    sh("git", "switch", "-q", "fix/T7-x", cwd=repo)
    log_before = open(log, encoding="utf-8").read()
    secret = {"command": "echo SECRET-CMD", "file_path": "/x/SECRET-PATH"}
    outputs = [beat("h-1", "UserPromptSubmit", prompt="SECRET-PROMPT")]
    check("state: prompt → between, task from branch", seen("h-1", "state", "tool", "task"), ("between", "", "T7"))
    check("state: fields", sorted(beat_file("h-1")), ["action", "name", "session", "since", "state", "task", "time", "tool", "version"])
    manifest = json.load(open(os.path.join(HOOKS, "..", ".claude-plugin", "plugin.json"), encoding="utf-8"))
    check("state: records the plugin version that ran", beat_file("h-1").get("version"), manifest["version"])
    outputs.append(beat("h-1", "PreToolUse", tool_name="Bash", tool_input=secret))
    check("state: before a tool → running", seen("h-1", "action", "session", "state", "tool", "name"), ("heartbeat", "h-1", "running", "Bash", ""))
    outputs.append(beat("h-1", "PostToolUse", tool_name="Bash", tool_input=secret, tool_response="SECRET-OUT"))
    check("state: after a tool → between at once (no throttle)", seen("h-1", "state", "tool", "task"), ("between", "Bash", "T7"))
    outputs.append(beat("h-1", "PostToolUseFailure", tool_name="Read", error="SECRET-ERR"))
    check("state: failed tool → between", seen("h-1", "state", "tool"), ("between", "Read"))
    outputs.append(beat("h-1", "PreToolUse", tool_name="AskUserQuestion", tool_input={"questions": "SECRET-Q"}))
    check("state: question → waiting", seen("h-1", "state"), ("waiting",))
    old = beat_file("h-1")
    old["since"] = "2026-01-01T00:00:00+00:00"
    if os.path.isdir(beats):
        open(os.path.join(beats, "h-1.json"), "w").write(json.dumps(old))
    outputs.append(beat("h-1", "PermissionRequest", tool_name="AskUserQuestion"))
    check("state: same state and tool keep their start time", seen("h-1", "state", "since"), ("waiting", "2026-01-01T00:00:00+00:00"))
    outputs.append(beat("h-1", "Notification", "waiting", message="SECRET-MSG"))
    check("state: permission notification → waiting, tool kept", seen("h-1", "state", "tool"), ("waiting", "AskUserQuestion"))
    outputs.append(beat("h-1", "Stop"))
    check("state: stop → done, new start time", (beat_file("h-1").get("state"), beat_file("h-1").get("since") != old["since"]), ("done", True))
    outputs.append(beat("h-1", "Notification", "done"))
    check("state: idle notification → done", seen("h-1", "state"), ("done",))
    outputs.append(beat("h-1", "StopFailure"))
    check("state: API error → error", seen("h-1", "state"), ("error",))
    check("state: hook prints nothing and exits 0 (never decides a permission)", set(outputs), {(0, "")})
    check("state: nothing sensitive stored", "SECRET" in json.dumps(beat_file("h-1")), False)
    check("state: branch name not stored", "fix/" in json.dumps(beat_file("h-1")), False)
    beat("h-1", "SessionEnd")
    check("state: session end removes the file", os.path.exists(os.path.join(beats, "h-1.json")), False)
    beat("h-2", "PreToolUse", tool_name="Agent", tool_input={"subagent_type": "core:ui-designer"})
    check("state: Agent call is Uno handing off", seen("h-2", "name", "tool"), ("", "Agent"))
    beat("h-3", "PreToolUse", tool_name="Grep", agent_type="core:codebase-explorer")
    check("state: old Claude Code (no agent_id) names the agent on the main file", seen("h-3", "name"), ("core:codebase-explorer",))
    beat("h-3", "Stop")
    check("state: end of turn is the main thread again", seen("h-3", "name"), ("",))

    # one file per subagent: <session>.<agent_id>.json
    beat("p-1", "UserPromptSubmit")
    beat("p-1", "SubagentStart", agent_id="a1", agent_type="core:security-reviewer", prompt="SECRET-SUB")
    check("agent: start creates the member's file", seen("p-1.a1", "state", "name", "session", "task"), ("between", "core:security-reviewer", "p-1", "T7"))
    beat("p-1", "PreToolUse", tool_name="Grep", agent_id="a1", agent_type="core:security-reviewer", tool_input=secret)
    beat("p-1", "PreToolUse", tool_name="Bash", agent_id="a2", agent_type="core:test-writer", tool_input=secret)
    check("agent: two members work at once", (seen("p-1.a1", "state", "tool"), seen("p-1.a2", "state", "tool", "name")),
          (("running", "Grep"), ("running", "Bash", "core:test-writer")))
    check("agent: main thread file stays Uno", seen("p-1", "name", "state"), ("", "between"))
    beat("p-1", "SubagentStop", agent_id="a1", agent_type="core:security-reviewer", last_assistant_message="SECRET-ANSWER")
    check("agent: stop removes only that member's file", (os.path.exists(os.path.join(beats, "p-1.a1.json")), os.path.exists(os.path.join(beats, "p-1.a2.json"))), (False, True))
    beat("p-1", "PostToolUse", tool_name="Agent", tool_input={"subagent_type": "core:test-writer"}, tool_response={"status": "async_launched"})
    check("agent: a background launch keeps the member", os.path.exists(os.path.join(beats, "p-1.a2.json")), True)
    beat("p-1", "PostToolUse", tool_name="Agent", tool_input={"subagent_type": "core:test-writer"}, tool_response={"status": "completed", "content": "SECRET-OUT"})
    check("agent: a finished Agent call removes the member (no SubagentStop needed)", os.path.exists(os.path.join(beats, "p-1.a2.json")), False)
    beat("p-1", "PreToolUse", tool_name="Read", agent_id="a3", agent_type="core:architect")
    for bad in ("../x", "a/b", ""):
        beat("p-1", "PreToolUse", tool_name="Read", agent_id=bad, agent_type="core:architect")
    check("agent: unsafe agent ids never make a file", sorted(f for f in os.listdir(beats) if f.startswith("p-1.")), ["p-1.a3.json", "p-1.json"])
    check("agent: nothing sensitive stored", "SECRET" in "".join(open(os.path.join(beats, f)).read() for f in os.listdir(beats) if f.startswith("p-1")), False)
    trail_path = os.path.join(repo, ".uno", "office.log")
    steps = [json.loads(l) for l in open(trail_path, encoding="utf-8")] if os.path.exists(trail_path) else []
    check("trail: start, tools, stop and return in office.log",
          [(x["event"], x["agent"], x["name"], x["tool"]) for x in steps if x["session"] == "p-1"],
          [("start", "a1", "core:security-reviewer", ""), ("tool", "a1", "core:security-reviewer", "Grep"),
           ("tool", "a2", "core:test-writer", "Bash"), ("stop", "a1", "core:security-reviewer", ""),
           ("return", "", "core:test-writer", ""), ("tool", "a3", "core:architect", "Read")])
    check("trail: fields are names only", sorted(steps[0]) if steps else [], ["action", "agent", "event", "name", "session", "time", "tool"])
    check("trail: nothing sensitive stored", "SECRET" in open(trail_path, encoding="utf-8").read(), False)
    check("trail: main-thread tools are not in the trail", any(x["session"] == "h-1" for x in steps), False)
    check("trail: audit.log untouched", open(log, encoding="utf-8").read(), log_before)
    with open(trail_path, "a", encoding="utf-8") as fh:
        fh.write(("x" * 1100 + "\n") * 1000)
    beat("p-1", "PreToolUse", tool_name="Read", agent_id="a3", agent_type="core:architect")
    lines = open(trail_path, encoding="utf-8").read().splitlines()
    check("trail: past 1 MB keeps the newest 1,000 lines", (len(lines), json.loads(lines[-1]).get("tool")), (1000, "Read"))
    beat("p-1", "SessionEnd")
    check("agent: session end removes every file of the session", sorted(f for f in os.listdir(beats) if f.startswith("p-1")), [])
    sh("git", "switch", "-qc", "no-task-here", cwd=repo)
    beat("h-2", "PostToolUse", tool_name="Read")
    check("task: other events keep the last value", seen("h-2", "task"), ("T7",))
    beat("h-2", "PreToolUse", tool_name="Read")
    check("task: branch without id → empty", seen("h-2", "task"), ("",))
    sh("git", "switch", "-q", "--detach", cwd=repo)
    beat("h-2", "UserPromptSubmit")
    check("task: detached HEAD → empty", seen("h-2", "task"), ("",))
    sh("git", "switch", "-q", "fix/T7-x", cwd=repo)
    nogit = os.path.join(tmp, "nogit")
    os.makedirs(os.path.join(nogit, ".uno"))
    hook("trace_heartbeat.py", {"session_id": "n-1", "hook_event_name": "PreToolUse", "tool_name": "Read"}, nogit)
    try:
        n = json.load(open(os.path.join(nogit, ".uno", "heartbeat", "n-1.json")))
    except OSError:
        n = {}
    check("task: not a git repo → still written, no task", (n.get("state"), n.get("task")), ("running", ""))
    beat("h-4", "Other", tool_name="Read")
    beat("h-4", "Notification")
    check("state: unknown events write nothing", os.path.exists(os.path.join(beats, "h-4.json")), False)
    check("heartbeat: audit.log untouched", open(log, encoding="utf-8").read(), log_before)
    for bad in ("../escape", "a/b", "", None):
        rc, _ = beat(bad, "PreToolUse", tool_name="Read")
        check(f"heartbeat: unsafe session id {bad!r} ignored", rc, 0)
    check("heartbeat: only the good sessions have files",
          sorted(os.listdir(beats)) if os.path.isdir(beats) else [], ["h-2.json", "h-3.json"])
    os.makedirs(beats, exist_ok=True)
    stale = os.path.join(beats, "gone.json")
    open(stale, "w").write("{}")
    os.utime(stale, (0, 0))
    beat("h-5", "PreToolUse", tool_name="Read")
    check("heartbeat: files older than a day removed", os.path.exists(stale), False)
    rc, _ = hook("trace_heartbeat.py", {"session_id": "h-6", "hook_event_name": "Stop"}, bare)
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
