# uno-report architecture

Ports & Adapters (hexagonal). The core knows nothing about files, `lsof`, HTML or HTTP; the
outside world plugs in through traits. Cargo enforces the direction: a crate cannot use what
it does not depend on.

```
                 ┌──────────── root package `uno-report` (src/main.rs) ────────────┐
                 │  CLI (clap) · picks adapters · test seams UNO_REPORT_FAKE_*    │
                 └──────┬──────────────┬───────────────┬──────────────┬───────────┘
                        ▼              ▼               ▼              ▼
                  adapters-fs     adapters-os     adapters-web    adapters-file
                  ~/UNO/data      date            views, server   write HTML file
                        │              │               │
                        └──────► app (ports + use cases) ◄┘
                                       │
                                       ▼
                                    domain
```

| Crate | Owns | Depends on |
|---|---|---|
| `domain` | `Project`, `Task`, `AuditRecord`, `Event`/`EventKind`, counting rules (`Project::apply`, `finish`), agent names, `open_sessions`, `parse_time` | std only |
| `app` | ports `ProjectStore`, `ManualSource`, `SessionProbe`, `Clock`, `ChangeFeed`; use cases `build_dashboard`, `signature`, `Watch` | `domain` |
| `adapters-fs` | `FsStore` (project folders, `audit.log`, `tasks.md`), `FsManual`, `manual_stamp`, parsers, `FsChangeFeed` (notify), `AuditSessions` (open sessions from audit events) | `app`, `domain`, `serde_json`, `notify` |
| `adapters-os` | `SystemClock` | `app` |
| `adapters-web` | `render_page`, `PageMode`, `manual_html`, `render_panes`, `diff_message`, charts, CSS/JS, `Server` (axum: 127.0.0.1, Host check, GET only, `/ws`) | `app`, `domain`, `pulldown-cmark`, `serde_json`, `tokio`, `axum` |
| `adapters-file` | `write_atomic` | std only |
| root | `Cli`, wiring, `LocalClock`/`LocalSessions` (honour `UNO_REPORT_FAKE_NOW` / `UNO_REPORT_FAKE_SESSIONS`), watcher thread, tokio runtime for `--serve` | all of the above, `clap`, `tokio` |

## Live updates (`--serve`)
```
FsChangeFeed (notify, 300 ms debounce) ──┐
every 5 s (session expiry) ──────────────┴─► Watch::step → new signature ─► tokio watch channel
                                                                                │
browser ◄── /ws: {"sig","panes":{changed only},"meta"} or {"sig","reload":true} ◄┘ per socket
```
- Panes: `psel` (picker options), `all`, `p0`…, `manual`. Each socket remembers what it last sent and sends only the panes whose HTML changed; a different set of panes (project added or removed) sends `reload`.
- The page swaps panes in place, keeping the selected project, language and open sections. If the socket drops it polls `/version` and reconnects after 1, 2, 4 … 30 s. Header: Live / Polling / Disconnected.
- Reads are not changes: the watcher ignores `Access` events (rendering opens every file), `uno-report.html`, `*.tmp` and `trash/`.
- Safety: 127.0.0.1 only; Host checked on every route; `/ws` refuses a foreign `Origin`; at most 8 sockets; the server never acts on what the browser sends.
- `tests/e2e_live.py` (Playwright, run by hand) checks the whole loop.

## Open sessions
UNO's `trace_session.py` hook writes `session-start` / `session-end` to the project's `audit.log`, and every hook event carries the session id. A session counts as open when its latest event is not `session-end` and is at most 30 minutes old (`SESSION_IDLE_SECS`: Claude Code can exit without running SessionEnd). uno-report does not look at other processes on the machine.

## Rules
- `domain` and `app` never do I/O. Anything that touches the machine is an adapter.
- Adapters never depend on each other; only the root wires them.
- Text for people (English and Thai) is made in `adapters-web`; `domain` stores what happened (`EventKind`).

## Adding an adapter
1. Implement the port from `app::ports` in a new `crates/adapters-<name>` (depend on `app`, `domain`).
2. Test its parsing with plain strings, not the real machine.
3. Choose it in `src/main.rs`.

## Build
- Rust 1.91 or newer (`rust-version` in `Cargo.toml`). Install with `cargo install --path tools/uno-report --force --locked` so `Cargo.lock` is used.

## Tests
- `cargo test --workspace` — unit tests per crate, `tests/golden.rs` (page and `--help`
  byte-identical to v0.5.12 for the fixtures in `tests/fixtures/data`), `tests/cli.rs`.
- Fixtures use invented names only. A golden change must be deliberate: regenerate with the
  command in `tests/golden.rs` and review the diff.
