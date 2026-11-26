# uno-report R2 — Real-time Live Dashboard Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `uno-report --serve` pushes only the changed parts of the dashboard over a WebSocket within 1 s of a change in `~/UNO/data`, falling back to polling.

**Architecture:** A `ChangeFeed` port (app) is implemented by a `notify` watcher (adapters-fs). A `Watch` use case (app) turns "files changed or 5 s passed" into "signature changed". The web adapter moves to tokio + axum, renders the dashboard as named panes, and sends each socket the panes that differ from what it last sent.

**Tech Stack:** Rust 2021 workspace from R1; tokio 1, axum 0.8 (`ws`), notify 8; dev: tokio-tungstenite.

**Spec:** `docs/superpowers/specs/2026-11-27-uno-report-r2-realtime.md`

## Global Constraints
- File mode output stays byte-identical to v0.5.13 **except** the inline script: the old polling block moves out of `JS` into a live-only `LIVE_JS`. The golden file is regenerated once, in Task 4, and its diff must touch only the `<script>` after `</main>`.
- `domain` and `app` gain no dependencies. `notify` only in `adapters-fs`; `tokio`/`axum` only in `adapters-web` and the root.
- Dependencies (exact features):
  - `tokio = { version = "1", features = ["rt-multi-thread", "net", "sync", "time", "macros"] }`
  - `axum = { version = "0.8", default-features = false, features = ["http1", "tokio", "ws"] }`
  - `notify = { version = "8", default-features = false, features = ["macos_fsevent"] }`
  - dev only, in adapters-web: `tokio-tungstenite = "0.28"` (or the version axum 0.8 resolves; ledger it)
- Server rules: bind `127.0.0.1`; Host ∈ {`localhost:<port>`, `127.0.0.1:<port>`} on every route else 403; `/ws` Origin ∈ {`http://localhost:<port>`, `http://127.0.0.1:<port>`} or absent, else 403; > 8 open sockets → 503; non-GET → 405; headers `Cache-Control: no-store`, `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer` on every HTTP response; client messages are read and dropped; ping every 20 s.
- Debounce 300 ms; session re-check every 5 s; reconnect back-off 1, 2, 4, 8, 16, 30 s (cap 30).
- Ignored paths for change detection: file name `uno-report.html`, any name ending `.tmp`, any path with a `trash` component.
- Pane ids: `psel` (the `<option>` list), `all`, `p0`…`p<N-1>` (projects in name order), `manual`. Message JSON: `{"sig":…, "panes":{…}, "meta":…}` or `{"sig":…, "reload":true}`.
- Test command (in `tools/uno-report`): `cargo test --workspace`. Before the last commit: `cargo clippy --workspace --all-targets -- -D warnings`.
- Execution: clone in Claude's workspace (tree = `main` at v0.5.14 after D1 is applied), branch `r2-realtime`; hand-over with `git format-patch`.

## Review Focus
1. A burst of writes (UNO hooks append several lines in a row) → one push, not dozens. Test in Task 2 (debounce) and Task 5 (e2e: 50 writes → ≤ 3 swaps).
2. The data folder is deleted or recreated while serving → watcher keeps working or the page falls back to polling; no crash. Test in Task 2 (`watch_survives_root_recreated`).
3. A second browser tab / reconnect → each socket gets the full pane set on connect, then only diffs. Test in Task 3 (`first_message_has_all_panes`, `second_message_has_only_changed`).
4. A project added or removed while connected → `reload:true`, the page does a full refresh. Test in Task 3.
5. A web page on another origin opening `ws://localhost:8765/ws` → 403 before upgrade. Test in Task 3.

---

### Task 1: `ChangeFeed` port and `Watch` use case (app)

**Files:**
- Modify: `crates/app/src/ports.rs`, `crates/app/src/lib.rs`
- Create: `crates/app/src/watch.rs`

**Interfaces:**
- Produces:
```rust
pub trait ChangeFeed { /// Block up to `timeout`; true when watched files changed (already debounced).
    fn wait(&self, timeout: std::time::Duration) -> bool; }
pub struct Watch { last: Option<String> }
impl Watch {
    pub fn new() -> Self;
    /// One round: wait on the feed (up to `tick`), recompute the signature, and return it only when it differs from the last one returned (the first call always returns it).
    pub fn step(&mut self, feed: &impl ChangeFeed, store: &impl ProjectStore, manual_stamp: &str, sessions: &impl SessionProbe, tick: Duration) -> Option<String>;
}
```
- Consumes: R1 `app::signature`.

- [ ] **Step 1: Write failing tests** in `watch.rs` with `FakeFeed(Cell<Vec<bool>>)`, `FakeStore` (stamp from a `Cell<String>`), `FakeSessions`:
  - `first_step_returns_signature`
  - `unchanged_returns_none` — two steps, same inputs → second is `None`
  - `store_change_returns_new_sig` — change stamp between steps → `Some(new)`, `new != first`
  - `session_change_without_file_event_returns_new_sig` — feed returns `false` (timeout), sessions change → `Some`
  - `feed_timeout_is_passed_through` — FakeFeed records the `Duration` it was given; equals `tick`
