# UNO changelog

Newest first. Each entry: what changed, why, and what to do after updating.

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
