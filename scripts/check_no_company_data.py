#!/usr/bin/env python3
"""Block commits to the UNO repo that would carry company or personal data.

Checks the lines being added in staged files for:
  - names and repo owners of projects in registry/projects.yaml (local, not committed)
  - extra terms in .forbidden-terms (local, one per line: company, client, hospital names...)
  - e-mail addresses other than the allowed ones
  - 13-digit national ID numbers and long secret-looking strings
Exit 1 lists the findings; nothing is changed. Bypass only after checking: git commit --no-verify
"""
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(subprocess.run(["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True).stdout.strip() or ".")
ALLOWED_EMAILS = {"jetsadawwts@gmail.com", "noreply@anthropic.com"}
ALLOWED_EMAIL_DOMAINS = ("example.com", "example.org")
EMAIL = re.compile(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}")
THAI_ID = re.compile(r"(?<!\d)\d{13}(?!\d)")
SECRET = re.compile(r"(?<![A-Za-z0-9+/=_-])[A-Za-z0-9+/=_-]{40,}(?![A-Za-z0-9+/=_-])")
SKIP_FILES = {"scripts/check_no_company_data.py"}


def looks_secret(run):
    """Random-looking: has digits, upper and lower case, and is not a file path."""
    return (run.count("/") < 3 and any(c.isdigit() for c in run)
            and any(c.isupper() for c in run) and any(c.islower() for c in run))


def terms():
    found = set()
    reg = ROOT / "registry" / "projects.yaml"
    if reg.exists():
        for line in reg.read_text(encoding="utf-8").splitlines():
            m = re.match(r"\s*-?\s*(name|repo|owner):\s*([^#\s]+)", line)
            if m and not m.group(2).startswith("<"):
                for part in re.split(r"[/:]", m.group(2)):
                    if len(part) >= 4:
                        found.add(part.lower())
    extra = ROOT / ".forbidden-terms"
    if extra.exists():
        found |= {t.strip().lower() for t in extra.read_text(encoding="utf-8").splitlines()
                  if t.strip() and not t.startswith("#")}
    return found


def added_lines():
    diff = subprocess.run(["git", "diff", "--cached", "-U0", "--no-color"], capture_output=True, text=True, cwd=ROOT).stdout
    current = None
    for line in diff.splitlines():
        if line.startswith("+++ "):
            current = line[6:] if line.startswith("+++ b/") else None
        elif line.startswith("+") and not line.startswith("+++") and current and current not in SKIP_FILES:
            yield current, line[1:]


def added_files():
    out = subprocess.run(["git", "diff", "--cached", "--name-only", "--diff-filter=A"], capture_output=True, text=True, cwd=ROOT).stdout
    return [p for p in out.splitlines() if p]


def main():
    bad = []
    for p in added_files():
        if p.split("/")[-1] in (".DS_Store", "Thumbs.db"):
            bad.append((p, "OS junk file (folder listing)", p))
    words = terms()
    for path, text in added_lines():
        low = text.lower()
        for w in words:
            if w in low:
                bad.append((path, f"company/project term '{w}'", text))
        for e in EMAIL.findall(text):
            if e.lower() not in ALLOWED_EMAILS and not e.lower().endswith(ALLOWED_EMAIL_DOMAINS):
                bad.append((path, f"e-mail '{e}'", text))
        if THAI_ID.search(text):
            bad.append((path, "13-digit ID number", text))
        if any(looks_secret(m) for m in SECRET.findall(text)):
            bad.append((path, "long secret-like string", text))
    if not bad:
        return 0
    print("UNO data check: this commit may carry company or personal data:\n", file=sys.stderr)
    for path, why, text in bad[:30]:
        print(f"  {path}: {why}\n      {text.strip()[:120]}", file=sys.stderr)
    print("\nRewrite it as a generic rule, or add the file to .gitignore. "
          "If it is a false positive, commit with --no-verify after checking.", file=sys.stderr)
    return 1


if __name__ == "__main__":
    sys.exit(main())
