---
name: work
description: Run one task from an onboarded project's `.uno/tasks.md` under UNO rules — pick the small or full mode, branch from the default branch, delegate to Superpowers skills when installed, verify, and hand over a PR. Use when the user says "do task T3", "next UNO task", "work on the backlog", or asks UNO to make a change in an onboarded project.
---

# Work one task

UNO decides **what** may be done and **how safe** it must be. The Superpowers skills (if installed) decide **how** the code gets written. This skill joins the two.

## 0. Preconditions — stop if any fail

- `.uno/zones` and `.uno/tasks.md` exist. Missing → the project is not onboarded; tell the user to run `/core:onboard`.
- Read the project's registry entry (`tier`, `autonomy`), `CLAUDE.md`, the header of `.uno/project-profile.md` (`Tier`, `Review`) and the **last 5 entries** of the Done log in `.uno/tasks.md` plus any task in `depends_on`. Do not read `.uno/audit.log` or the whole history; it is for retro and benchmarks.
- The task is in `.uno/tasks.md` with status `ready` or `in-progress`. A request that is not in the backlog: add it as a new row with status `proposed` and stop until a human marks it `ready`.
- The task's level is allowed: L1 → you may only propose a diff or plan in chat and open a PR the human reviews; never merge. `sensitive` tier caps at L2 and never auto-merges; `internal` and `open` may reach L3.

## 1. Pick the mode

| Mode | Use when **all** are true | Steps |
|---|---|---|
| **small** | touches ≤ 3 files · no `review` zone · no behaviour change, or behaviour is pinned by existing tests · no new dependency | plan in 5 lines → change → verify → PR |
| **full** | anything else, and every UI task | brainstorm → written plan → worktree → implement with subagents → two-stage review → verify → PR |

Write the chosen mode and the reason into the task row. When unsure, use **full**.

## 2. Branch — one task, one branch, from the default branch

1. Find the default branch: `git symbolic-ref --short refs/remotes/origin/HEAD` (strip `origin/`); fall back to `main`, then `master`.
2. `git fetch origin` then create `<type>/<task-id>-<slug>` **from `origin/<default>`**, e.g. `test/T1-utils-characterization`. Types: `feat`, `fix`, `test`, `chore`, `docs`, `refactor`.
3. Never check out a branch created before the project moved to personal mode (its tree still tracks `.uno/` or `CLAUDE.md`; `guard_git` blocks it). To reuse its work: `git switch -c <type>/<task-id>-<slug> origin/<default>` then `git cherry-pick <its commits>`.
4. Never build a task branch on top of another task branch. If the task truly depends on unmerged work, set `depends_on` in the task row, branch from that branch, and say so in the PR description so it is merged in order.
5. Never commit on the default branch. The `guard_git` hook blocks it.
6. In **full** mode use a git worktree (`superpowers:using-git-worktrees` if available) so parallel tasks do not share a working tree.

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

**UNO rules that override any skill:**
- Legacy code without tests: write characterization tests that pin current behaviour **before** changing it. Commit them separately.
- No feature work and bug fixes in the same branch. No refactor mixed with behaviour change.
- Any subagent you start gets a brief in the format of `references/subagent-brief.md`. Team names and codes are in `references/team.md`; refer to members as `Name (code)`.
- Files in a `review` zone: when done, run Epsilon (`security-reviewer`) on the diff and paste its verdict in the PR.
- UI tasks (the main change is how screens or components look): always **full** mode. Delegate the visual change to Gamma (`ui-designer`) with a 4-part brief plus a design intent (`match`, `refresh` or `new`; default `match` for existing products). Paste its DESIGN INTENT, BEFORE / AFTER and NEEDS HUMAN DECISION into the PR, then run Epsilon (`security-reviewer`). In `sensitive` projects a UI task counts as touching a `review` zone. Logic and UI may change in the same task: Gamma is a step inside full mode, not a separate flow.
- Split a task only when it is too big to review in one sitting: more than ~300 changed lines, or more than one screen or user flow. Each part becomes its own task with `depends_on`. Logic changes in a split still come first, with tests.
- Anything in `forbidden`: do not try workarounds through the shell. Describe the change and hand it to a human.
- Before calling a dependency unused, search `src` **and** build config files (vite, tailwind, postcss, webpack, babel, tsconfig, package scripts).

## 4. Verify — evidence, not opinion

Run the task's acceptance check plus the project's baseline commands from `CLAUDE.md` (build, type check, lint, tests). Paste the actual command output summary in the PR. A failing check that also failed at baseline is recorded, not hidden.

Retry limit: 3 failed attempts at the same check → stop, set status `escalated`, write what you tried and the last error into the task row, and tell the user.

## 5. Hand over

1. Push the branch and open a PR (draft if L1). Title: `<type>: <task title> (<task-id>)`.
2. PR description: what changed, why, how verified (outputs), zones touched, Epsilon (security-reviewer) verdict, `depends_on` if any, and a review label (below).
3. Update the task row: status `in-review`, branch, PR link.
4. When the human merges, the next session marks it `done` and writes one Done log entry in the format of `templates/tasks.md`: a short summary that points to the branch, PR and merge commit. Code history lives in git; never copy diffs or code into `.uno/`.

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
  For `review: careful`, list which of the 8 apply so the owner knows what to read. `review: quick` still needs Epsilon PASS and green checks.
- `second-reviewer`: the repo requires another approver. Say so in the PR, add the label as above, and never suggest the owner merge alone.

Never merge, never push to the default branch, never close someone else's PR.
