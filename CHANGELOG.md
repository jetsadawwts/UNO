# UNO changelog

Newest first. Each entry: what changed, why, and what to do after updating.

## v0.5.23 — 2026-11-28
- **Mu has a desk in the Office.** 12 workstations: Mu's (`desk12.webp`, `chair12.webp`) is the fourth desk of the front row, built on the same desk body as the others (Kappa's) with Mu's own items (rubber duck, orange mug, notebook, pen cup) so size and colour match; `people.webp` gets a 13th row (Mu, orange flannel shirt). Art made with an AI image tool from prompts written for UNO, like the rest. Mu now sits, walks and gets hand-off lines like every member instead of showing only on cards.
- **Rows evenly spaced.** The front row (Theta, Iota, Kappa, Mu) moved 30 px left and 6 px up, so the three rows step back by the same distance; seats and walking routes follow the desks (`layout.json`).
- **A relaxed office: members sit only when called.** A member walks to their desk when Uno hands them work and sits there while working (a heartbeat line from the last 30 minutes); the rest of the time they take a break around the office and keep walking between spots (3–8 s stops instead of 14–32 s, so it never looks frozen). Uno stays in his glass office while any Claude Code session with UNO is open (the dashboard's open-session rule); offline he takes a break too. The summary card shows "Free" (taking a break) or "Offline" instead of "Away". New `ofc-data` entry: the number of open sessions. Where people go on a break is decoration and the page says so.
- **Uno walks out to hand off work.** For each hand-off Uno gets up, leaves the glass office through its door, walks to the member's desk (by a new aisle along the right ends of the rows), the hand-off line plays there and the member shows "new task" without leaving the desk, then Uno walks back. Several hand-offs play one after another; the timeline on the side stays real time.
- **After updating:** only the dashboard changed (plus the version number in the plugin): stop `uno-report --serve`, `cargo install --path ~/UNO/uno/tools/uno-report --force --locked`, start it again. Updating the plugin is optional.

## v0.5.22 — 2026-11-28
- **New Office tab on the dashboard** (Dashboard · Office · Manual): UNO's team in a pixel-art office floor (open office, Uno's glass office, server room, restroom, kitchen and dining table, lounge with arcade and foosball).
  - **Summary cards:** working, needs you (waiting for you or stopped by an error), away. **Team list** with role and status; pick a member to see what they do now and where they are. **Needs you** list. **Hand-offs from Uno**: the latest agent calls in `audit.log` (Uno → member, description). **Stage bar** for the task in a live line, from its status in `tasks.md` (proposed → ready → in progress → in review → done); hidden when the task is not in `tasks.md`.
  - **Scene:** each member whose newest heartbeat (any project) is at most 30 minutes old sits at their own workstation with an icon for what they do (command, edit, read, hand-off, working, waiting for you, error, turn finished) and the task id. Others are away and walk to lounge spots; where they go is decoration, not data, and the page says so. A new hand-off draws a line from Uno and the member walks to Uno's door, then back. "Uno's room" zooms into the glass office; "Show names" labels everyone. Follows language and the light / dark / auto theme; reduced motion moves people without walking.
  - **Art:** `tools/uno-report/assets/office/` (floor, 11 workstations split into chair and desk, Uno's desk front, a 12-row character sheet, `layout.json`), ~500 KB of WebP embedded in the binary and page; nothing is loaded from outside. Generated with an AI image tool from prompts written for UNO; see the folder's README. The golden test replaces the art with `<OFFICE-ART>`. New live pane `ofc-data`.
- **One heartbeat per team member.** `trace_heartbeat.py` now writes a subagent's state to `.uno/heartbeat/<session>.<agent_id>.json` (main thread, Uno, stays in `<session>.json`), so several members show as working at once and each one leaves the desk as soon as it stops. New hook events SubagentStart (creates the file) and SubagentStop (removes it); the main thread's PostToolUse of a waited-for Agent call also removes it (not a background launch), and SessionEnd removes them all. Uno calling the Agent tool now shows as Uno handing off. Still names only: no prompts, answers, commands or paths (tested). Without `agent_id` in hook input (older Claude Code) subagent work falls back to the old single file.
  - **Hand-off line:** opening the page replays hand-offs from the last 10 minutes (up to 3), the line stays ~8 s, and the member card shows the last task from Uno.
- **Hand-off timeline.** Each hand-off from Uno is one card: started, how many reads, commands, edits and other tools, then done (with time and minutes), working, or outcome unknown (session ended or quiet for 30 minutes). The steps come from `.uno/office.log`, new: `trace_heartbeat.py` appends one line per subagent start, tool, stop and returned answer (time, session id, agent id, agent type, step, tool name; no prompts, answers, commands or paths; newest 1,000 lines kept past 1 MB; tested). A call is matched to the member's start only when it is the one open call of that member in that session within 2 minutes; otherwise, and for calls made before this version, the card shows the headline only ("no work record").
- **Needs you** now also lists Uno's finished turns from the last 30 minutes ("finished a turn, waiting for your reply"; it cannot tell a plan waiting for approval from a plain answer, since content is not stored) and tasks only a person can move on, from `tasks.md`: escalated (needs your decision), in-review (review or merge), proposed (approval). Order: errors, waiting for you, finished turns, escalated, in review, proposed. The summary card counts them all.
- **Uno plans and delegates; new member Mu writes the code.** The log showed only Uno because no member wrote application code: Uno wrote small changes itself and Superpowers' unnamed `general-purpose` subagents wrote the rest.
  - **Mu (`developer`)**, new agent `plugins/core/agents/developer.md`: application code for one approved plan step at a time, makes Delta's tests pass, runs the Stack checks, never edits Delta's tests or `.uno/`, never commits. Uno commits each step. Roster in `skills/work/references/team.md` (13 members: Uno, Alpha to Mu; reserved names now start at Nu).
  - **`guard_zones` holds Uno to it:** in an onboarded project the main thread may edit only `.uno/`, `CLAUDE.md`, `.claude/settings.local.json`, `docs/intent.md`, `docs/spec.md` and `docs/refactor/`; anything else is blocked (rule `lead-delegates`) with a message naming who should do it. Members are told apart by the `agent_id` Claude Code sends for subagent calls. `UNO_LEAD_EDITS=allow` in the environment turns the rule off.
  - **Reviews by members:** Delta (`test-writer`) and Lambda (`architect`) get a read-only `review` mode. With Superpowers (7.0.2) `subagent-driven-development` keeps its flow, with UNO members in its dispatches: implementer → Mu, task reviewer (spec + quality) → Delta, final whole-branch review → Lambda. Without Superpowers the built-in loop is the same: Mu acts, Delta reviews each step, Lambda the branch. 3 failed review rounds → escalated. `/core:refactor` and `/core:new-project` (scaffold) delegate code the same way.
  - **Logs name who did it:** every guard (`guard_zones`, `guard_git`, `guard_plan`, `guard_secrets`, `guard_browser`, `guard_infra`, `guard_danger`) now writes `name` on its `audit.log` line: the agent type for a member, `uno` for the main thread (names only). The dashboard feed shows "Mu: edited src/a.ts" or "Uno: blocked: …"; older lines carry no name and show no one.
  - **Office:** Mu has no workstation or sprite yet in this version; it appears on cards and in hand-offs only (desk added in v0.5.23).
- **Limits:** first step of `docs/superpowers/specs/2026-11-27-office-view-design.md` (still a draft). SubagentStart/SubagentStop were not checked in Cowork; if they do not arrive, a member still shows while it uses tools and leaves when the Agent call returns (background agents stay until they go quiet for 30 minutes). No replies from members are recorded, so there is no conversation view. Walks can pass close behind chairs. The lead rule needs `agent_id` in hook input: on an older Claude Code without it, Mu's edits would be blocked too (set `UNO_LEAD_EDITS=allow` until updating). Small tasks now take more time and tokens: Mu, Delta and Lambda each run. A member in review mode is read-only by instruction, not by hook.
- **After updating:** in Claude Code `/plugin marketplace update uno`, update `core@uno`, and restart open Claude Code sessions (new hook events); then stop `uno-report --serve`, `cargo install --path ~/UNO/uno/tools/uno-report --force --locked`, and start it again.

## v0.5.21 — 2026-11-27
- **The dashboard shows versions.** It was not possible to tell which update was running.
  - The header shows `dashboard v0.5.21`, the version of the running `uno-report`; `uno-report --version` (`-V`) prints it.
  - Each live line on the team activity card ends with `hook v0.5.21`: the plugin version of the hook that session is running (`trace_heartbeat.py` reads it from its own `plugin.json` and adds `version` to the heartbeat). When it differs from the dashboard's it is highlighted with "differs from dashboard".
  - `uno-report` now has the same version as the plugin (was always 0.1.0). To release, change both `plugins/core/.claude-plugin/plugin.json` and `[workspace.package]` in `tools/uno-report/Cargo.toml`, then run `cargo update -w --offline` there for `Cargo.lock`; `tests/version.rs` fails when the two numbers differ.
- **Limits:** the hook version appears only on live lines. Sessions on v0.5.20 hooks show a line without it; older hooks send no live line. It is not read from the installed-plugin list (`claude plugin list` shows that).
- **After updating:** in Claude Code `/plugin marketplace update uno`, update `core@uno`, and restart open Claude Code sessions; then stop `uno-report --serve`, `cargo install --path ~/UNO/uno/tools/uno-report --force --locked`, and start it again.

## v0.5.20 — 2026-11-27
- **The team activity card shows what each session is doing now, with the task id.** v0.5.19 showed the last tool, at most once a minute. The card now has one line per open session, for example `T38 · Uno: running Bash · 12 s`, with the states running a tool, working (between tools), waiting for you, turn finished and stopped by an error; the duration counts up in the browser.
  - `trace_heartbeat.py` now runs on UserPromptSubmit, PreToolUse, PostToolUse, PostToolUseFailure, PermissionRequest, Notification, Stop, StopFailure and SessionEnd, and writes on every state change (no throttle). It prints nothing and exits 0, so it never decides a permission or blocks a tool.
  - The task id comes from the branch name (same rule as the plan guard, e.g. `fix/T38-x`); the hook runs `git rev-parse --abbrev-ref HEAD` on UserPromptSubmit and PreToolUse. Only the id is stored — never the branch name, commands, paths, prompts or messages.
  - SessionEnd removes the session's heartbeat file. Counts, the feed and the 30-minute open-session rule are unchanged.
- **Limits:** Claude Code documents no event for a user interrupt, and a blocked or denied tool may send no closing event, so a line can keep its last state until the next event (it is hidden after 30 minutes). PermissionRequest may fire without a dialog, briefly showing "waiting for you". Parallel tools and subagents in one session share one line (the last event wins). A branch with no task id shows none; a finished task's branch keeps showing its id. Each tool call now starts the hook twice.
- **Checked in Cowork** with a temporary probe: UserPromptSubmit, PreToolUse, PostToolUse, PostToolUseFailure, PermissionRequest, Notification (permission_prompt) and Stop arrived. Not verified there: user interrupt, Notification idle_prompt, StopFailure, SessionEnd.
- **After updating:** in Claude Code `/plugin marketplace update uno`, update `core@uno`, and restart open Claude Code sessions; then stop `uno-report --serve`, `cargo install --path ~/UNO/uno/tools/uno-report --force --locked`, and start it again.

## v0.5.19 — 2026-11-27
- **The team activity card now follows the work.** It took its time and Working state from the newest `audit.log` line of any kind but its text from the newest listed event, so a short session that only opened and closed showed "Working", "12 min ago" and an hour-old agent task. Reading, running commands and editing `.uno/` files (plans, specs, tasks) logged nothing, so a busy session also looked closed after 30 minutes.
  - Card text, time and Working state now come from the same activity; session start and end are not activity, and a project with no open session never shows Working.
  - New hook `trace_heartbeat.py` (PreToolUse, every tool): at most once a minute per session it overwrites `.uno/heartbeat/<session>.json` with time, session id, tool name and agent id — no commands, paths or content, nothing added to `audit.log`; files older than a day are removed. `uno-report` uses it to count open sessions and to show lines such as `Beta: used Grep`. Heartbeats change no dashboard count.
  - Limit: a session that calls no tool for 30 minutes still counts as closed, and the card can lag the latest tool by up to a minute.
- **After updating:** in Claude Code `/plugin marketplace update uno`, update `core@uno`, and restart open Claude Code sessions (the new hook runs in sessions started afterwards); then stop `uno-report --serve`, `cargo install --path ~/UNO/uno/tools/uno-report --force --locked`, and start it again.

## v0.5.18 — 2026-11-27
- **Hooks now work when the plugin folder's path has a space.** Every command in `hooks.json` used `python3 ${CLAUDE_PLUGIN_ROOT}/hooks/<hook>.py` without quotes. Claude Code in the Terminal installs plugins under `~/.claude/plugins/…` (no space), but the Claude app's local agent mode loads the plugin under `~/Library/Application Support/Claude/…`: the shell split the path, `python3` exited 2, and every hook — all eight guards included — failed there (Claude Code treats exit 2 as a block). The path is now quoted (`python3 "${CLAUDE_PLUGIN_ROOT}/hooks/<hook>.py"`), and so is the scanner path in `/core:portfolio-scan`. New `tests/test_hooks_json.py` runs every `hooks.json` command through a shell with the plugin under a folder named `Application Support` (12 cases); it runs in pre-commit.
- **After updating:** `/plugin marketplace update uno` and update `core@uno` in Claude Code; in the Claude app, reload or re-add the UNO plugin so it picks up 0.5.18. Until then do not rely on UNO's guards inside the Claude app.

## v0.5.17 — 2026-11-27
- **uno-report no longer looks at other processes, and launchd no longer restarts it.** CleanMyMac flagged the v0.5.15 binary as an infostealer: an unsigned-looking binary that starts at login, restarts itself when stopped, lists every process with `lsof`/`ps` and watches files in the home folder matches that pattern. An audit of the source and all 90 dependencies found no network client, no access to Keychain, browsers or credentials, and no commands other than `date` and `open` besides the removed `lsof`/`ps`. Changes:
  - Open sessions are now counted from UNO's own hook events: new hook `trace_session.py` (SessionStart / SessionEnd) writes `session-start` / `session-end` to `audit.log`; a session with no event for 30 minutes counts as closed. `lsof`/`ps` are gone.
  - `launchd/com.uno.report.plist` keeps `RunAtLoad` but drops `KeepAlive`; starting at login stays optional, and the manual now suggests running `uno-report --serve` in a Terminal tab.
  - The socket limit no longer uses `fetch_update` (deprecated in newer Rust).
  - Hook tests: 155 (new `trace_session` cases).
- **After updating:** in Claude Code `/plugin marketplace update uno` and update `core@uno` (the new hook runs in sessions started afterwards), then `cargo install --path ~/UNO/uno/tools/uno-report --force --locked`, and scan `~/.cargo/bin/uno-report` with CleanMyMac before starting it.

## v0.5.16 — 2026-11-27
- **`uno-report` needs Rust 1.91 or newer, and installs with `--locked`.** Since v0.5.13–0.5.15 its dependencies need a newer Rust than some machines have (`clap` 4.6 uses edition 2024, `notify-types` 2.3 needs 1.91); on Rust 1.84 `cargo install` stopped with ``feature `edition2024` is required``. `tools/uno-report` now declares `rust-version = "1.91"` (on Rust 1.85–1.90 cargo now says which version is needed), and every install command uses `--locked` so the dependency versions that passed the tests are the ones built.
- **After updating:** `rustc --version` (must be 1.91+; if not: `rustup update stable`, or `brew upgrade rust` for a Homebrew Rust) → `launchctl unload ~/Library/LaunchAgents/com.uno.report.plist` → `cargo install --path ~/UNO/uno/tools/uno-report --force --locked` → `launchctl load ~/Library/LaunchAgents/com.uno.report.plist`.

## v0.5.15 — 2026-11-27
- **Live dashboard updates in under a second, only the part that changed (R2).** `uno-report --serve` now watches `~/UNO/data` (FSEvents) and re-checks open sessions every 5 s; changes go to the page over a WebSocket (`/ws`) as the changed panes only (overview, each project, project picker, manual), swapped in place without losing the selected project or language. If the socket drops the page polls `/version` as before and reconnects by itself; the header shows Live / Polling / Disconnected. Server: tokio + axum, still 127.0.0.1 only, Host checked on every route, foreign `Origin` refused on `/ws`, at most 8 live pages, GET only. File mode (`--open`, `--watch`) is unchanged except that its page no longer carries the polling script. Tests: 86 Rust tests plus `tools/uno-report/tests/e2e_live.py` (7 browser checks).
- **After updating:** `launchctl unload ~/Library/LaunchAgents/com.uno.report.plist` → `cargo install --path ~/UNO/uno/tools/uno-report --force` → `launchctl load ~/Library/LaunchAgents/com.uno.report.plist`. Same flags, same plist.

## v0.5.14 — 2026-11-26
- **Data check: `Cargo.lock` checksums no longer look like ID numbers.** A sha256 `checksum = "<64 hex>"` line in a `Cargo.lock` can hold 13 digits in a row, which the 13-digit ID rule flagged, forcing `--no-verify` on every dependency change. Only those exact lines in files named `Cargo.lock` skip the ID and secret-string rules; company/project terms and e-mail addresses are still checked on them, and every other line and file is checked as before. New `tests/test_data_check.py` (6 cases) runs in pre-commit after the hook tests.

## v0.5.13 — 2026-11-26
- **`uno-report` is now a Ports & Adapters workspace (R1).** `tools/uno-report` splits into `domain` (counting rules), `app` (ports and use cases), `adapters-fs`, `adapters-os`, `adapters-web`, `adapters-file`, and the root CLI; Cargo enforces that the core does no I/O and adapters do not depend on each other (`tools/uno-report/ARCHITECTURE.md`). Command line parsing uses `clap`. **No visible change:** a golden test proves the page and `--help` are byte-identical to v0.5.12; 60 tests in total. Also fixes a crash when an audit record has no full date (`time` missing or shorter than a day). Prepares R2 (real-time updates) and R3 (settings, config, logging).
- **After updating:** `launchctl unload ~/Library/LaunchAgents/com.uno.report.plist` → `cargo install --path ~/UNO/uno/tools/uno-report --force` → `launchctl load ~/Library/LaunchAgents/com.uno.report.plist`. Same flags, same plist.

## v0.5.12 — 2026-11-26
- **`uno-report` live dashboard:** `--serve [--port N]` runs it on http://localhost:8765 (127.0.0.1 only, Host header checked, GET only) and updates the page in place within seconds when `~/UNO/data` or open sessions change; `--watch N` regenerates the file every N minutes. `tools/uno-report/launchd/com.uno.report.plist` starts `--serve` at login (log: `/tmp/uno-report.log`).
- **Dashboard:** settings menu (language, theme and day/night hours, page reload, team-activity window, start page, live interval; stored in the browser), team activity per project (open Claude Code sessions, latest audit event), six overview charts, per-project views, EN/TH manual (`manual.md`, `manual.th.md`).
- **After updating:** `launchctl unload ~/Library/LaunchAgents/com.uno.report.plist` (if loaded) → `cargo install --path ~/UNO/uno/tools/uno-report --force` → copy the plist to `~/Library/LaunchAgents/` and `launchctl load` it.

## v0.5.11 — 2026-11-26
- **`uno-report` (Rust, `tools/uno-report`):** builds one local HTML file with a Dashboard and the manual. Reads every project in `~/UNO/data` (`audit.log`, `tasks.md` Status column) and `~/UNO/data/manual.md`; writes `~/UNO/data/uno-report.html`. Dashboard: tasks done/open/dropped, plan approvals (and how many by a local model), Epsilon reviews, guard blocks by rule, owner prompts, agents used, activity per day, open tasks. No network, no external scripts; light and dark. Install: `cargo install --path ~/UNO/uno/tools/uno-report`, run: `uno-report --open`.

## v0.5.10 — 2026-11-26
- **New guard `guard_danger` (Bash): deleting is fine when it can be undone.** Inside the project, destructive commands run without a prompt after the hook keeps a way back in `.uno/recovery.md`: files about to be deleted, discarded or cleaned are copied to `.uno/trash/<time>/`; branches, stashes, HEAD and remote refs about to move are recorded by sha with the restore command. Build caches, node_modules, /tmp and .uno are not copied. The owner is asked only for commands that reach outside the project (files outside it — copied to the trash first when they fit —, sudo, brew or global package removal, pip uninstall, OS settings and services, chmod/chown -R outside), data no snapshot holds (SQL DROP/TRUNCATE/DELETE, migration reset), shred, unpublishing packages, `git clean -x`, and anything too big to snapshot (200 MB). Trash folders older than 30 days are removed on the next snapshot; `recovery.md` keeps the history.
- **New guard `guard_secrets` (Bash):** blocks shell commands that read a file the `Read(...)` deny rules protect (`cat`, `grep`, `head`, `python -c`, `git show HEAD:path`, globs like `public/*.ini`). Read deny rules alone only stop the Read tool. Patterns come from `.claude/settings*.json` plus a built-in list (.env, *.ini, *.pem, *.key, kubeconfig, tfstate, tfvars, ~/.aws, ~/.kube, ~/.ssh). Git exclusions (`':!**/config.ini'`) and example files (`.env.example`) pass. Not caught: a recursive search with no file named (`grep -r KEY .`). With this guard, auto mode is safe to leave on.
- Fewer permission prompts: `/core:work` runs one plain command per Bash call (no chains, variable assignments, pipes into sed/sort, or redirects to scratch files), since any of these makes Claude Code ask even when each part is allowed.
- Settings templates allow `npx tsc --noEmit`, `npx eslint` and `echo`.
- Hook tests: 148 cases.

## v0.5.9 — 2026-11-24
- Settings templates drop `Write(...)` rules: Claude Code 2.1.3xx applies `Edit(...)` rules to Write too and warns that Write rules never match. Remove them from each project's `.claude/settings.local.json`.

## v0.5.8 — 2026-11-24
- **One approval per task:** settings templates set `defaultMode: acceptEdits`, so Claude Code no longer asks before each file edit. The owner's `approve T<n>` on the plan is the confirmation; `guard_plan`/`guard_zones` still block edits without an approved plan or in forbidden zones, and the deny list still applies. Review every change in the draft PR.
- After updating: add `"defaultMode": "acceptEdits"` under `permissions` in each project's `.claude/settings.local.json`.

## v0.5.7 — 2026-11-24
- **Language-agnostic dependency rule at commit time:** `guard_git` blocks a commit that changes any dependency manifest or lock file (package.json, Cargo.toml, pyproject.toml, requirements*.txt, composer.json, go.mod, pom.xml, build.gradle(.kts), libs.versions.toml, Gemfile, pubspec.yaml, Podfile, Package.swift, mix.exs, *.csproj, vcpkg.json, CMakeLists.txt, build.sbt and more) while the approved plan's Dependencies says none — whatever tool made the change.
- Install commands also gated for Flutter/Dart, Deno, Expo, Elixir, Swift; settings allow build/test for Maven, Gradle, Flutter, Dart, Swift, Ruby, Elixir, Deno.
- **Works for any language, not only Node.** New profile table `## Stack` (one row per part: language, install-from-lock, build, test, lint, run). `/core:work` verify and Delta use each part's own commands.
- **Dependency gate covers every ecosystem:** npm/pnpm/yarn/bun, pip/uv/poetry/pipenv/conda, cargo, composer, go, gem/bundle, dotnet. Installs from a lock or requirements file (`npm ci`, `composer install`, `pip install -r`, `uv sync`, `cargo build`) stay allowed; adding a package must be in the approved plan. Python `name==1.2`, composer `vendor/pkg:^1`, cargo/go `name@version` are matched by name. (`cargo add` used to bypass the gate.)
- Settings templates allow read/test/build commands for Rust, Python, PHP, Go, .NET and Next.js.
- `/core:refactor`: "Moving to another language or framework" — ADR with reason and options, boundary tests that run on old and new, strangler one piece per task, cut-over release with rollback to the last old build.
- **Team of 12.** Iota (`devops-engineer`): Docker, compose, Kubernetes, Helm, Terraform, CI/CD — validates locally, hands the owner exact commands for shared systems. Kappa (`data-engineer`): schema, reversible migrations (up/down/up on a local DB), queries, backfills; sensitive columns named. Lambda (`architect`): ADRs with options, trade-offs, rollout and task breakdown for changes bigger than one task; called by intake, refactor and work. Reserved names: Mu to Psi; Pi, Sigma, Omega last.
- **Infrastructure guard (`guard_infra`):** agents may build, lint, render and dry-run Docker, Kubernetes, Helm, Terraform and Kafka, and change only local targets. Blocked: kubectl/helm writes to any context not listed under the profile's `## Infra` → `Local clusters`; `terraform`/`tofu`/`pulumi` apply, destroy, import; `docker push`/`login`; cloud CLIs that write (aws, gcloud, az…, S3/GCS writes); Kafka topic/config/ACL/offset changes; deleting Docker volumes. Settings deny reading kubeconfig, tfstate, tfvars, `~/.aws`, `~/.docker/config.json`.
- Zones template puts Dockerfiles, compose files, k8s/helm/charts, Terraform, CI pipelines and Kafka config in `review`; kubeconfig, `*secret*.yaml`, tfstate and tfvars in `forbidden`. Helm `Chart.yaml`/`Chart.lock` and `.terraform.lock.hcl` count as dependency files.
- Profile gets an `## Infra` section (container build, local clusters, local services, shared systems never touched).
- Hook tests: 106 cases.
- After updating: add a `## Stack` table to each project's profile; add the new allow rules to each `.claude/settings.local.json` if you want fewer prompts.

## v0.5.6 — 2026-11-24
- **UNO can look at the running app — only on fake data.** New profile section `## Preview` (run command, URL, allowed hosts, `Data: none | fake`, who confirmed). `/core:work` step 3b drives the built-in browser for UI checks, BEFORE/AFTER screenshots (`.uno/designs/<task>/`) and user-guide images; without `Data: fake` it skips and says so.
- **`guard_browser` hook:** any browser tool (built-in browser, Claude in Chrome, Playwright MCP) may open only localhost and the profile's allowed hosts, and only when `Data: fake`. Protects against a remembered login being used on production when test and real accounts are the same.
- **On-screen check by kind of change:** any change to app code gets at least a smoke check (app starts, main pages load, no console errors); logic, UI and export changes also run the acceptance steps on screen (UI adds screenshots, export opens the file). No fake-data preview → skipped, said in the PR, and `review: careful` (condition 10). Profile gets `- Main pages:`; the plan's Verification names the level.
- The owner signs in by hand; UNO never types or stores passwords.
- Hook tests: 65 cases.
- After updating: add a `## Preview` section to each project's profile; keep `Data: none` until you confirm the data is fake.

## v0.5.5 — 2026-11-24
- **Theta (`doc-writer`) joins the team.** Writes and updates README, user guides, deploy/operations guides and developer docs inside an approved plan; edits documentation files only; every command, path and setting it documents must be confirmed in the repo (anything unconfirmed is left out and reported); fake data only in examples.
- `/core:work`: documentation tasks go to Theta; a code task that changes what users or operators do ends with a plan step for Theta to update the affected docs.
- `/core:release`: proposes a `docs` task for Theta when a release changes user- or operator-facing behaviour and the docs were not updated.
- After updating: nothing to migrate.

## v0.5.4 — 2026-11-24
- **Smoke test proves the hooks load, without calling any model.** It sends `uno-smoke <nonce>`; the `approve_on_prompt` hook logs the nonce and blocks the prompt before it reaches the model. The old check passed whenever the model's answer mentioned "T99", and with a local model it waited on a slow model call.
- Local check: Ollama reachable, model present, `/v1/messages` answers (180 s limit), then the same hook proof. Each Claude Code call is limited to 60 s (portable `perl alarm`, macOS has no `timeout`).
- After updating: nothing to migrate.

## v0.5.3 — 2026-11-24
- Plugin and marketplace descriptions match what UNO does now (shown in `/plugin`).
- Recommended install source is the GitHub repo (`/plugin marketplace add https://<user>@github.com/<user>/UNO.git`), so only pushed releases reach project sessions.
- **Which model did the work is recorded.** Plan approval by chat adds `backend:` (anthropic or the host of `ANTHROPIC_BASE_URL`) and `model:` to the `Approved-by:` line and to the audit log; the Done log copies them. Terminal approvals record `backend:unknown`.
- **Work done with a non-Anthropic model is always `review: careful`** (condition 9), so the owner reads the diff in full.
- **`scripts/smoke_test.sh <project> [local]`** after updating Claude Code, Ollama or UNO: hook tests, `uno-stable` on a tag, `.uno` link, and a real `claude -p "approve T99"` per backend to prove the hooks load. Set `UNO_LOCAL_URL` for the local check.
- ONBOARDING: "Local models (Ollama)" — `uno` / `uno-local` shell functions, separate history with `CLAUDE_CONFIG_DIR`, never `--continue` across backends.
- After updating: nothing to migrate.

## v0.5.2 — 2026-11-24
- **Works in Auto mode.** Package installs are no longer an `ask` prompt; `guard_plan` lets an install through only if every package is listed under the approved plan's "Dependencies", blocks unlisted ones as a plan change, blocks installs outside a task or before approval, and blocks chained install commands. `npm install` / `npm ci` from the lock file stay allowed.
- **`/core:work` opens draft PRs with `gh`** when it is signed in (else writes the PR text for the owner), and marks merged PRs `done` at the start of the next run. `git fetch` only, never `git pull`, in a task.
- Settings templates deny risky `gh` actions (merge, close, review, ready, api write methods, repo, release, secret, variable, workflow, auth) and `git pull`; allow `gh pr create/view/list/edit/checks/diff`.
- Hooks (plan gate, git guard, zones, self-approval block) enforce in every permission mode, including Auto.
- **Shell writes need an approved plan too.** `guard_plan` blocks `sed -i`, redirection into project files, `tee`, `cp`, `mv`, `rm`, `touch`, `git apply`, `patch`, `git restore` before the plan is approved or outside a task branch (writes to `.uno/`, `/tmp`, `/dev/null` stay allowed).
- **Hook regression tests:** `tests/test_hooks.py` (52 cases: git guard, plan gate, self-approval, prompt approval, installs, shell writes). The pre-commit hook runs it after the data check; a failing test blocks the commit.
- **Stable copy for daily use:** run project sessions from `~/UNO/uno-stable` (a git worktree checked out at a release tag), not from the working copy in `~/UNO/uno`, so uncommitted UNO edits never reach real work. See ONBOARDING "Install, update, rollback".

## v0.5.1 — 2026-11-23
Fixes from the first real run (T7):
- **Approve by typing `approve T7`** (or `อนุมัติ T7`) in chat. New `UserPromptSubmit` hook `approve_on_prompt` reacts only to human prompts, so an agent cannot approve. The agent tried to run `approve_plan.py` itself; `guard_plan` now blocks any Bash call that mentions `approve_plan` or `Approved-by`.
- **A plan cannot be approved with missing sections** (Goal, UI design, Dependencies, Files to change, Steps, Verification, Rollback) — checked by both the prompt hook and `approve_plan.py`.
- **Plans use facts from the repo now**: base sha from `origin/<default>` after fetch, tier and review from the profile, counts measured on that base (the T7 plan reused a stale base, branch and tier).
- Skill names its template path and lists the sections as a fallback; prefers allowed commands over `node -e` / `python -c`.
- Project sessions may read `~/UNO/uno` but not edit or write it (settings templates use `<home>`).
- After updating: in each onboarded project's `.claude/settings.local.json` add `Read(//<home>/UNO/uno/**)` to allow and `Edit(…)`, `Write(…)` of the same path to deny; remove any deny rule naming `approve_plan`.

## v0.5.0 — 2026-11-23
- **Full team of 8.** New agents: Alpha (`intake-triager`), Beta (`codebase-explorer`, read only), Delta (`test-writer`, test files only), Zeta (`release-manager`, `.uno/releases/` only, never tags or deploys), Eta (`incident-scribe`, `.uno/` only, never touches servers).
- **New skills close the SDLC loop:** `/core:intake` (request → spec in `.uno/specs/` → proposed tasks), `/core:release` (merged PRs → `.uno/releases/<version>.md` with deploy and rollback; owner tags and deploys), `/core:incident` (report → `.uno/incidents/` → proposed fix and regression test).
- `/core:work` calls Beta before planning and Delta for planned tests. Team roster in `references/team.md` shows what each member may edit.
- Not yet used on a real task: Alpha, Beta, Delta, Zeta, Eta.
- After updating: nothing to migrate.

## v0.4.3 — 2026-11-23
- **Plan gate on every task.** New hook `guard_plan`: no edit outside `.uno/` until `.uno/plans/<task-id>.md` is approved by the owner and unchanged since. The branch must carry the task id (`fix/T7-slug`).
- **Owner-only approval:** `scripts/approve_plan.py <task-id>` appends `Approved-by: <name> <time> sha256:<hash>`; run it with `!` inside Claude Code or in a terminal. Agents cannot write that line; changing the plan voids it.
- **UI is designed in the plan:** Gamma runs in `design` mode (read only) while the plan is written; its DESIGN PROPOSAL goes into the plan's "UI design" section, so the owner approves design and code together. After approval Gamma runs in `build` mode and stays inside that design.
- **The approved plan is the go-ahead.** `/core:work` runs end to end and stops only for decisions the plan did not make: plan approval, outside the plan, escalation. Anything that affects the plan (files, steps, packages, design, approach, acceptance, risk, rollback) is brought to the owner as a joint decision with options and a recommendation, then written into the plan and re-approved. Only details that leave the plan unchanged (names, test layout, fixing errors the change caused) are decided alone. Package installs still ask every time.
- **Fewer permission prompts:** settings templates `allow` read-only and task-branch commands (git status/diff/log/switch/cherry-pick/add/commit/push, gh pr, npm lint/test/build, tsc, vitest). `guard_git` and `guard_plan` still block the unsafe ones.
- Plan template `work/templates/plan.md` with rollback point, one commit per step and a rollback section.
- Known limit: edits made through Bash (redirection, `sed -i`) bypass the gate; `/core:work` forbids them and the audit log shows Bash commands.
- After updating: add the `allow` list from `settings.<tier>.json` to each onboarded project's `.claude/settings.local.json`. Tasks already in progress need a plan approved before the next edit.

## v0.4.2 — 2026-11-19
- **`guard_git` blocks checkout/switch to old branches** whose tree still tracks `.uno/` or `CLAUDE.md` while those are symlinks; the message tells the agent to cherry-pick onto a fresh branch. `/core:work` states the same rule.
- **No permission prompt for `.uno`:** settings templates add `additionalDirectories: ["<uno-data-dir>"]`; onboarding fills in the absolute `~/UNO/data/<project>` path.
- `THIRD_PARTY.md`: Ponytail listed under evaluation with its checks.
- After updating: add `"additionalDirectories": ["/Users/<you>/UNO/data/<project>"]` under `permissions` in each onboarded project's `.claude/settings.local.json`.

## v0.4.1 — 2026-11-18
- **Owner reviews alone by default.** New profile field `Review: self | second-reviewer`; onboarding proposes it from the repo's branch protection. PRs get `review: careful` (with which of 8 conditions apply) or `review: quick`.
- **UI and logic in one task.** Gamma is a step inside full mode; split a task only when it is too big to review (~300 lines or more than one screen/flow).
- **Project data in `~/UNO/data/<project>/`**, linked as `.uno`; `.git/info/exclude` uses `.uno` (no slash) so the symlink is ignored. Back it up locally; never push it.
- **Done log is a summary pointing to git** (branch, PR, merge commit); `/core:work` reads only the last 5 entries.
- Data check also blocks `.DS_Store` and `Thumbs.db`.
- After updating: for each onboarded project, move `.uno/` into `~/UNO/data/<project>/`, replace it with the symlink, change `.uno/` to `.uno` in `.git/info/exclude`, and add `Review: self` (or `second-reviewer`) to the profile header.

## v0.4.0 — 2026-11-17
- **Tiers work for any kind of project:** `open` · `internal` · `sensitive` replace `standard` · `clinical`. Hospital rules become one example of `sensitive`; project-specific display rules move to `.uno/project-profile.md`.
- **Always-on guards in every tier:** secrets unreadable, installs ask first (`settings.internal.json`, `settings.sensitive.json` replace the old templates).
- **Data check for this repo:** `.githooks/pre-commit` blocks commits carrying project names from the registry, extra terms from `.forbidden-terms`, foreign e-mails, 13-digit IDs or secret-like strings.
- `THIRD_PARTY.md`: review checklist and list of external plugins/skills.
- Scanner detects personal data beyond health (passport, tax ID, salary, card, bank account, employee ID).
- After updating: change `tier: clinical` → `sensitive` and `standard` → `internal` (or `open`) in `registry/projects.yaml`; run `git config core.hooksPath .githooks`.

## v0.3.0 — 2026-11-15
- **Personal mode is the default.** Onboarding no longer opens a setup PR. UNO files (`.uno/`, `CLAUDE.md`, `.claude/settings.local.json`) stay local through `.git/info/exclude`.
- Settings templates hold only `permissions`; no marketplace entries in projects.
- Install/update/rollback section in `ONBOARDING.md`.
- After updating: run the migration for projects onboarded with v0.1–v0.2 (move `.claude/settings.json` UNO entries to `settings.local.json`, exclude UNO files, remove them from shared branches).

## v0.2.3 — 2026-11-15
- Gamma (`ui-designer`) works for any project: design intent `match` / `refresh` / `new`, accessibility baseline, extra rules only for `clinical`.
- UI tasks that need logic are split into a logic task, then a UI task (`depends_on`).

## v0.2.2 — 2026-11-15
- `trace_agents` hook logs every skill and subagent call to `.uno/audit.log`.
- Gamma preloads `frontend-design` via `skills:` and reports `loaded` / `not loaded`.

## v0.2.1 — 2026-11-15
- Gamma (`ui-designer`) added; team roster Uno, Alpha…Eta in `references/team.md`, reserved names Theta, Iota, Kappa, Lambda.

## v0.2.0 — 2026-11-15
- `/core:work` with small/full modes and Superpowers mapping; `.uno/tasks.md`; Epsilon (`security-reviewer`); `guard_git` hook; scanner skips gitignored build output.

## v0.1.0 — 2026-11-14
- First release: portfolio-scan, onboard, new-project, refactor, `guard_zones` hook.
