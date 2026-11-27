#!/usr/bin/env python3
"""Run every command in plugins/core/hooks/hooks.json the way Claude Code does (through a shell),
with the plugin copied under a folder whose path has a space — like the Claude app's
"~/Library/Application Support/Claude/…" — and check each hook actually starts."""
import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

PLUGIN = Path(__file__).resolve().parent.parent / "plugins" / "core"


def commands():
    hooks = json.loads((PLUGIN / "hooks" / "hooks.json").read_text(encoding="utf-8"))["hooks"]
    for event, groups in hooks.items():
        for g in groups:
            for h in g.get("hooks", []):
                if h.get("type") == "command":
                    yield event, h["command"]


def main():
    tmp = Path(tempfile.mkdtemp(prefix="uno-hj-"))
    root = tmp / "Application Support" / "Claude plugins" / "core"
    shutil.copytree(PLUGIN, root)
    project = tmp / "project"
    project.mkdir()
    env = {k: v for k, v in os.environ.items() if not k.startswith(("GIT_", "ANTHROPIC_"))}
    env.update(CLAUDE_PLUGIN_ROOT=str(root), CLAUDE_PROJECT_DIR=str(project))
    event_input = json.dumps({"tool_name": "Read", "tool_input": {}, "prompt": "", "cwd": str(project),
                              "hook_event_name": "Other", "session_id": "t"})
    results = []
    for event, cmd in commands():
        r = subprocess.run(["/bin/sh", "-c", cmd], input=event_input, capture_output=True, text=True, env=env, cwd=project)
        out = r.stdout + r.stderr
        started = "can't open file" not in out and "No such file" not in out
        results.append((f"{event}: {cmd}", started and r.returncode == 0, r.returncode, out.strip()[:160]))
    wired = {(e, c.split('trace_heartbeat.py"')[1].strip()) for e, c in commands() if "trace_heartbeat.py" in c}
    need = {(e, "") for e in ("UserPromptSubmit", "PreToolUse", "PostToolUse", "PostToolUseFailure",
                              "PermissionRequest", "Stop", "StopFailure", "SessionEnd")}
    need |= {("Notification", "waiting"), ("Notification", "done")}
    results.append(("heartbeat is registered for every state event", wired == need, 0, f"missing {need - wired}, extra {wired - need}"))
    shutil.rmtree(tmp, ignore_errors=True)
    failed = [x for x in results if not x[1]]
    for name, ok, rc, out in failed:
        print(f"FAIL hooks.json under a path with a space: {name} → exit {rc}: {out}")
    print(f"UNO hooks.json tests: {len(results) - len(failed)}/{len(results)} passed")
    return 1 if failed or not results else 0


if __name__ == "__main__":
    sys.exit(main())
