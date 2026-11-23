# UNO changelog

Newest first. Each entry: what changed, why, and what to do after updating.

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
