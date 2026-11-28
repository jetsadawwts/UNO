#!/usr/bin/env python3
"""PreToolUse hook for Bash: infrastructure commands may only touch local, owner-approved targets.

Agents may build, lint, render and dry-run infrastructure, and may change LOCAL targets
(kind / minikube / docker-desktop clusters, local compose). Anything that changes a shared or
remote system is blocked: kubectl/helm against a context not listed under `- Local clusters:` in
the profile's `## Infra` section, terraform/pulumi apply or destroy, docker push / login, cloud CLIs
that write, and destructive Kafka admin commands. Read-only commands pass. Active only in
onboarded projects. Never blocks on its own bug.
"""
import json
import os
import re
import shlex
import subprocess
import sys
from datetime import datetime, timezone

DEFAULT_LOCAL = {"kind-kind", "minikube", "docker-desktop", "rancher-desktop", "orbstack", "k3d-local"}
KUBECTL_WRITE = {"apply", "create", "delete", "replace", "patch", "edit", "scale", "rollout", "set",
                 "label", "annotate", "drain", "cordon", "uncordon", "taint", "exec", "cp", "port-forward",
                 "run", "expose", "autoscale"}
HELM_WRITE = {"install", "upgrade", "uninstall", "delete", "rollback", "push"}
ALWAYS_BLOCK = [
    (re.compile(r"\bterraform\s+(apply|destroy|import|taint|state\s+(rm|mv|push))\b"), "terraform changes real infrastructure"),
    (re.compile(r"\btofu\s+(apply|destroy|import)\b"), "OpenTofu changes real infrastructure"),
    (re.compile(r"\bpulumi\s+(up|destroy|import|cancel)\b"), "pulumi changes real infrastructure"),
    (re.compile(r"\bdocker\s+(push|login)\b|\bpodman\s+(push|login)\b"), "pushing images or logging in to a registry"),
    (re.compile(r"\b(aws|gcloud|az|doctl|oci)\s+\S+.*\b(create|delete|update|put|deploy|apply|set|remove|start|stop|terminate|run)\b"), "a cloud CLI that writes"),
    (re.compile(r"\baws\s+s3\s+(rm|rb|mv|cp|sync)\b|\bgsutil\s+(rm|mv|cp|rsync)\b|\bgcloud\s+storage\s+(rm|mv|cp|rsync)\b"), "writing to cloud storage"),
    (re.compile(r"\bkafka-(topics|configs|acls|consumer-groups|reassign-partitions|delete-records)(\.sh)?\b.*--(delete|alter|create|reset-offsets|execute)\b"), "Kafka admin change"),
    (re.compile(r"\bdocker\s+compose\b.*\bdown\b.*(-v|--volumes)\b|\bdocker\s+(volume|system)\s+(rm|prune)\b"), "deleting Docker volumes (data loss)"),
]


def profile_local_clusters(root):
    try:
        text = open(os.path.join(root, ".uno", "project-profile.md"), encoding="utf-8").read()
    except OSError:
        return set(DEFAULT_LOCAL)
    m = re.search(r"^- Local clusters:\s*(.+)$", text, re.M)
    if not m:
        return set(DEFAULT_LOCAL)
    return {c.strip().strip("`") for c in re.split(r"[,\s]+", m.group(1)) if c.strip() and not c.startswith("<")}


def current_context():
    try:
        out = subprocess.run(["kubectl", "config", "current-context"], capture_output=True, text=True, timeout=5)
        return out.stdout.strip() if out.returncode == 0 else ""
    except (OSError, subprocess.SubprocessError):
        return ""


def flag_value(tok, *names):
    for i, t in enumerate(tok):
        for n in names:
            if t == n and i + 1 < len(tok):
                return tok[i + 1]
            if t.startswith(n + "="):
                return t.split("=", 1)[1]
    return None


def check(root, command):
    for rx, why in ALWAYS_BLOCK:
        if rx.search(command):
            return f"UNO: blocked — {why}. Agents prepare and dry-run; the owner runs this by hand."
    for part in re.split(r"&&|\|\||;|\n|\|", command):
        try:
            tok = shlex.split(part.strip())
        except ValueError:
            tok = part.split()
        if not tok:
            continue
        tool = os.path.basename(tok[0])
        if tool in ("kubectl", "oc", "k"):
            verbs = [t for t in tok[1:] if not t.startswith("-")]
            verb = verbs[0] if verbs else ""
            if verb not in KUBECTL_WRITE or "--dry-run=client" in tok or "--dry-run=server" in tok or "--dry-run" in tok:
                continue
        elif tool == "helm":
            verbs = [t for t in tok[1:] if not t.startswith("-")]
            verb = verbs[0] if verbs else ""
            if verb not in HELM_WRITE or "--dry-run" in tok:
                continue
        else:
            continue
        ctx = flag_value(tok, "--context", "--kube-context") or current_context()
        local = profile_local_clusters(root)
        if ctx not in local:
            return (f"UNO: '{tool} {verb}' would change cluster context '{ctx or 'unknown'}', which is not a local "
                    f"cluster in the profile ({', '.join(sorted(local))}). Use --dry-run, a local cluster, or ask the owner.")
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
    command = (event.get("tool_input") or {}).get("command", "")
    try:
        reason = check(root, command)
    except Exception:
        return 0
    if not reason:
        return 0
    try:
        with open(os.path.join(root, ".uno", "audit.log"), "a", encoding="utf-8") as fh:
            fh.write(json.dumps({"time": datetime.now(timezone.utc).isoformat(timespec="seconds"),
                                 "session": event.get("session_id"), **who(event), "tool": "Bash", "command": command[:200],
                                 "action": "blocked", "rule": "infra"}, ensure_ascii=False) + "\n")
    except OSError:
        pass
    print(reason, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main())
