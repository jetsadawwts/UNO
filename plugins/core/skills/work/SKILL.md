---
name: work
description: Run one task from an onboarded project's `.uno/tasks.md` under UNO rules — pick the small or full mode, branch from the default branch, delegate to Superpowers skills when installed, verify, and hand over a PR. Use when the user says "do task T3", "next UNO task", "work on the backlog", or asks UNO to make a change in an onboarded project.
---

# Work one task

UNO decides **what** may be done and **how safe** it must be. The Superpowers skills (if installed) decide **how** the code gets written. This skill joins the two.

## Run the whole flow — stop only at these points

Once started with a task id, carry the task through every step below in one go: branch, plan, implement, verify, review, PR, task row. Do not ask "shall I continue?", do not wait between steps, and do not hand the user commands to type that you can run yourself. The approved plan is the owner's go-ahead for everything in it. After approval, ask nothing about work the plan covers. Stop **only** for a decision the plan did not make:

1. **Plan approval** — the plan is written; ask the owner to run `approve <task-id>` as a chat message (e.g. `approve T7`), then continue on your own once they say it is done. List every package to install, upgrade or remove under "Dependencies".
2. **Anything that affects the plan** — a file, step, package, design, approach, acceptance check, risk or rollback different from what was approved, even if small or clearly better. Do not decide alone: bring it to the owner as a joint decision (format below), update the plan with the agreed option, and go back to 1.
3. **Escalation** — 3 failed attempts at the same check, Epsilon BLOCK, or a NEEDS HUMAN DECISION item.
4. **Package installs** — not asked separately: `guard_plan` lets an install through only if every package is listed under the approved plan's "Dependencies"; anything else is a plan change (point 2). Run each install as one plain command.

**Decide yourself, without asking**, only details that leave the plan exactly as approved: names of variables, functions, tests and commits; how to lay out a test; formatting of lines you already change; re-running a flaky check; fixing a lint or type error your own change caused. Write the notable ones in the PR description. If you are unsure whether something affects the plan, treat it as if it does.

**How to ask for a joint decision** (keep it short):
```
PLAN CHANGE — <task-id>
What I found: <one or two lines, with file:line>
Why it matters for the plan: <scope / steps / risk / rollback>
Options:
  A) <option> — effect on plan, effort, risk, rollback
  B) <option> — ...
  C) stop the task here (only if sensible)
My recommendation: <A/B> because <one line>
```
Wait for the owner's choice, write it into the plan (with a "Decision" line: what was chosen and why), then ask for re-approval.

**Always ask** (even if it looks small) when the choice changes what users see or what data leaves the system, touches a `review` or `forbidden` zone beyond the plan, changes how a sensitive field is shown, or cannot be undone with git.

When the PR is open, finish with a short report (PR link, review label, what the owner should read) — that is the end of the task, not a question. At each stop say in one line why you stopped and what the owner must decide.

## 0. Preconditions — stop if any fail

- `.uno/zones` and `.uno/tasks.md` exist. Missing → the project is not onboarded; tell the user to run `/core:onboard`.
- Read the project's registry entry (`tier`, `autonomy`), `CLAUDE.md`, the header of `.uno/project-profile.md` (`Tier`, `Review`) and the **last 5 entries** of the Done log in `.uno/tasks.md` plus any task in `depends_on`. Do not read `.uno/audit.log` or the whole history; it is for retro and benchmarks.
- The task is in `.uno/tasks.md` with status `ready` or `in-progress`. A request that is not in the backlog: add it as a new row with status `proposed` and stop until a human marks it `ready`.
- The task's level is allowed: L1 → you may only propose a diff or plan in chat and open a PR the human reviews; never merge. `sensitive` tier caps at L2 and never auto-merges; `internal` and `open` may reach L3.

## 1. Pick the mode

| Mode | Use when **all** are true | Steps |
|---|---|---|
| **small** | touches ≤ 3 files · no `review` zone · no behaviour change, or behaviour is pinned by existing tests · no new dependency | short plan → **owner approves** → change → verify → PR |
| **full** | anything else, and every UI task | brainstorm → written plan → **owner approves** → worktree → implement with subagents → two-stage review → verify → PR |

Write the chosen mode and the reason into the task row. When unsure, use **full**.

## 2. Branch — one task, one branch, from the default branch

