#!/usr/bin/env python3
"""End-to-end check of the live page (run by hand; needs Python Playwright + Chromium).

    python3 tools/uno-report/tests/e2e_live.py [path/to/uno-report]

Starts `uno-report --serve` on a free port against a temporary copy of the fixtures and checks
the socket updates, the polling fallback and reconnecting. Prints "e2e: N/N passed".
"""
import json
import shutil
import socket
import subprocess
import sys
import tempfile
import time
from pathlib import Path

from playwright.sync_api import sync_playwright

HERE = Path(__file__).resolve().parent
BIN = sys.argv[1] if len(sys.argv) > 1 else str(HERE.parent / "target" / "release" / "uno-report")


def free_port():
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def start(port, data):
    return subprocess.Popen([BIN, "--serve", "--port", str(port), "--data", str(data)],
                            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


def append(path, task):
    with open(path, "a", encoding="utf-8") as f:
        f.write(json.dumps({"time": "2026-01-12T08:00:00+07:00", "event": "plan-approved", "task": task, "backend": "anthropic"}) + "\n")


# Every picker option has its pane, and every project pane has its option.
CONSISTENT = """(()=>{var o=[...document.querySelectorAll('#psel option')].map(x=>x.value).sort().join();
  var v=[...document.querySelectorAll('.pview')].map(x=>x.id).sort().join();return o===v})()"""
SHAPE = """[...document.querySelectorAll('#psel option')].map(x=>x.value).join()+' | '+
  [...document.querySelectorAll('.pview')].map(x=>x.id).join()"""


def until(fn, timeout):
    t0 = time.time()
    while time.time() - t0 < timeout:
        if fn():
            return time.time() - t0
        time.sleep(0.05)
    return None


def main():
    data = Path(tempfile.mkdtemp(prefix="uno-e2e-"))
    shutil.copytree(HERE / "fixtures" / "data", data, dirs_exist_ok=True)
    port = free_port()
    proc = start(port, data)
    time.sleep(0.8)
    results = []

    def check(name, ok, note=""):
        results.append(ok)
        print(("PASS " if ok else "FAIL ") + name + (f" ({note})" if note else ""))

    try:
        with sync_playwright() as p:
            b = p.chromium.launch()
            pg = b.new_page()
            errors = []
            pg.on("pageerror", lambda e: errors.append(str(e)))
            pg.goto(f"http://localhost:{port}/")
            pg.evaluate("""window.__states=[];new MutationObserver(()=>window.__states.push(
                document.getElementById('liveflag').dataset.state)).observe(
                document.getElementById('liveflag'),{attributes:true,attributeFilter:['data-state']})""")
            live = until(lambda: pg.get_attribute("#liveflag", "data-state") == "live", 3)
            check("connects, no page errors", live is not None and not errors, f"errors={errors}")

            pg.evaluate("window.__p0=document.getElementById('p0')")
            append(data / "alpha" / "audit.log", "T90")
            t = until(lambda: "Plan T90 approved" in pg.inner_text("#dash"), 5)
            same = pg.evaluate("window.__p0===document.getElementById('p0')")
            check("change shows within 1 s, in place", t is not None and t < 1.0 and same, f"t={t}, same_node={same}")

            time.sleep(1)
            before = pg.evaluate("window.unoSwaps||0")
            for i in range(50):
                append(data / "alpha" / "audit.log", f"T{100 + i}")
                time.sleep(0.005)
            until(lambda: "Plan T149 approved" in pg.inner_text("#dash"), 5)
            time.sleep(1)
            swaps = pg.evaluate("window.unoSwaps||0") - before
            check("50 quick writes → at most 3 swaps", 1 <= swaps <= 3, f"swaps={swaps}")

            proc.terminate()
            proc.wait()
            off = until(lambda: pg.get_attribute("#liveflag", "data-state") == "off", 10)
            states = pg.evaluate("window.__states")
            check("server stopped → Polling, then Offline", off is not None and "poll" in states, f"states={states[-4:]}")

            (data / "aaa").mkdir()  # sorts first: shifts every project pane while the socket is down
            (data / "aaa" / "tasks.md").write_text("| ID | Task | Status |\n|---|---|---|\n| T1 | Early | open |\n", encoding="utf-8")
            proc = start(port, data)
            back = until(lambda: pg.get_attribute("#liveflag", "data-state") == "live", 35)
            check("server back → Live again within 35 s", back is not None, f"t={back}")
            fixed = until(lambda: pg.evaluate(CONSISTENT) and "aaa" in pg.inner_text("#psel"), 5)
            check("reconnect after projects changed → page rebuilt, picker matches panes", fixed is not None, pg.evaluate(SHAPE))

            (data / "zeta").mkdir()
            (data / "zeta" / "tasks.md").write_text("| ID | Task | Status |\n|---|---|---|\n| T1 | New | open |\n", encoding="utf-8")
            added = until(lambda: "zeta" in pg.inner_text("#psel") and pg.evaluate(CONSISTENT), 5)
            check("new project → page lists it", added is not None and not errors, f"errors={errors}")
            b.close()
    finally:
        proc.terminate()
        shutil.rmtree(data, ignore_errors=True)
    print(f"e2e: {sum(results)}/{len(results)} passed")
    return 0 if all(results) else 1


if __name__ == "__main__":
    sys.exit(main())
