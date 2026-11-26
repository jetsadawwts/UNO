# uno-report R2 — real-time live dashboard (spec)

Status: direction agreed with the owner (JWT) on 2026-11-26 (WebSocket, Hexagonal, full stack); this R2 slice needs `approve` before code.

## Why
`uno-report --serve` polls `/version` every 2–10 s and reloads the whole dashboard. JWT wants updates within a second, and only the part that changed.

## What R2 delivers
1. **Change detection:** the server watches `~/UNO/data` (macOS FSEvents via `notify`) and re-checks open Claude Code sessions every 5 s. Bursts of file events within 300 ms count as one change. Its own output (`uno-report.html`, `*.tmp`) and `trash/` folders do not count.
2. **Push:** `GET /ws` (WebSocket). On connect and on every change the server sends one JSON text message: `{"sig": "<16 hex>", "panes": {"<id>": "<inner HTML>", …}, "meta": "<header meta HTML>"}` with only the panes whose HTML changed since that connection's last message. If the set of panes changed (a project added or removed) it sends `{"sig": …, "reload": true}` and the page does a full refresh. The browser never sends anything that the server acts on.
3. **Page:** connects to `/ws`; applies panes in place, keeping the selected project, language, theme and scroll. If the socket drops: polls `/version` as today and retries the socket after 1, 2, 4 … 30 s. Header shows 🟢 Live / 🟡 Polling / ⚪ Offline (EN/TH).
4. **Server:** tokio + axum 0.8, still bound to 127.0.0.1 only; Host must be `localhost:<port>` or `127.0.0.1:<port>` on every route (403); `/ws` also rejects an `Origin` that is not `http://localhost:<port>` or `http://127.0.0.1:<port>` (403; no Origin header is allowed — only local programs omit it); at most 8 sockets (503 for the 9th); ping every 20 s; non-GET → 405; same response headers as v0.5.13 (`Cache-Control: no-store`, `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`).
5. **Unchanged:** file mode (`--open`, `--watch`) output, CLI flags, `cargo install` path, launchd plist, `/` and `/version`.

## Not in R2
htmx / Alpine.js, askama, config file, tracing, cargo audit/deny (R3) · any database · any button that changes data.

## Constraints
- Hexagonal rules from R1 hold: `domain`/`app` do no I/O and know no tokio/notify; adapters do not depend on each other.
- New dependencies allowed: `tokio` (rt-multi-thread, net, sync, time, macros), `axum` 0.8 (`ws`, `http1`, `tokio`, no other defaults), `notify` 8; dev-only `tokio-tungstenite` for socket tests.
- Fixtures: invented names only; the UNO data check must pass.
