---
name: onboard
description: Onboard one existing (legacy) project into UNO — read-only discovery by the virtual team, a project profile, a human review gate, then CLAUDE.md, zones and a baseline before UNO takes on work. Use when the user says "onboard this project", "analyze this repo for UNO", or wants UNO to continue work on an old codebase.
---

# Onboard a legacy project

UNO treats an existing project like a new hire joining a team: learn first, change nothing, get corrected by the people who know it, then start with low-risk work.

All outputs go to `.uno/` in the project root plus a `CLAUDE.md`. Status is tracked in `.uno/onboarding.md` so onboarding can pause and resume across sessions.

## Phase 0 — Check the registry

- Look up the project in the UNO registry (`registry/projects.yaml`). Use its `tier` and `wave`.
- Not registered → stop and ask the CTO to run `portfolio-scan` or add the entry. Do not guess the tier.
- Keep project data in one backed-up place: create `~/UNO/data/<project>/` and link it with `ln -s ~/UNO/data/<project> .uno` from the project root. If `.uno/` already exists as a folder, move its files into the data folder first. `~/UNO/data` holds company details: back it up (e.g. Time Machine), never push it to a personal remote.
- Create `.uno/onboarding.md` from `templates/onboarding.md`. Mark Phase 0 done.

## Phase 1 — Discovery (read-only)

Hard rule: no edits to project code, no installs, no migrations, no network calls to production, no running code that touches real data. Reading files, `git log`, and listing dependencies is allowed. Running the existing test suite is allowed for `open` and `internal` tier, and for `sensitive` only if the owner says yes.

Run each lens as a separate pass (subagent if available) and write findings into the profile sections:

| Lens | Answers | Profile section |
|---|---|---|
| Tech Lead | architecture, modules, entry points, build/deploy, real conventions (from code, not docs) | 1, 2, 3 |
| Backend/Integration | data stores, external systems (HIS, HL7/FHIR, LIS/PACS, card readers), jobs, APIs | 4 |
| Security | where personal or sensitive data flows, secrets in repo, auth model, logging of sensitive fields | 5 |
| QA | test types, how to run them, coverage gaps, flaky areas | 6 |
| DevOps/SRE | environments, CI/CD, config, observability | 3, 7 |
| SA | what the system is for, who uses it, known pain points from issues/commits | 0 |

For repos over ~100k LOC, do discovery module by module and record which modules are done in `.uno/onboarding.md`.

Dependency findings: before calling a package unused, search `src` **and** build config files (vite, tailwind, postcss, webpack, babel, tsconfig, eslint, package.json scripts). Plugins loaded with `require()` in config are easy to miss.

Uncommitted work: if `git status` shows changes, stop and ask the owner to commit them on their own branch before discovery, so the baseline is a commit, not a working tree.

Every claim in the profile carries a confidence tag: `[seen]` (read in code), `[inferred]` (deduced), `[unknown]` (needs a human). Never write `[seen]` for something you did not open.

Output: `.uno/project-profile.md` from `templates/project-profile.md`, and a draft `.uno/zones.draft` from `templates/zones`.

## Phase 2 — Human review gate (required)

Stop and hand over to the project owner with:
- the profile, with every `[inferred]` and `[unknown]` listed as questions
- proposed zones, especially anything marked `forbidden`
- proposed `Tier` and `Review` for the profile header. For `Review`, check whether the default branch requires approvals: `gh api repos/<owner>/<repo>/branches/<default>/protection --jq .required_pull_request_reviews.required_approving_review_count`. A number ≥ 1 → propose `second-reviewer`; no protection, 0, or the check fails → propose `self` and say the check could not confirm it.
- the top 5 risks found

Do not continue until the owner has corrected the profile and written `approved-by` and date in `.uno/onboarding.md`. Corrections from the owner become `[confirmed]`.

## Phase 3 — Setup

UNO is the owner's personal tool. Its files stay on the owner's machine and are **never committed to a shared repo** unless the owner says the repo is personal.

Only after approval:
1. Write `CLAUDE.md` at the project root: short, linking to `.uno/project-profile.md`; include build/test commands, conventions, zones summary, and "ask before" items. Keep it under ~150 lines.
2. Ask the owner to review `.uno/zones.draft` and rename it to `.uno/zones` themselves. Once it exists the `guard_zones` hook enforces it, and agents can no longer edit it.
3. Write `.claude/settings.local.json` with the `permissions` block of the tier template (`new-project/templates/settings.<tier>.json`). Replace `<uno-data-dir>` with the absolute path of `~/UNO/data/<project>` (so Claude Code reads and writes `.uno` through the symlink without asking each session) and `<default-branch>` with the repo's real default branch (`git symbolic-ref --short refs/remotes/origin/HEAD`); do not assume `main`. Do not add marketplace or plugin entries: `core@uno` is installed at user scope.
4. Create `.uno/tasks.md` from `work/templates/tasks.md`.
5. Keep these files out of the shared repo without touching its `.gitignore`: append `.uno` (no trailing slash, so the symlink matches), `CLAUDE.md` and `.claude/settings.local.json` to `.git/info/exclude`. If the repo already tracks a `CLAUDE.md` owned by the team, write UNO's notes to `.uno/CLAUDE.uno.md` instead and add `@.uno/CLAUDE.uno.md` to nothing shared — tell the owner so they can decide.
6. No setup PR. Report to the owner what was written and that `git status` shows none of it.

## Phase 4 — Baseline

Record in `.uno/baseline.md` (from git history and CI where available): median PR lead time, commits/month, open bugs, test count and pass rate, last incident. Without a baseline, UNO's effect cannot be measured.

## Phase 5 — First tasks at L1

Write 3 to 5 starter tasks into `.uno/tasks.md` with status `proposed`, all low risk, ranked:
1. characterization tests for an untested but stable module
2. documentation gaps found in discovery
3. small, isolated bug fixes
4. dependency or lint cleanup with no behaviour change

The owner marks the ones they want as `ready`. Each is then done with `/core:work <task-id>`: one branch per task, each cut from the default branch (not stacked on another task branch), one PR each.

No feature work, refactors across modules, or anything in `review` or `forbidden` zones until the project has 2 sprints without escalations. Then the CTO may promote it to L2 in the registry.

## After onboarding

- When UNO gets something wrong because the profile was wrong, fix the profile in the same PR and log it under "Corrections" in `.uno/onboarding.md`.
- Corrections that repeat across projects go to the UNO portfolio review, not just the project.
