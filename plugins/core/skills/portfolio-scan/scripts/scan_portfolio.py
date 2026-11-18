#!/usr/bin/env python3
"""Scan a folder of legacy repos and produce a triage inventory for UNO onboarding.

Read-only: never modifies any repo. Uses only the Python standard library and git.

Usage:
    python3 scan_portfolio.py <parent-folder> [--out inventory.md] [--json projects.json]
"""
import argparse
import json
import os
import re
import subprocess
import sys
from collections import Counter
from datetime import datetime, timezone

SKIP_DIRS = {".git", "node_modules", "vendor", "dist", "build", "bin", "obj", ".venv",
             "venv", "__pycache__", ".next", "target", "packages", ".idea", ".vs"}

LANG_BY_EXT = {
    ".py": "Python", ".js": "JavaScript", ".jsx": "JavaScript", ".ts": "TypeScript",
    ".tsx": "TypeScript", ".cs": "C#", ".java": "Java", ".kt": "Kotlin", ".go": "Go",
    ".php": "PHP", ".rb": "Ruby", ".vb": "VB.NET", ".sql": "SQL", ".vue": "Vue",
    ".dart": "Dart", ".swift": "Swift", ".cpp": "C++", ".c": "C", ".rs": "Rust",
}

# Signals that a codebase handles personal or sensitive data (health, identity, finance, HR).
# A hit means "needs sensitive-tier review", not proof.
PHI_PATTERNS = re.compile(
    r"\b(passport|ssn\b|tax_?id|salary|payroll|credit_?card|card_?number|bank_?account|iban\b|employee_?id|"
    r"patient|hn\b|hospital_?number|national_?id|citizen_?id|\bcid\b|id_?card|"
    r"hl7|fhir|diagnos|icd_?10|date_?of_?birth|\bdob\b|allerg|prescri|lab_?result|"
    r"medical_?record|\bemr\b|\behr\b|\bhis\b)|ผู้ป่วย|เลขบัตรประชาชน|การวินิจฉัย",
    re.IGNORECASE,
)

TEST_HINTS = re.compile(r"(^|/)(tests?|__tests__|spec|specs)(/|$)|(_test|\.test|\.spec|Tests?)\.[a-z]+$",
                        re.IGNORECASE)

MAX_FILE_BYTES = 512 * 1024


def git(repo, *args):
    try:
        out = subprocess.run(["git", "-C", repo, *args], capture_output=True, text=True, timeout=30)
        return out.stdout.strip() if out.returncode == 0 else ""
    except (OSError, subprocess.TimeoutExpired):
        return ""


def ignored_dirs(path):
    """Simple top-level directory names from .gitignore (e.g. build output like `log/`)."""
    names = set()
    try:
        with open(os.path.join(path, ".gitignore"), encoding="utf-8", errors="ignore") as fh:
            for raw in fh:
                line = raw.strip()
                if not line or line.startswith(("#", "!")) or any(c in line for c in "*?["):
                    continue
                name = line.strip("/")
                if name and "/" not in name:
                    names.add(name)
    except OSError:
        pass
    return names


def scan_repo(path):
    skip_here = SKIP_DIRS | ignored_dirs(path)
    langs = Counter()
    loc = 0
    files = 0
    test_files = 0
    phi_files = []
    has_ci = False
    has_docs = os.path.exists(os.path.join(path, "README.md")) or os.path.isdir(os.path.join(path, "docs"))
    has_claude = os.path.exists(os.path.join(path, "CLAUDE.md")) or os.path.exists(os.path.join(path, "AGENTS.md"))

    for root, dirs, names in os.walk(path):
        top = root == path
        dirs[:] = [d for d in dirs
                   if (d not in SKIP_DIRS and not (top and d in skip_here) and not d.startswith("."))
                   or d in (".github", ".gitlab")]
        rel_root = os.path.relpath(root, path).replace(os.sep, "/")
        if rel_root.startswith((".github/workflows", ".gitlab")):
            has_ci = True
        for name in names:
            if name in (".gitlab-ci.yml", "Jenkinsfile", "azure-pipelines.yml", "bitbucket-pipelines.yml"):
                has_ci = True
            ext = os.path.splitext(name)[1].lower()
            lang = LANG_BY_EXT.get(ext)
            if not lang:
                continue
            fp = os.path.join(root, name)
            rel = f"{rel_root}/{name}" if rel_root != "." else name
            files += 1
            if TEST_HINTS.search(rel):
                test_files += 1
            try:
                if os.path.getsize(fp) > MAX_FILE_BYTES:
                    continue
                with open(fp, "r", encoding="utf-8", errors="ignore") as fh:
                    text = fh.read()
            except OSError:
                continue
            n = text.count("\n") + 1
            loc += n
            langs[lang] += n
            if PHI_PATTERNS.search(text):
                phi_files.append(rel)

    last_commit = git(path, "log", "-1", "--format=%cI")
    commits_90d = git(path, "rev-list", "--count", "--since=90.days", "HEAD")
    authors_90d = git(path, "shortlog", "-sne", "--since=90.days", "HEAD")
    days_since = None
    if last_commit:
        try:
            dt = datetime.fromisoformat(last_commit)
            days_since = (datetime.now(timezone.utc) - dt).days
        except ValueError:
            pass

    return {
        "name": os.path.basename(path.rstrip("/")),
        "path": path,
        "is_git": bool(git(path, "rev-parse", "--git-dir")),
        "languages": [l for l, _ in langs.most_common(3)],
        "loc": loc,
        "source_files": files,
        "test_files": test_files,
        "test_ratio": round(test_files / files, 2) if files else 0,
        "phi_signal_files": len(phi_files),
        "phi_examples": phi_files[:5],
        "has_ci": has_ci,
        "has_docs": has_docs,
        "has_agent_rules": has_claude,
        "last_commit": last_commit[:10] if last_commit else None,
        "days_since_commit": days_since,
        "commits_90d": int(commits_90d) if commits_90d.isdigit() else 0,
        "active_authors_90d": len(authors_90d.splitlines()) if authors_90d else 0,
    }


