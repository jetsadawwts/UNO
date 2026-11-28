#!/usr/bin/env python3
"""PreToolUse hook for browser tools: open only local and approved preview hosts.

Matches any tool whose name contains "Browser" or "chrome" (built-in browser, Claude in Chrome,
Playwright MCP). If its input has a URL, the host must be localhost / 127.0.0.1 or listed on the
`- Allowed hosts:` line of the `## Preview` section in .uno/project-profile.md, and that section
must say `- Data: fake` (owner-confirmed). Anything else is blocked, so a remembered login can
never be used on a production system. Active only in onboarded projects. Never blocks on its own bug.
"""
import json
import os
import re
import sys
from datetime import datetime, timezone
from urllib.parse import urlparse

LOCAL = {"localhost", "127.0.0.1", "::1", "0.0.0.0"}


def preview(root):
    try:
        text = open(os.path.join(root, ".uno", "project-profile.md"), encoding="utf-8").read()
    except OSError:
        return None, set()
    m = re.search(r"^## Preview\s*$(.*?)(?=^## |\Z)", text, re.M | re.S)
    if not m:
        return None, set()
    sec = m.group(1)
    data = re.search(r"^- Data:\s*(\S+)", sec, re.M)
    hosts = re.search(r"^- Allowed hosts:\s*(.+)$", sec, re.M)
    allowed = set()
    if hosts:
        for h in re.split(r"[,\s]+", hosts.group(1)):
            h = h.strip().strip("`").lower()
            if h and not h.startswith("<"):
                allowed.add(re.sub(r"^https?://", "", h).split("/")[0])
    return (data.group(1).strip("`").lower() if data else None), allowed


def find_url(obj):
    if isinstance(obj, dict):
        for k, v in obj.items():
            if k.lower() in ("url", "uri", "href") and isinstance(v, str):
                return v
            found = find_url(v)
            if found:
                return found
    elif isinstance(obj, list):
        for v in obj:
            found = find_url(v)
            if found:
                return found
    return None


def check(root, url):
    if not url or url.startswith(("about:", "data:", "chrome://newtab")):
        return None
    parsed = urlparse(url if "://" in url else "http://" + url)
    host = (parsed.hostname or "").lower()
    hostport = host + (f":{parsed.port}" if parsed.port else "")
    data, allowed = preview(root)
    if data != "fake":
        return ("UNO: no owner-confirmed fake-data preview for this project (profile `## Preview` needs "
                "`- Data: fake`). Do not open the app in a browser; test with unit/component tests instead.")
    if host in LOCAL or host in allowed or hostport in allowed:
        return None
    return (f"UNO: browser blocked for {host}. Only localhost and the profile's allowed preview hosts "
            f"({', '.join(sorted(allowed)) or 'none'}) may be opened; the same login may work on production.")


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
    tool = event.get("tool_name", "")
    if "browser" not in tool.lower() and "chrome" not in tool.lower():
        return 0
    root = os.environ.get("CLAUDE_PROJECT_DIR") or event.get("cwd") or os.getcwd()
    if not os.path.exists(os.path.join(root, ".uno", "zones")):
        return 0
    try:
        reason = check(root, find_url(event.get("tool_input") or {}))
    except Exception:
        return 0
    if not reason:
        return 0
    try:
        with open(os.path.join(root, ".uno", "audit.log"), "a", encoding="utf-8") as fh:
            fh.write(json.dumps({"time": datetime.now(timezone.utc).isoformat(timespec="seconds"),
                                 "session": event.get("session_id"), **who(event), "tool": tool,
                                 "url": str(find_url(event.get("tool_input") or {}))[:200],
                                 "action": "blocked", "rule": "browser-host"}, ensure_ascii=False) + "\n")
    except OSError:
        pass
    print(reason, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main())
