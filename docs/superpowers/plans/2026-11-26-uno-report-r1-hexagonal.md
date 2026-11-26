# uno-report R1 — Hexagonal Workspace Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split `tools/uno-report/src/main.rs` (1,171 lines) into a Ports & Adapters Cargo workspace with byte-identical output to v0.5.12.

**Architecture:** `domain` (types + rules) ← `app` (ports + use cases) ← `adapters-fs` / `adapters-os` / `adapters-web` / `adapters-file` ← root package `uno-report` (CLI + wiring). A golden test captured from today's binary guards every move.

**Tech Stack:** Rust 2021, Cargo workspace, `serde_json`, `pulldown-cmark` 0.12 (html only), `clap` 4 (derive) — new.

**Spec:** `docs/superpowers/specs/2026-11-26-uno-report-r1-hexagonal.md`

## Global Constraints
- Output of `/`, the HTML file, `/version` semantics, CLI flags (`--data --out --manual --open --watch N --serve --port N -h/--help`) and defaults are unchanged from v0.5.12 (tag `v0.5.12`, commit `321aadd`).
- `cargo install --path ~/UNO/uno/tools/uno-report --force` keeps working → the root `Cargo.toml` is both `[workspace]` and `[package] name = "uno-report"` with `src/main.rs`.
- Dependency direction: `domain` uses std only; `app` uses `domain` only; each `adapters-*` uses `domain` + `app` (+ its own external crate); no adapter depends on another adapter; only the root wires them.
- Only new external dependency: `clap = { version = "4", features = ["derive"] }`, in the root package only.
- Server: bind `127.0.0.1` only; Host must equal `localhost:<port>` or `127.0.0.1:<port>` (else 403); non-GET → 405; `/` page, `/version` signature, else 404.
- Fixtures: invented names only (`alpha`, `beta`, …), no 13-digit numbers; `python3 scripts/check_no_company_data.py` must pass.
- Root `.gitignore` ignores `*.log`; fixture `audit.log` files need an explicit `!` exception.
- Test command for every task (run in `tools/uno-report`): `cargo test --workspace`. Lint before the last commit: `cargo clippy --workspace --all-targets -- -D warnings`.
- Execution environment: a clone of `github.com/jetsadawwts/UNO` at `v0.5.12` in Claude's workspace, branch `r1-hexagonal`. Hand-over is `git format-patch v0.5.12..r1-hexagonal`; the owner applies with `git am` and pushes.

## Review Focus
1. Data folder missing or unreadable → same `uno-report: cannot read <dir>: …` message as v0.5.12, no panic. Test in Task 4.
2. A project folder with only `tasks.md`, only `audit.log`, or empty files → listed with zeros, not skipped, not a panic. Test in Task 4 (fixture `beta`).
3. Garbled lines in `audit.log` (not JSON, missing fields, non-UTF-8 bytes) → skipped; `time`-less records counted in `events` but not in `recent`/`per_day`. Test in Task 2 and Task 4.
4. `lsof` absent or failing → `sessions_known = false` and the activity panel shows the "unknown" state, not "0 sessions". Test in Task 3 (fake probe returning `None`) and Task 5.
5. Only one manual language present (or none) → falls back exactly as v0.5.12 (EN→TH copy, TH→EN copy, none → "No manual found at …"). Test in Task 6.

---

### Task 1: Golden harness on the current code

**Files:**
- Modify: `tools/uno-report/src/main.rs` (`now()` ~L980, `open_sessions()` ~L214)
- Create: `tools/uno-report/tests/fixtures/data/{alpha,beta,_old,.hidden,gamma}/…`, `tests/fixtures/data/manual.md`, `tests/fixtures/data/manual.th.md`
- Create: `tools/uno-report/tests/golden.rs`, `tools/uno-report/tests/golden/report.html`
- Modify: `.gitignore` (repo root)

