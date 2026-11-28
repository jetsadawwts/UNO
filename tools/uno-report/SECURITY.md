# uno-report — what it does on this machine

For IT / security review, and for anti-malware alerts. Applies from UNO v0.5.17.

## Summary
`uno-report` is a local dashboard built from source in this repository (`cargo install --path tools/uno-report --locked`). It reads UNO's own log and task files under `~/UNO/data`, writes one HTML file there, and can serve that page on `127.0.0.1` only. It has no code that sends data off the machine.

## What it reads
- `~/UNO/data/<project>/audit.log`, `tasks.md`, `heartbeat/*.json` and, since v0.5.22, `office.log` (written by UNO's Claude Code hooks), `~/UNO/data/manual.md` and `manual.th.md`.
- Nothing else: no Keychain, browser profiles, cookies, SSH or cloud credentials, environment secrets (it reads only `HOME` and two `UNO_REPORT_FAKE_*` test variables).

## What it writes
- `~/UNO/data/uno-report.html` (via `uno-report.html.tmp`, then rename). Nothing else.
- Since v0.5.22 the page embeds the Office tab art (WebP files in `assets/office/`, built into the binary, inlined as `data:` URLs); the page loads nothing from outside.

## Processes it starts
- `date` (local time for the page header) and, only with `--open`, `open <file or http://localhost:PORT/>`.
- Since v0.5.17 it no longer runs `lsof` or `ps`; open Claude Code sessions are counted from UNO's hook events in `audit.log` and, since v0.5.19, `heartbeat/*.json` (time, session id, tool name, agent id and, since v0.5.20, state, since when and task id only; since v0.5.21 also the plugin version; since v0.5.22 one file per running subagent, `<session>.<agent_id>.json`, removed when it stops; and `office.log`, one line per subagent start, tool, stop and returned answer: time, session id, agent id, agent type, step and tool name only, trimmed to the newest 1,000 lines past 1 MB). Since v0.5.22 the guards' `audit.log` lines also carry `name`: `uno` or the member's agent type, nothing else.
- Separate from `uno-report`: since v0.5.20 the plugin's `trace_heartbeat.py` hook runs `git rev-parse --abbrev-ref HEAD` in the project to read the task id from the branch name; only the id is stored.

## Network
- `--serve` listens on `127.0.0.1:<port>` (default 8765) only. Every request must carry `Host: localhost:<port>` or `127.0.0.1:<port>`; `/ws` also refuses a foreign `Origin`; GET only; at most 8 WebSocket connections.
- No outgoing connections: no HTTP client in the code or its dependencies (`hyper` is built with the `server` feature only).

## Persistence
- None by default. `launchd/com.uno.report.plist` is optional: it starts `uno-report --serve` at login (`RunAtLoad`) and, since v0.5.17, does not restart it when stopped (no `KeepAlive`).

## Files watched
- `~/UNO/data` (FSEvents via the `notify` crate) to refresh the live page. Reads (Access events) are ignored.

## How to check a build
```
codesign -dv ~/.cargo/bin/uno-report        # arm64 binaries are ad-hoc "linker-signed" by Apple's linker
shasum -a 256 ~/.cargo/bin/uno-report
cargo audit                                  # in tools/uno-report; RustSec advisories
grep -rn 'Command::new' tools/uno-report/crates tools/uno-report/src
```

## Audit of 2026-11-27 (v0.5.16 → v0.5.17)
- Source: every `Command::new` is `date`, `open`, and the removed `lsof`/`ps`; no `unsafe`; the only file write is the HTML file above; no network client; no access to credential stores.
- Dependencies: 90 crates in the release build (`cargo tree -e normal`), 108 in `Cargo.lock` with dev and other-platform crates; `cargo audit` against 1,568 RustSec advisories: no vulnerabilities. Versions are pinned by `Cargo.lock` and installed with `--locked`; cargo verifies each crate's checksum on download.
- Anti-malware: CleanMyMac reported the v0.5.15 binary as an infostealer ("AvengerAgent"). No public description of that name was found; the binary matched a behaviour pattern (unsigned-looking binary, login item that restarts itself, process listing, home-folder watching). v0.5.17 removes the process listing and the restart.