def classify(p):
    """Suggest tier and onboarding wave. A human confirms both in registry/projects.yaml."""
    sensitive = p["phi_signal_files"] > 0
    active = p["commits_90d"] > 0
    tier = "sensitive" if sensitive else "internal"  # `open` is set by a human
    if not p["is_git"]:
        wave, why = "hold", "not a git repo: put under version control first"
    elif active and not sensitive:
        wave, why = "1", "active, no personal-data signals: safest place to tune UNO"
    elif active and sensitive:
        wave, why = "2", "active, touches personal/sensitive data: onboard after wave 1 lessons"
    else:
        wave, why = "3", "inactive: onboard only when work arrives"
    flags = []
    if p["test_ratio"] < 0.05:
        flags.append("few/no tests")
    if not p["has_ci"]:
        flags.append("no CI")
    if p["active_authors_90d"] <= 1 and active:
        flags.append("bus factor 1")
    if p["loc"] > 100_000:
        flags.append("large: split discovery by module")
    return tier, wave, why, flags


def to_markdown(rows):
    out = ["# UNO portfolio inventory", "",
           f"Scanned {datetime.now().strftime('%Y-%m-%d %H:%M')} — {len(rows)} projects. "
           "Tier and wave are suggestions; confirm them in `registry/projects.yaml`.", "",
           "| Wave | Project | Tier | Languages | LOC | Tests | Commits 90d | Last commit | Flags |",
           "|---|---|---|---|---:|---:|---:|---|---|"]
    order = {"1": 0, "2": 1, "3": 2, "hold": 3}
    for p in sorted(rows, key=lambda r: (order[r["wave"]], -r["commits_90d"])):
        out.append(
            f"| {p['wave']} | {p['name']} | {p['tier']} | {', '.join(p['languages']) or '-'} | "
            f"{p['loc']:,} | {int(p['test_ratio'] * 100)}% | {p['commits_90d']} | "
            f"{p['last_commit'] or '-'} | {'; '.join(p['flags']) or '-'} |")
    out += ["", "## Patient-data signals (verify by hand)", ""]
    for p in rows:
        if p["phi_signal_files"]:
            out.append(f"- **{p['name']}**: {p['phi_signal_files']} files, e.g. "
                       + ", ".join(f"`{e}`" for e in p["phi_examples"]))
    out += ["", "## Why each wave", ""]
    for p in rows:
        out.append(f"- **{p['name']}** → wave {p['wave']}: {p['why']}")
    return "\n".join(out) + "\n"


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("parent")
    ap.add_argument("--out", default="inventory.md")
    ap.add_argument("--json", default=None)
    args = ap.parse_args()

    if not os.path.isdir(args.parent):
        sys.exit(f"Not a folder: {args.parent}")

    rows = []
    for entry in sorted(os.listdir(args.parent)):
        path = os.path.join(args.parent, entry)
        if not os.path.isdir(path) or entry.startswith("."):
            continue
        p = scan_repo(path)
        p["tier"], p["wave"], p["why"], p["flags"] = classify(p)
        rows.append(p)
        print(f"scanned {p['name']}: wave {p['wave']}, {p['tier']}", file=sys.stderr)

    with open(args.out, "w", encoding="utf-8") as fh:
        fh.write(to_markdown(rows))
    if args.json:
        with open(args.json, "w", encoding="utf-8") as fh:
            json.dump(rows, fh, indent=2, ensure_ascii=False)
    print(f"wrote {args.out}", file=sys.stderr)


if __name__ == "__main__":
    main()
