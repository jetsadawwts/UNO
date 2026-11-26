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
                  ~/UNO/data      lsof/ps, date   views, server   write HTML file
                        │              │               │
                        └──────► app (ports + use cases) ◄┘
                                       │
                                       ▼
                                    domain
```

| Crate | Owns | Depends on |
|---|---|---|
| `domain` | `Project`, `Task`, `AuditRecord`, `Event`/`EventKind`, counting rules (`Project::apply`, `finish`), agent names | std only |
| `app` | ports `ProjectStore`, `ManualSource`, `SessionProbe`, `Clock`; use cases `build_dashboard`, `signature` | `domain` |
| `adapters-fs` | `FsStore` (project folders, `audit.log`, `tasks.md`), `FsManual`, `manual_stamp`, parsers | `app`, `domain`, `serde_json` |
| `adapters-os` | `LsofSessions` (macOS), `SystemClock`, `count_sessions` | `app` |
| `adapters-web` | `render_page`, `PageMode`, `manual_html`, charts, CSS/JS, `Server` (127.0.0.1, Host check, GET only) | `app`, `domain`, `pulldown-cmark` |
| `adapters-file` | `write_atomic` | std only |
| root | `Cli`, wiring, `LocalClock`/`LocalSessions` (honour `UNO_REPORT_FAKE_NOW` / `UNO_REPORT_FAKE_SESSIONS`) | all of the above, `clap` |

## Rules
- `domain` and `app` never do I/O. Anything that touches the machine is an adapter.
- Adapters never depend on each other; only the root wires them.
- Text for people (English and Thai) is made in `adapters-web`; `domain` stores what happened (`EventKind`).

## Adding an adapter
1. Implement the port from `app::ports` in a new `crates/adapters-<name>` (depend on `app`, `domain`).
2. Test its parsing with plain strings, not the real machine.
3. Choose it in `src/main.rs`.

## Tests
- `cargo test --workspace` — unit tests per crate, `tests/golden.rs` (page and `--help`
  byte-identical to v0.5.12 for the fixtures in `tests/fixtures/data`), `tests/cli.rs`.
- Fixtures use invented names only. A golden change must be deliberate: regenerate with the
  command in `tests/golden.rs` and review the diff.
