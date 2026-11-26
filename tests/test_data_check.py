#!/usr/bin/env python3
"""Tests for scripts/check_no_company_data.py, each in a throwaway git repo."""
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

SCRIPT = Path(__file__).resolve().parent.parent / "scripts" / "check_no_company_data.py"
# Test data is assembled at run time so this file does not trip the check it tests.
DIGITS13 = "1234567" + "890123"
# shaped like a real Cargo.lock sha256 (64 lower-case hex) holding 13 digits in a row
HEX_WITH_ID = "52ce7f38b242319f7cabaacd" + DIGITS13 + "ecdc9d355bbb4ce0c68908cd813"
assert len(HEX_WITH_ID) == 64


def check(files, forbidden=()):
    """Stage `files` ({path: text}) in a new repo and run the data check. Returns its exit code."""
    d = Path(tempfile.mkdtemp(prefix="uno-dc-"))
    try:
        subprocess.run(["git", "init", "-q", str(d)], check=True)
        if forbidden:
            (d / ".forbidden-terms").write_text("\n".join(forbidden) + "\n", encoding="utf-8")
        for path, text in files.items():
            p = d / path
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_text(text, encoding="utf-8")
            subprocess.run(["git", "add", path], cwd=d, check=True)
        return subprocess.run([sys.executable, str(SCRIPT)], cwd=d, capture_output=True, text=True).returncode
    finally:
        shutil.rmtree(d)


CASES = [
    ("checksum line in Cargo.lock passes", {"tools/x/Cargo.lock": f'checksum = "{HEX_WITH_ID}"\n'}, (), 0),
    ("same line in another file is caught", {"notes.md": f'checksum = "{HEX_WITH_ID}"\n'}, (), 1),
    ("short checksum in Cargo.lock is caught", {"Cargo.lock": f'checksum = "{DIGITS13}"\n'}, (), 1),
    ("other 13-digit line in Cargo.lock is caught", {"Cargo.lock": f'name = "pkg-{DIGITS13}"\n'}, (), 1),
    ("forbidden term on a checksum line is caught", {"Cargo.lock": f'checksum = "{HEX_WITH_ID}" # acmehosp\n'}, ("acmehosp",), 1),
    ("plain line passes", {"README.md": "hello\n"}, (), 0),
]


def main():
    failed = 0
    for name, files, forbidden, want in CASES:
        got = check(files, forbidden)
        if got != want:
            failed += 1
            print(f"FAIL data check: {name}: got {got}, want {want}")
    print(f"UNO data check tests: {len(CASES) - failed}/{len(CASES)} passed")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