**Interfaces:**
- Produces: env seams `UNO_REPORT_FAKE_NOW` (string used verbatim as "now") and `UNO_REPORT_FAKE_SESSIONS` (`"alpha=1,beta=0"` → map; `"none"` → `None`; `""` → empty map). Later tasks keep these names; Task 7 moves them into the root as `FixedClock` / `FixedSessions`.
- Produces: `tests/golden.rs::golden_file_output_matches` and `golden_help_matches` — later tasks only run them, never edit the golden files.

Fixture contents (decided here):
- `alpha/audit.log`: 12 JSON lines covering every `read_audit` branch once — `plan-approved` (backend `anthropic`), `plan-approved` (backend `ollama`), `smoke`, `blocked` with `rule`, `blocked` with only `zone`, `asked-owner` with a 70-char command (exercises `clip` at 60), `allowed` zone `review`, `allowed` zone `normal`, `called` tool `Agent` name `core:security-reviewer`, `called` tool `Skill` name `core:work`, one line `not json`, one valid line without `time`. Times on two different days in 2026-01.
- `alpha/tasks.md`: table `| ID | Task | Status |` with T1 `**Done**`, T2 `open`, T3 `` `dropped` ``, T4 `merged`, T2 again (duplicate, must be ignored), `T2a` (ignored), a non-table line.
- `beta/tasks.md` only (T1 `open`); `beta` has no `audit.log`.
- `_old/audit.log`, `.hidden/tasks.md` (must be skipped); `gamma/` contains only `notes.txt` (skipped: neither file).
- `manual.md` and `manual.th.md`: one heading, one table, one task list, one strikethrough each.

- [ ] **Step 1: Write the failing test** — `tests/golden.rs`:

```rust
fn run(args: &[&str]) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_uno-report"))
        .args(args)
        .env("UNO_REPORT_FAKE_NOW", "2026-01-15 09:30")
        .env("UNO_REPORT_FAKE_SESSIONS", "alpha=1,beta=0")
        .output().unwrap()
}
fn fixture() -> std::path::PathBuf { std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/data") }

#[test]
fn golden_file_output_matches() {
    let out = std::env::temp_dir().join(format!("uno-golden-{}.html", std::process::id()));
    let data = fixture();
    let o = run(&["--data", data.to_str().unwrap(), "--out", out.to_str().unwrap()]);
    assert!(o.status.success());
    let got = std::fs::read_to_string(&out).unwrap().replace(data.to_str().unwrap(), "<DATA>");
    let want = include_str!("golden/report.html");
    assert_eq!(got, want);
}

#[test]
fn golden_help_matches() {
    let o = run(&["--help"]);
    assert_eq!(String::from_utf8_lossy(&o.stdout), include_str!("golden/help.txt"));
}
```

- [ ] **Step 2: Run** `cargo test --test golden` — Expected: compile error, `golden/report.html` not found.
- [ ] **Step 3: Add the two env seams** at the top of `now()` and `open_sessions()` (return early when the variable is set), add fixtures and the `.gitignore` line `!tools/uno-report/tests/fixtures/**/audit.log`.
- [ ] **Step 4: Capture the golden files** from the binary itself: run it with the same env and args, write the normalised page to `tests/golden/report.html` and `--help` stdout to `tests/golden/help.txt`. Then open `report.html` in Playwright (light + dark, EN + TH) and confirm it shows 2 projects (`alpha`, `beta`), alpha: 4 tasks done/dropped mix, 2 approvals (1 local), 1 Epsilon review, 2 blocks.
- [ ] **Step 5: Run** `cargo test --test golden` — Expected: 2 passed. Then change one byte in `report.html`, run again — Expected: FAIL; revert.
- [ ] **Step 6: Run** `python3 scripts/check_no_company_data.py` from the repo root — Expected: exit 0.
- [ ] **Step 7: Commit** `test(uno-report): golden harness with fake data, fixed clock and sessions (R1)`.

---

### Task 2: Workspace + `domain` crate

**Files:**
- Modify: `tools/uno-report/Cargo.toml` (add `[workspace] members = ["crates/*"]`, `resolver = "2"`, `[workspace.package] edition = "2021"`; root package keeps name/bin)
- Create: `crates/domain/Cargo.toml`, `crates/domain/src/lib.rs`, `src/{project,task,event,agent}.rs`
- Modify: `src/main.rs` — delete the moved items, `use domain::…`

