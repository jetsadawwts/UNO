# UNO team roster

AI team members are named in lifecycle order. People hold **roles** (`owner`, `tech-lead`, `qa`, `domain-user`, `security`, `ops`, `change-approver`); AI members only help those roles and never approve.

| Name | Code | Job | Helps role | Edits | Status |
|---|---|---|---|---|---|
| Uno | `uno` | lead: plans, enforces rules, implements, opens PRs | all | code, inside an approved plan | live (`work`, `onboard`, `new-project`, `refactor`, `portfolio-scan`) |
| Alpha | `intake-triager` | request or bug → spec + proposed tasks | owner | `.uno/` only | live (`/core:intake`) |
| Beta | `codebase-explorer` | read-only search, file:line answers | tech-lead | nothing | live (called by skills) |
| Gamma | `ui-designer` | UI design (`design` mode, in the plan) and build (`build` mode) | tech-lead | design: nothing · build: UI inside the plan | live |
| Delta | `test-writer` | characterization tests, failing tests for new behaviour | qa | test files only | live (called by `work`) |
| Epsilon | `security-reviewer` | personal/sensitive data, secrets, auth, deps review of a diff | security | nothing | live (before every PR) |
| Zeta | `release-manager` | release notes, deploy steps, rollback plan | ops, tech-lead | `.uno/releases/` only | live (`/core:release`) |
| Eta | `incident-scribe` | incident record, likely causes, fix + test tasks | ops | `.uno/` only | live (`/core:incident`) |
| Theta | `doc-writer` | README, user guides, deploy/operations and developer docs, checked against the code | owner, ops | documentation files only | live (called by `work`, suggested by `release`) |

Reserved names for new members: Iota, Kappa, Lambda.
Subagents cannot start other subagents: a skill (run by Uno) calls Beta first and passes its answer on.

Rules
- In PRs, logs and `tasks.md`, write name **and** code: `Epsilon (security-reviewer)`, so a later reader knows it was an AI.
- No AI name may match a real person in any project you work on. If it does, rename the AI.
- In release notes, write "pre-release" rather than "alpha"/"beta" versions, so version labels are not confused with team members.