- [ ] **Step 2: Run** `cargo test -p app` — Expected: FAIL (missing items).
- [ ] **Step 3: Implement** `Watch` (`Default` too, for clippy).
- [ ] **Step 4: Run** `cargo test --workspace` — Expected: all pass.
- [ ] **Step 5: Commit** `feat(uno-report): ChangeFeed port and Watch use case (R2)`.

---

### Task 2: `FsChangeFeed` (adapters-fs, notify)

**Files:**
- Modify: `crates/adapters-fs/Cargo.toml` (+ `notify`), `crates/adapters-fs/src/lib.rs`
- Create: `crates/adapters-fs/src/watch.rs`, test `crates/adapters-fs/tests/watch.rs`

**Interfaces:**
- Produces: `pub struct FsChangeFeed` with `pub fn new(root: &Path, manual_en: &Path) -> Result<Self, String>` (watches `root` recursively and the manual's folder non-recursively) implementing `ChangeFeed`; pure `pub fn relevant(path: &Path) -> bool` (Global Constraints ignore rules).
- `wait(timeout)`: block for the first relevant event up to `timeout`; after one arrives keep draining until 300 ms pass with no relevant event; return true. Timeout with nothing → false. If the watcher reports an error or the root disappears, re-create the watcher on the next `wait` (once per call) and return true so the caller re-checks.

- [ ] **Step 1: Write failing tests:**
  - `relevant_ignores_own_output_tmp_and_trash`: `uno-report.html` false, `x/uno-report.html.tmp` false, `alpha/trash/2026/a.txt` false, `alpha/audit.log` true, `manual.th.md` true.
  - `write_is_seen_within_a_second` (temp dir): spawn a thread that appends to `alpha/audit.log` after 100 ms; `wait(3s)` returns true in < 1 s total.
  - `burst_is_one_change`: 50 appends 5 ms apart; first `wait(3s)` → true; the next `wait(400ms)` → false.
  - `own_output_is_not_a_change`: write `uno-report.html` → `wait(800ms)` false.
  - `watch_survives_root_recreated`: remove and recreate the temp root, then write `alpha/audit.log` → some `wait(3s)` within two calls returns true.
- [ ] **Step 2: Run** `cargo test -p adapters-fs --test watch` — Expected: FAIL.
- [ ] **Step 3: Implement** with `notify::recommended_watcher` sending into a `std::sync::mpsc` channel.
- [ ] **Step 4: Run** `cargo test --workspace` — Expected: all pass. (FSEvents latency on macOS can exceed Linux inotify; the 1 s bound is checked again on the Mac in Task 5.)
- [ ] **Step 5: Commit** `feat(uno-report): FsChangeFeed watches ~/UNO/data with notify (R2)`.

---

### Task 3: Panes and the axum server (adapters-web)

**Files:**
- Modify: `crates/adapters-web/Cargo.toml` (+ tokio, axum; dev tokio-tungstenite), `src/views.rs`, `src/lib.rs`, `src/server.rs` (rewritten)
- Create: `crates/adapters-web/src/panes.rs`, `crates/adapters-web/tests/server.rs`; remove the old `handle`-based tests from `tests/web.rs` that the new tests replace (keep the render/manual tests)

**Interfaces:**
- Produces:
  - `pub fn render_panes(d: &Dashboard, m: &ManualText, data_label: &str) -> Panes` where `pub struct Panes { pub panes: BTreeMap<String, String>, pub meta: String }` — every pane's inner HTML exactly as it appears inside `render_page` (`dashboard()` and `render_page` are refactored to build from the same pieces; golden must not change in this task).
  - `pub fn diff_message(sig: &str, prev: Option<&Panes>, next: &Panes) -> String` — JSON per Global Constraints: no `prev` → all panes; same key set → changed panes only (`"panes":{}` allowed); different key set → `reload:true`.
  - `pub struct Server<S, M, P, C, F>` — same fields as R1 plus `pub changes: tokio::sync::watch::Receiver<String>` (the latest signature); `pub async fn run(self, open: bool)`; `pub fn router(self) -> axum::Router` for tests.
- Consumes: Task 1 `Watch` is driven by the root (Task 4) and feeds the `watch::Sender<String>`.

- [ ] **Step 1: Write failing tests:**
  - `panes.rs` unit: `panes_match_page` (every pane HTML is a substring of `render_page` for the fixtures); `diff_first_is_full`; `diff_same_keys_only_changed`; `diff_new_project_is_reload`; `message_is_valid_json` (parse with a 20-line hand parser is not allowed — add `serde_json` as a dev-dependency and parse).
  - `tests/server.rs` (tokio, real listener on port 0): `root_ok_and_headers` (200, three headers present); `foreign_host_403` for `/`, `/version`, `/ws`; `post_405`; `unknown_404`; `ws_foreign_origin_403`; `ws_no_origin_ok`; `first_message_has_all_panes`; `second_message_has_only_changed` (send a new value on the `watch::Sender` after changing the fake store → message with only `all`/`p0`); `project_added_sends_reload`; `ninth_socket_503`; `client_text_is_ignored` (send "hello" → no reply, socket stays open).
- [ ] **Step 2: Run** `cargo test -p adapters-web` — Expected: FAIL.
- [ ] **Step 3: Implement** panes (refactor `dashboard()` to build `psel` options and pane bodies once, used by both `render_page` and `render_panes`), then the axum router: a Host-check middleware on all routes, `/ws` with Origin check, an `AtomicUsize` socket counter (decremented on drop), per-socket task that keeps its last `Panes` and on each `changes.changed()` renders on `spawn_blocking` and sends `diff_message`.
- [ ] **Step 4: Run** `cargo test --workspace` — Expected: all pass; golden 2/2 unchanged.
- [ ] **Step 5: Commit** `feat(uno-report): pane diff over WebSocket on axum (R2)`.

---

### Task 4: Browser side and wiring (root + assets)

**Files:**
- Modify: `crates/adapters-web/src/assets.rs` (move the `if(window.UNO_LIVE){…}` block out of `JS` into `LIVE_JS`), `src/views.rs` (append `LIVE_JS` only in `PageMode::Live`; live flag gets three states), `src/main.rs` (tokio runtime for `--serve`; a std thread runs `Watch::step` in a loop with `FsChangeFeed` and tick 5 s and sends into `watch::Sender`), `tests/golden/report.html` (regenerated once)

**Interfaces:**
- Consumes: Task 1 `Watch`, Task 2 `FsChangeFeed`, Task 3 `Server`.
- `LIVE_JS` behaviour: open `ws://` + `location.host` + `/ws`; on `{panes}` set `innerHTML` of `#<id>` for each (for `psel` the `<select>`'s options), set `.meta`, then `bindDyn(); setLang(current); showP(currentSelection, true)`; on `{reload}` run the existing full `refresh()`; on close → state Polling, start the existing `/version` poll, retry socket with the back-off; state Offline when `/version` also fails. Labels: `Live`/`สด`, `Polling`/`ตรวจเป็นระยะ`, `Offline`/`ขาดการเชื่อมต่อ`.

- [ ] **Step 1: Write failing tests:** `tests/golden.rs::file_page_has_no_live_script` (file page does not contain `new WebSocket`); `crates/adapters-web/tests/web.rs::live_page_has_socket_script` (`render_page(Live)` contains `new WebSocket` and `/ws`).
- [ ] **Step 2: Run** `cargo test --workspace` — Expected: the two new tests FAIL (the old block is still in `JS`; no socket code yet).
- [ ] **Step 3: Implement** the move and `LIVE_JS`; regenerate the golden and check with `git diff --stat` + `git diff tests/golden/report.html` that only the script after `</main>` changed (the removed polling block); wire the root.
- [ ] **Step 4: Run** `cargo test --workspace` — Expected: all pass. `cargo build --release`.
- [ ] **Step 5: Commit** `feat(uno-report): live page uses the socket, polling as fallback (R2)`.

---

### Task 5: End-to-end check, docs, version

**Files:**
- Create: `tools/uno-report/tests/e2e_live.py` (Playwright; run by hand, not by cargo)
- Modify: `tools/uno-report/ARCHITECTURE.md` (ChangeFeed, Watch, panes, socket flow), `CHANGELOG.md` (v0.5.15), `plugins/core/.claude-plugin/plugin.json` (0.5.15)

- [ ] **Step 1: Write `e2e_live.py`** that starts `uno-report --serve --port <free> --data <temp copy of fixtures>` and asserts: no console errors; flag shows Live; append a plan-approved line → text appears in < 1 s (Linux) and the `#p0` pane element is the same DOM node before and after (in-place swap, not reload); 50 quick appends → ≤ 3 pane swaps (count via a MutationObserver); kill server → flag Polling then Offline; restart server → Live again within 35 s; add a new project folder → page lists it (reload path).
- [ ] **Step 2: Run** `python3 tools/uno-report/tests/e2e_live.py` — Expected: before Task 4 is built this fails; after, "e2e: 6/6 passed".
- [ ] **Step 3: Docs:** ARCHITECTURE.md section "Live updates"; CHANGELOG entry with the after-update steps (unload → `cargo install --force` → load); owner's manual EN/TH dashboard section text is handed over in the final message (manual lives in `~/UNO/data`, not in git).
- [ ] **Step 4: Run** `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `python3 tests/test_hooks.py`, `python3 tests/test_data_check.py` — Expected: all green (hook tests 146/148 on Linux is the known /tmp case; 148/148 on the Mac).
- [ ] **Step 5: Commit** `docs(uno-report): live updates; v0.5.15 (R2)`, then `git format-patch`.