1. Find the default branch: `git symbolic-ref --short refs/remotes/origin/HEAD` (strip `origin/`); fall back to `main`, then `master`.
2. `git fetch origin` (never `git pull` in a task; it can merge outside the plan) then create `<type>/<task-id>-<slug>` **from `origin/<default>`**, e.g. `test/T1-utils-characterization`. Types: `feat`, `fix`, `test`, `chore`, `docs`, `refactor`.
3. Never check out a branch created before the project moved to personal mode (its tree still tracks `.uno/` or `CLAUDE.md`; `guard_git` blocks it). To reuse its work: `git switch -c <type>/<task-id>-<slug> origin/<default>` then `git cherry-pick <its commits>`.
4. Never build a task branch on top of another task branch. If the task truly depends on unmerged work, set `depends_on` in the task row, branch from that branch, and say so in the PR description so it is merged in order.
5. Never commit on the default branch. The `guard_git` hook blocks it.
6. In **full** mode use a git worktree (`superpowers:using-git-worktrees` if available) so parallel tasks do not share a working tree.

## 2b. Plan gate — every task, every mode

No code edit before the owner approves a plan. The `guard_plan` hook enforces it: edits outside `.uno/` are blocked unless the branch carries the task id and `.uno/plans/<task-id>.md` is approved and unchanged since.

1. Write `.uno/plans/<task-id>.md` from `templates/plan.md` in this skill's base directory (if it is missing, use these sections: Goal and acceptance check, UI design, Dependencies, Files to change, Steps, Verification, Rollback, Decisions during the work, Risks and out of scope). Take facts from the repo now, not from old task rows: base = `git rev-parse --short origin/<default>` after `git fetch`, tier and review from `.uno/project-profile.md`, test and lint counts measured on that base. Content: rollback point (base commit), files and zones, steps (one commit each), verification, rollback, risks. In full mode this is the output of `superpowers:writing-plans`, saved there. Small mode may keep each section to one line, but no section is skipped.
2. Show the plan in chat and **stop**. Ask the owner to approve with `approve <task-id>` as a chat message (e.g. `approve T7`). Never write the `Approved-by:` line and never run `approve_plan.py` yourself; hooks block both. Only the owner's own chat message approves (the `approve_on_prompt` hook reacts to human prompts only).
3. If the work needs anything the plan does not cover (another file, another step, a dependency), stop, update the plan, and ask again. Editing the plan voids the approval automatically.
4. Make every change with the Edit/Write tools, never with shell redirection or `sed -i`, so the gate and the audit log see it.
5. Prefer commands the project already allows (`cat`, `grep`, `git …`, `npm run …`, `npx tsc`) over `node -e`, `python -c` or other inline scripts, which need a permission prompt and can do anything.
6. Never edit files under `~/UNO/uno` from a project session; UNO changes are made in the UNO repo and released as a version.

## 3. Do the work

**If Superpowers is installed**, invoke its skills instead of improvising:

| Step | Skill |
|---|---|
| clarify the change (full mode only) | `superpowers:brainstorming` — skip when the task row already has clear acceptance criteria |
| plan | `superpowers:writing-plans` |
| implement | `superpowers:subagent-driven-development` (or `superpowers:executing-plans` for human checkpoints) |
| tests | `superpowers:test-driven-development` |
| debugging | `superpowers:systematic-debugging` |
| before saying "done" | `superpowers:verification-before-completion` |
| finish | `superpowers:finishing-a-development-branch` — but always choose **open a PR**, never merge locally |

**If Superpowers is not installed**, use the built-in loop: Plan → Act → Verify → Reflect.

**Team members inside `/core:work`:**
- Before writing the plan, ask Beta (`codebase-explorer`) concrete questions instead of reading whole folders yourself; put its file:line evidence in the plan's "Files to change".
- When a planned step needs tests, call Delta (`test-writer`) with that step: characterization tests first (commit them separately), then failing tests for the new behaviour; you change the code.
- UI: Gamma (`ui-designer`) `design` mode for the plan, `build` mode after approval. Security: Epsilon (`security-reviewer`) before every PR.
- Documentation (README, user guide, deploy guide, developer docs): Theta (`doc-writer`) writes it inside the plan. A code task that changes what users or operators do adds a final plan step for Theta to update the affected docs; list those files in the plan.

**UNO rules that override any skill:**
- Legacy code without tests: write characterization tests that pin current behaviour **before** changing it. Commit them separately.
- No feature work and bug fixes in the same branch. No refactor mixed with behaviour change.
- Any subagent you start gets a brief in the format of `references/subagent-brief.md`. Team names and codes are in `references/team.md`; refer to members as `Name (code)`.
- Files in a `review` zone: when done, run Epsilon (`security-reviewer`) on the diff and paste its verdict in the PR.
- UI tasks (the main change is how screens or components look): always **full** mode. Call Gamma (`ui-designer`) twice with a 4-part brief and a design intent (`match`, `refresh` or `new`; default `match` for existing products): first in `design` mode while writing the plan, and paste its DESIGN PROPOSAL into the plan's "UI design" section so the owner approves the design and the code plan together; then, after approval, in `build` mode. Paste its BEFORE / AFTER and NEEDS HUMAN DECISION into the PR, then run Epsilon (`security-reviewer`). In `sensitive` projects a UI task counts as touching a `review` zone. Logic and UI may change in the same task: Gamma is a step inside full mode, not a separate flow.
- Split a task only when it is too big to review in one sitting: more than ~300 changed lines, or more than one screen or user flow. Each part becomes its own task with `depends_on`. Logic changes in a split still come first, with tests.
- Anything in `forbidden`: do not try workarounds through the shell. Describe the change and hand it to a human.
- Before calling a dependency unused, search `src` **and** build config files (vite, tailwind, postcss, webpack, babel, tsconfig, package scripts).

