# UNO changelog

Newest first. Each entry: what changed, why, and what to do after updating.

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