**Interfaces:**
- Produces:
  - `pub struct Task { pub id: String, pub title: String, pub status: String }`, `pub fn is_open(t: &Task) -> bool`, `pub const DONE: &[&str]`, `pub const DROPPED: &[&str]`, `pub const RECENT_KEEP: usize = 40`
  - `pub fn agent_name(raw: &str) -> String` (same 11-entry table)
  - `pub fn clip(s: &str, n: usize) -> String`
  - `#[derive(Default, Clone, Debug, PartialEq)] pub struct AuditRecord { pub time, event, action, backend, task, rule, zone, tool, command, file, name, description: String }` — every field defaults to `""`.
  - `#[derive(Clone, Debug, PartialEq)] pub enum EventKind { PlanApproved { task: String }, Smoke, Blocked { rule: String }, AskedOwner { command: String }, Edited { file: String }, Agent { who: String, description: String }, Skill { name: String } }`
  - `#[derive(Clone, Debug, PartialEq)] pub struct Event { pub time: String, pub kind: EventKind }`
  - `#[derive(Default)] pub struct Project { …all v0.5.12 fields…, pub recent: Vec<Event> }` with `count_status`, `blocked_total`, `reviews`, `open` as today, plus `pub fn apply(&mut self, r: &AuditRecord)` (one record = today's loop body; `clip(…, 60)` happens here) and `pub fn finish(&mut self)` (sort recent newest first, truncate to `RECENT_KEEP`).
- Consumes: nothing.

- [ ] **Step 1: Write failing unit tests** in `crates/domain/src/project.rs`:
  - `apply_plan_approved_local_backend`: record `{event:"plan-approved", backend:"ollama", task:"T3", time:"2026-01-02T10:00"}` → `approvals == 1`, `approvals_local == 1`, `approvals_by_day["2026-01-02"] == 1`, `recent[0].kind == PlanApproved{task:"T3"}`.
  - `apply_blocked_falls_back_rule_zone_tool`: rule `""`, zone `"forbidden"` → `blocked["forbidden"] == 1`.
  - `apply_without_time_counts_event_only`: `events == 1`, `per_day.is_empty()`, `recent.is_empty()`.
  - `apply_called_agent_maps_name`: tool `Agent`, name `core:security-reviewer` → `agents["Epsilon"] == 1`, `reviews() == 1`.
  - `finish_keeps_newest_40`: 45 records → `recent.len() == 40`, `recent[0].time` is the newest.
  - `is_open_excludes_done_merged_dropped_wont`, `agent_name_unknown_returns_id` (`"x:foo"` → `"foo"`), `clip_counts_chars_not_bytes` (Thai text).
- [ ] **Step 2: Run** `cargo test -p domain` — Expected: FAIL (crate items missing).
- [ ] **Step 3: Move the types and rules** from `main.rs` into `domain`; `read_audit` in `main.rs` now builds an `AuditRecord` per JSON line and calls `apply`, then `finish`. Event text (`"Plan T3 approved"` / `"อนุมัติแผน T3"` …) moves to a `fn event_text(k: &EventKind) -> (String, String)` in `main.rs` (moves to `adapters-web` in Task 6).
- [ ] **Step 4: Run** `cargo test --workspace` — Expected: domain tests pass, golden 2/2 pass.
- [ ] **Step 5: Commit** `refactor(uno-report): workspace + domain crate (R1)`.

---

### Task 3: `app` crate — ports and use cases

**Files:**
- Create: `crates/app/{Cargo.toml,src/lib.rs,src/ports.rs,src/dashboard.rs,src/signature.rs}`
- Modify: `src/main.rs` — `render()` calls `app::build_dashboard`, `signature()` calls `app::signature`

**Interfaces:**
- Produces (`app::ports`):
```rust
pub struct ProjectFiles { pub name: String, pub audit: Vec<AuditRecord>, pub tasks: Vec<Task> }
pub trait ProjectStore {
    fn projects(&self) -> Result<Vec<ProjectFiles>, String>; // sorted by name; `_`/`.` and folders with neither file excluded
    fn stamp(&self) -> String;                               // cheap fingerprint of files the page shows (sizes + mtimes)
}
pub struct ManualText { pub en: Option<String>, pub th: Option<String>, pub location: String } // markdown source
pub trait ManualSource { fn manual(&self) -> ManualText; }
pub trait SessionProbe { fn open_sessions(&self) -> Option<std::collections::BTreeMap<String, usize>>; }
pub trait Clock { fn now(&self) -> String; } // "YYYY-MM-DD HH:MM"
```
- Produces (`app`): `pub struct Dashboard { pub projects: Vec<Project>, pub sessions_known: bool, pub generated_at: String }`, `pub fn build_dashboard(store: &impl ProjectStore, sessions: &impl SessionProbe, clock: &impl Clock) -> Result<Dashboard, String>`, `pub fn signature(store: &impl ProjectStore, manual_stamp: &str, sessions: &impl SessionProbe) -> String` (same FNV-1a loop and seed `1469598103934665603`, prime `0x0100_0000_01b3`, `{:016x}`).
- Consumes: Task 2 `Project::apply`, `finish`.

- [ ] **Step 1: Write failing tests** with in-memory fakes (`FakeStore`, `FakeSessions(Option<…>)`, `FakeClock`) in `crates/app/src/dashboard.rs`:
  - `sessions_known_false_when_probe_none` → `sessions_known == false`, every `project.sessions == 0`.
  - `sessions_mapped_by_project_name` → `{"alpha":2}` gives alpha 2, beta 0.
  - `store_error_is_returned` → `Err("cannot read …")` passes through unchanged.
  - `generated_at_comes_from_clock`.
  - `signature_changes_when_stamp_or_sessions_change` and `signature_is_16_hex_chars`.
- [ ] **Step 2: Run** `cargo test -p app` — Expected: FAIL (missing items).
- [ ] **Step 3: Implement** the use cases; in `main.rs` add temporary private adapters wrapping today's functions so `render()`/`signature()` go through `app`.
- [ ] **Step 4: Run** `cargo test --workspace` — Expected: all pass, golden 2/2.
- [ ] **Step 5: Commit** `refactor(uno-report): app crate with ports and use cases (R1)`.

---

### Task 4: `adapters-fs`

**Files:**
- Create: `crates/adapters-fs/{Cargo.toml,src/lib.rs,src/audit.rs,src/tasks.rs}` (deps: `domain`, `app`, `serde_json`)
- Modify: `src/main.rs` — remove `read_audit`, `read_tasks`, `cells`, the dir walk, the stamp code

**Interfaces:**
- Produces: `pub struct FsStore { pub root: PathBuf }` impl `ProjectStore`; `pub struct FsManual { pub en_path: PathBuf }` impl `ManualSource` (TH = `en_path.with_file_name("manual.th.md")`, `location` = `en_path.display()`); `pub fn manual_stamp(en_path: &Path) -> String`; pure parsers `pub fn parse_audit(text: &str) -> Vec<AuditRecord>` and `pub fn parse_tasks(text: &str) -> Vec<Task>`.
- Error text for an unreadable root: exactly `format!("cannot read {}: {e}", root.display())`.

- [ ] **Step 1: Write failing tests:**
  - `parse_audit_skips_garbage_and_non_utf8`: input with `not json`, `{}` and a line built from `String::from_utf8_lossy(&[0xff, 0xfe])` → only valid JSON objects returned; `{}` returns a record with all fields `""`.
  - `parse_tasks_normalises_status_and_dedupes`: `**Done**` → `done`, `` `dropped` `` → `dropped`, `In progress` → `in`, duplicate T2 kept once, `T2a` and `X1` ignored, title from the `Task` column.
  - `store_lists_fixture_projects`: `FsStore{root: fixtures/data}` → names `["alpha","beta"]`; beta has empty `audit` and 1 task.
  - `store_missing_root_errors`: root `/nonexistent-uno` → `Err` starting `cannot read /nonexistent-uno:`.
  - `manual_falls_back_when_th_missing`: temp dir with `manual.md` only → `en.is_some()`, `th.is_none()`.
- [ ] **Step 2: Run** `cargo test -p adapters-fs` — Expected: FAIL.
- [ ] **Step 3: Move** the parsing and file walking from `main.rs`; the JSON → `AuditRecord` mapping reads each field with `as_str().unwrap_or("")`.
- [ ] **Step 4: Run** `cargo test --workspace` — Expected: all pass, golden 2/2.
- [ ] **Step 5: Commit** `refactor(uno-report): adapters-fs (R1)`.

---

### Task 5: `adapters-os`

**Files:**
- Create: `crates/adapters-os/{Cargo.toml,src/lib.rs}` (deps: `app`)
- Modify: `src/main.rs` — remove `open_sessions()` body and `now()` body (env seams stay in `main.rs` until Task 7)

**Interfaces:**
- Produces: `pub struct LsofSessions;` impl `SessionProbe`; `pub struct SystemClock;` impl `Clock` (runs `date +%Y-%m-%d %H:%M`, empty string on failure); pure `pub fn count_sessions(lsof_out: &str, ps_out: &str) -> BTreeMap<String, usize>`.

- [ ] **Step 1: Write failing tests** for `count_sessions`:
  - two `claude` pids in `/Users/x/UNO/projects/alpha/src` and `/Users/x/UNO/projects/beta` → `{alpha:1, beta:1}`.
  - a pid whose parent's comm is `claude` (child of the same session) → not counted.
  - cwd outside `/UNO/projects/` → ignored; comm given as full path `/opt/bin/claude` → recognised as claude.
  - empty `ps` output → every lsof pid counted (same as v0.5.12).
- [ ] **Step 2: Run** `cargo test -p adapters-os` — Expected: FAIL.
- [ ] **Step 3: Move** the parsing into `count_sessions`; `LsofSessions::open_sessions` runs the two commands exactly as v0.5.12 and returns `None` when `lsof` cannot run.
- [ ] **Step 4: Run** `cargo test --workspace` — Expected: all pass, golden 2/2.
- [ ] **Step 5: Commit** `refactor(uno-report): adapters-os (R1)`.

---

### Task 6: `adapters-web` (views + server) and `adapters-file`

**Files:**
- Create: `crates/adapters-web/{Cargo.toml,src/lib.rs,src/html.rs,src/charts.rs,src/views.rs,src/manual.rs,src/assets.rs,src/server.rs}` (deps: `domain`, `app`, `pulldown-cmark`)
- Create: `crates/adapters-file/{Cargo.toml,src/lib.rs}` (deps: none besides std)
- Move: `assets/*.svg` stay where they are (paths in `include_str!` updated)
- Modify: `src/main.rs` — only CLI and wiring remain

**Interfaces:**
- Produces (`adapters_web`):
  - `pub enum PageMode { File { watch: Option<u64> }, Live }`
  - `pub fn render_page(d: &Dashboard, m: &ManualText, data_label: &str, mode: &PageMode) -> String` — byte-identical to v0.5.12 `render()`; `event_text` lives here.
  - `pub struct Server<S, M, P, C> { pub port: u16, pub store: S, pub manual: M, pub sessions: P, pub clock: C, pub data_label: String, pub manual_path: PathBuf }` with `pub fn run(self, open: bool) -> !` and a testable `pub fn handle(&self, request_head: &str) -> (u16, &'static str, Vec<u8>)` (status, content type, body) — Host/Method/route rules from Global Constraints.
  - CSS, HEAD_JS, JS move verbatim into `assets.rs`.
- Produces (`adapters_file`): `pub fn write_atomic(out: &Path, page: &str) -> Result<(), String>` (write `<out>.html.tmp` → rename; same error strings as v0.5.12).

- [ ] **Step 1: Write failing tests:**
  - `manual_only_en_copies_to_th`, `manual_only_th_copies_to_en`, `manual_none_shows_location_message` (asserts the EN and TH sentences of v0.5.12 with the location).
  - `handle_rejects_foreign_host` → `"GET / HTTP/1.1\r\nHost: evil.example:8765\r\n\r\n"` → 403.
  - `handle_rejects_post` (good Host) → 405; `handle_unknown_path` → 404; `handle_version_is_16_hex` → 200, body 16 hex chars; `handle_root_is_live_page` → 200, body contains `window.UNO_LIVE=true;`.
  - `sessions_unknown_panel` (Review Focus 4): `render_page` with `sessions_known == false` contains the same "unknown" markup v0.5.12 emits (copy the expected substring from `golden/report.html` generated with `UNO_REPORT_FAKE_SESSIONS=none` during this step).
  - `write_atomic_replaces_and_leaves_no_tmp`.
- [ ] **Step 2: Run** `cargo test -p adapters-web -p adapters-file` — Expected: FAIL.
- [ ] **Step 3: Move** the view, chart, manual, asset, server and build code; the server's per-connection thread calls `handle` then writes the response with today's headers (`Cache-Control: no-store`, `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, `Connection: close`).
- [ ] **Step 4: Run** `cargo test --workspace` — Expected: all pass, golden 2/2.
- [ ] **Step 5: Commit** `refactor(uno-report): adapters-web and adapters-file (R1)`.

---

### Task 7: Root composition with `clap`, docs, version

**Files:**
- Modify: `tools/uno-report/Cargo.toml` (add `clap`), `src/main.rs` (≤ ~120 lines: `Cli` struct, `FixedClock`, `FixedSessions`, wiring)
- Create: `tools/uno-report/ARCHITECTURE.md` (the layer diagram, one paragraph per crate, the dependency rule, how to add an adapter)
- Modify: `CHANGELOG.md` (v0.5.13), `plugins/core/.claude-plugin/plugin.json` (0.5.13)

**Interfaces:**
- Consumes: every adapter from Tasks 4–6, `app::build_dashboard`, `app::signature`.
- Produces: `uno-report` binary with the v0.5.12 flags; `--help` prints exactly `tests/golden/help.txt` (use `clap`'s `override_help`/`help_template` with that text — the golden test is the contract); `--watch 0` and non-numeric `--watch` behave as "no watch" like v0.5.12; unknown flags are ignored as in v0.5.12 (`clap` `allow_external_subcommands`/`ignore_errors` — whichever keeps that behaviour; ledger the choice).

- [ ] **Step 1: Write failing tests** in `tests/cli.rs`: `watch_zero_is_no_watch` (process exits after one build with `--watch 0`), `unknown_flag_is_ignored` (`--bogus` still writes the file, exit 0), `port_flag_parses` (`--serve --port 0` is rejected the same way v0.5.12 rejects an unbindable port: message `cannot listen on 127.0.0.1:` and exit 1 — use a port already bound by the test).
- [ ] **Step 2: Run** `cargo test --test cli` — Expected: FAIL (new `Cli` not wired yet / behaviour differs).
- [ ] **Step 3: Implement** the root; `FixedClock` / `FixedSessions` are chosen when `UNO_REPORT_FAKE_NOW` / `UNO_REPORT_FAKE_SESSIONS` are set.
- [ ] **Step 4: Run** `cargo test --workspace` → all pass; `cargo clippy --workspace --all-targets -- -D warnings` → no warnings (fix moved code without changing output; golden proves it); `cargo build --release` → ok; check crate boundaries with `cargo tree -p domain` (only `domain`) and `cargo tree -p app` (only `app`, `domain`).
- [ ] **Step 5: Manual check:** `cargo install --path . --force` in a temp `CARGO_HOME`, run `--serve` on port 8799 against the fixtures, open in Playwright: page loads, no console errors, appending a line to `alpha/audit.log` shows within the live interval.
- [ ] **Step 6: Commit** `refactor(uno-report): clap CLI and composition root; docs; v0.5.13 (R1)`, then `git format-patch v0.5.12..HEAD -o /mnt/user-data/outputs/r1-patches`.