## 3b. Look at the running app (UI tasks, user guides, E2E checks)

How far to look depends on the change (write the level into the plan's Verification):

| Change | On screen |
|---|---|
| touches app code (`src/` or equivalent) but no behaviour: types, lint, renames | **smoke**: the app starts, each main page in the profile's `URL` list loads, no errors in the browser console |
| logic: filters, calculations, conditions, data handling | smoke + every step of the acceptance check done on screen |
| UI: screens, components, layout | smoke + the steps + BEFORE / AFTER screenshots |
| export, print, PDF, download | smoke + the steps + open the produced file and look at it |
| docs only, tests only | none (unless the docs need screenshots) |

Only when `.uno/project-profile.md` has a `## Preview` section with `- Data: fake`. Otherwise skip, write "screens not checked: no fake-data preview" in the PR, and label it `review: careful` (condition 10).

1. Use the built-in browser (Claude desktop app). Claude in Chrome only if the owner asks: it carries all of the owner's own logins.
2. Start the app with the profile's `Run` command if it is not running; open its `URL`. The `guard_browser` hook blocks any host other than localhost and the profile's `Allowed hosts`.
3. If a login page appears, stop and ask the owner to sign in by hand in that browser. Never type, read back, or store a password.
4. Do what the plan's Verification lists on screen. Save screenshots to `.uno/designs/<task-id>/` (`before-*.png`, `after-*.png`), never into the repo, unless the plan says they belong in the user docs (then Theta places them under `docs/`).
5. Hand Gamma and Theta the screenshot paths; they cannot drive the browser themselves.

## 4. Verify — evidence, not opinion

Run the task's acceptance check plus the project's baseline commands from `CLAUDE.md` (build, type check, lint, tests). Paste the actual command output summary in the PR. A failing check that also failed at baseline is recorded, not hidden.

Retry limit: 3 failed attempts at the same check → stop, set status `escalated`, write what you tried and the last error into the task row, and tell the user.

## 5. Hand over

1. Push the branch. If `gh pr list --limit 1` works, open the PR yourself with `gh pr create --draft --base <default> --title "<type>: <task title> (<task-id>)" --body-file <file>`. If `gh` is missing or not signed in, write the title and body into the task's PR column notes and give the owner the compare link. Never `gh pr ready`, `gh pr merge` or approve; settings deny them.
2. PR description: what changed, why, how verified (outputs), zones touched, Epsilon (security-reviewer) verdict, `depends_on` if any, and a review label (below).
3. Update the task row: status `in-review`, branch, PR link.
4. At the start of every `/core:work`, check tasks in `in-review` with `gh pr view <n> --json state,mergeCommit` (if `gh` works). Merged → status `done` and one Done log entry (copy `backend:` and `model:` from the plan's `Approved-by:` line) in the format of `templates/tasks.md`: a short summary that points to the branch, PR and merge commit. Code history lives in git; never copy diffs or code into `.uno/`.

### Review label

Read `Review` from `.uno/project-profile.md`:

- `self` (default): the owner reviews alone. Label the PR **`review: careful`** if any of these hold, else **`review: quick`**:
  1. touches a `review` zone
  2. changes how personal or sensitive data is shown
  3. UI change in a `sensitive` project
  4. adds or changes an export, report or download
  5. adds, removes or upgrades a runtime dependency
  6. touches auth, config, deploy or CI files
  7. Epsilon (security-reviewer) gave anything but a plain PASS
  8. the PR lists a NEEDS HUMAN DECISION item
  9. the plan's `Approved-by:` line shows a `backend:` other than `anthropic` (work done with a local or third-party model)
  10. the change touches app code and the on-screen check (3b) was skipped or failed
  For `review: careful`, list which of the 10 apply so the owner knows what to read. `review: quick` still needs Epsilon PASS and green checks.
- `second-reviewer`: the repo requires another approver. Say so in the PR, add the label as above, and never suggest the owner merge alone.

Never merge, never push to the default branch, never close someone else's PR.
