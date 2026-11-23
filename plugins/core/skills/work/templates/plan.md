# Plan <task-id>: <task title>

- Branch: `<type>/<task-id>-<slug>` from `origin/<default>` at `<short sha>` (rollback point)
- Mode: small | full · Tier: <tier> · Expected review label: quick | careful (which of the 8 conditions)

## Goal and acceptance check
<one or two lines; the exact command or observation that proves it is done>

## UI design (Gamma) — required when the task changes how anything looks; else write "none"
<paste Gamma's DESIGN PROPOSAL here: intent, screens/components, before → after, states,
sensitive fields and how they are shown, accessibility, new tokens/dependencies, needs human decision>

## Files to change
| File | Zone | What changes |
|---|---|---|

## Dependencies (install / upgrade / remove)
<exact package names and versions, e.g. `xlsx 0.20.3 (upgrade, fixes CVE)`, or "none".
The owner is still asked at each install; a package not listed here is outside the plan.>

## Steps (one commit each, so any step can be reverted alone)
1.
2.

## Verification
<commands: lint, tsc, tests, build, plus the acceptance check>

## Rollback
- Before merge: delete the branch, or `git revert <step commit>`.
- After merge: `git revert -m 1 <merge commit>`.
- Anything git cannot undo (data, config on a server, a dependency in production): <none | describe>

## Decisions during the work
<added after approval when the plan changes: date · what changed · options considered · chosen and why>

## Risks and out of scope
- Risk:
- Not doing:
