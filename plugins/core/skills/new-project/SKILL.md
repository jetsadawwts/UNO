---
name: new-project
description: Start a brand-new project the UNO way — intent, spec with risk tier, architecture ADR, then a scaffold that has CLAUDE.md, zones, UNO settings, CI and tests before the first feature. Use when the user wants to create a new app, service or tool, or says "new project" / "start from scratch".
---

# New project

A new project is born onboarded: UNO rules, tests and CI exist from the first commit. Each step produces one artifact and stops at its gate.

## Step 1 — Intent (gate: CTO)

Write `docs/intent.md` from `templates/intent.md`. Ask the user only what the template needs and cannot be inferred. Stop for approval.

## Step 2 — Spec and risk tier (gate: CTO; plus Security if sensitive)

Write `docs/spec.md`: scope, out of scope, users and roles, main flows, data handled, integrations, non-functional needs.

Decide the tier with the user:
- **sensitive** if it stores, shows, sends or logs personal or regulated data (health, identity, finance, HR, children), or integrates with systems that hold it (e.g. HIS, HL7/FHIR, payment, payroll).
- **internal** if it is company or client code without such data.
- **open** if it is the owner's own or open-source code with no company or personal data.

When unsure, choose the higher tier. For sensitive, the Security lens lists data flows and required controls (access control, audit logging, encryption, retention) in the spec before approval.

## Step 3 — Architecture (gate: Tech Lead / CTO)

Write `docs/adr/0001-architecture.md` from `templates/adr.md`. Compare at most 3 options. Prefer the company's existing stacks unless there is a stated reason not to; say what each option costs to operate.

## Step 4 — Scaffold

Only after Steps 1–3 are approved. Uno does not write code here either: Mu (`developer`) writes items 1–2, Iota (`devops-engineer`) item 3, Delta (`test-writer`, `mode: review`) and Lambda (`architect`, `mode: review`) review the scaffold; Uno writes items 4–7. Create, in this order:
1. Project skeleton for the chosen stack, with a single passing smoke test.
2. Lint, format and type check configured and passing.
3. CI that runs lint + tests on every PR and blocks merge on failure.
4. `CLAUDE.md` from `templates/CLAUDE.md`.
5. `.uno/zones` from the onboard skill's `templates/zones`, plus project-specific forbidden paths. The owner reviews it.
6. `.claude/settings.local.json` with the `permissions` block of `templates/settings.internal.json` (open and internal) or `templates/settings.sensitive.json`. UNO files (`.uno/`, `CLAUDE.md`, `.claude/settings.local.json`) go to `.git/info/exclude` when the repo is shared; commit them only if the owner says the repo is personal.
7. `.uno/onboarding.md` with all phases marked "n/a — born on UNO" except baseline.

Open the scaffold (code, tests, CI, zones-free) as one PR: `chore: scaffold <name>`. A human merges it. UNO's own files are not part of it.

## Step 5 — Register

Add the project to `registry/projects.yaml` in the UNO repo: tier, owner, `autonomy: L1`, plugins, release, `onboarding: done`.

## Step 6 — Build

Normal UNO loop: `plan.md` per feature → small PRs → review. A new project may be promoted to L2 after its first feature ships with tests and no escalations. Sensitive tier never goes above L2.

## Rules

- No feature code before CI and the smoke test pass.
- No real personal data in fixtures, seeds or tests; generate synthetic data.
- Every dependency added is named in the PR description with why.
