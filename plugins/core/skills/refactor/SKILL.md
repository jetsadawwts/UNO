---
name: refactor
description: Refactor existing code under UNO — behaviour must not change. Requires an onboarded project, characterization tests first, an ADR, then small behaviour-preserving PRs (strangler pattern for large changes), measured against baseline. Use when the user asks to refactor, restructure, clean up, modernize or migrate a module of an existing codebase.
---

# Refactor

Goal: change the structure, keep the behaviour. If behaviour must change, that is a feature or a fix — do it separately.

## Preconditions (check first, stop if any fails)

- Project is in the registry with `onboarding: done` and `.uno/project-profile.md` exists.
- Project autonomy is L2 or higher. At L1, produce only the ADR and plan; a human does the code.
- The target is not in a `forbidden` zone. If it is, stop and ask the owner to make the change or to open the zone temporarily, with the reason logged in `.uno/onboarding.md`.
- No open feature work is changing the same files (check open branches/PRs). If there is, agree an order with the owner.

## Step 1 — Why (gate: Tech Lead)

Write `docs/adr/<NNNN>-refactor-<area>.md` from `templates/refactor-adr.md`: the pain, the target shape, how success is measured, the rollback plan, and the scope boundary. Stop for approval.

## Step 2 — Safety net (gate: QA lens)

Before touching production code:
1. Write characterization tests that pin the current behaviour of the target, including odd behaviour. Name them `*characterization*` so they are easy to find.
2. Cover: public interfaces of the module, main paths from the critical flows in the profile, error paths, and any data written.
3. Run them on the current code: all must pass. Record the count and run time in the plan.
4. Commit tests alone in their own PR. Merge it before any refactor PR.

No safety net, no refactor. If the code cannot be tested as is, the first refactor step is a minimal seam (e.g. extract an interface) reviewed by a human line by line.

## Step 3 — Plan

Write `docs/refactor/<area>-plan.md` from `templates/refactor-plan.md`. Split into steps that are each:
- one PR, small enough to review in one sitting
- behaviour-preserving, with all existing tests passing and no existing test edited
- independently revertable

Large changes use strangler steps: build the new path beside the old, route a small slice to it behind a flag, compare, widen, then delete the old path last.

## Step 4 — Execute, one step at a time

For each step run the UNO inner loop (act → verify → reflect, max 3 tries, then escalate). Per PR:
- title prefix `refactor:`; description links the ADR and plan step
- no feature changes, no bug fixes, no formatting sweeps mixed in — log anything found in the plan's "Found along the way" list
- tests: characterization + existing suite pass; changed lines covered
- clinical tier: E2E regression on critical flows and Security sign-off before merge

Update the plan's checklist after each merge.

## Step 5 — Close

- Compare against `.uno/baseline.md` and the success measures in the ADR. Write the result in the ADR.
- Remove flags and the old path only after the agreed observation period.
- Update `.uno/project-profile.md` (modules, conventions) and `CLAUDE.md` if commands or structure changed.
- If the refactor revealed a pattern other projects share, raise it for the UNO portfolio review.

## Stop and escalate when

- a characterization test fails after a refactor step and the cause is not obvious in one look
- the step needs a schema/data migration not in the plan
- scope grows beyond the ADR boundary
