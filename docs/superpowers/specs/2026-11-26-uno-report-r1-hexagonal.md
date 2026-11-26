# uno-report R1 — Hexagonal workspace (spec)

Status: summary agreed with the owner (JWT) on 2026-11-26; this R1 slice needs `อนุมัติ` / `approve` before code.

## Why
`tools/uno-report/src/main.rs` is one 1,171-line file. R2 adds real-time updates (file watching, WebSocket) and R3 adds settings/config/logging. Before that, split the program so the core (counting tasks, events, approvals, team activity) knows nothing about files, `lsof`, HTML or HTTP.

## What R1 delivers
1. A Cargo workspace in `tools/uno-report` laid out as Ports & Adapters:
   - `crates/domain` — `Project`, `Task`, `Event`, `EventKind`, `AuditRecord`, status rules, agent names, aggregation. std only.
   - `crates/app` — ports (traits) and use cases `build_dashboard`, `signature`. Depends on `domain` only.
   - `crates/adapters-fs` — reads `~/UNO/data` (`audit.log`, `tasks.md`, `manual.md`, `manual.th.md`).
   - `crates/adapters-os` — Claude Code sessions via `lsof` + `ps` (macOS), system clock.
   - `crates/adapters-web` — HTML views, charts, CSS/JS, the current std-only HTTP server (`/`, `/version`).
   - `crates/adapters-file` — writes `uno-report.html` atomically.
   - Root package `uno-report` (`src/main.rs`) — composition root and CLI.
2. Dependency direction enforced by Cargo: `domain ← app ← adapters-* ← root`. No adapter depends on another adapter.
3. **Behaviour identical to v0.5.12**, proven by a golden test: the page rendered from fake fixture data, with a fixed clock and fixed sessions, is byte-identical before and after (after normalising the fixture's absolute path).
4. CLI flags, defaults, output path, help text, `cargo install --path ~/UNO/uno/tools/uno-report --force`, and the launchd plist all keep working unchanged.
5. Unit tests for domain rules, each adapter's parsing, and the use cases with fake adapters.

## Not in R1
tokio, axum, WebSocket, notify (R2) · askama, Alpine.js, htmx, config file, tracing, cargo audit/deny (R3) · any database · any visual change.

## Constraints
- Reads `~/UNO/data` only; nothing leaves the machine. Server binds 127.0.0.1 only, checks Host, GET only.
- Fixtures use invented project names and text only (no company names, no 13-digit numbers) — the UNO pre-commit data check must pass.
- New dependency allowed in R1: `clap` (derive) only. Existing: `serde_json`, `pulldown-cmark`.
- Commits are made by the owner: work happens in a clone in Claude's workspace; the result is handed over as a patch series for `git am`.
