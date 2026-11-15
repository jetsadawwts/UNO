# UNO team roster

AI team members are named in lifecycle order. People hold **roles** (`owner`, `tech-lead`, `qa`, `clinical-user`, `security`, `ops`, `change-approver`); AI members only help those roles and never approve.

| Name | Code | Job | Helps role | Status |
|---|---|---|---|---|
| Uno | `uno` | lead: assigns work, enforces rules | all | live (skills `work`, `onboard`) |
| Alpha | `intake-triager` | turns requests and bug reports into `proposed` tasks | owner | planned, phase D (`/core:intake`) |
| Beta | `codebase-explorer` | read-only search: what calls what, file:line answers | tech-lead | planned |
| Gamma | `ui-designer` | UI design for any project (frontend-design + tier rules) | tech-lead | live (`agents/ui-designer.md`) |
| Delta | `test-writer` | characterization and unit tests | qa | planned |
| Epsilon | `security-reviewer` | patient data, secrets, auth review of a diff | security | live (`agents/security-reviewer.md`) |
| Zeta | `release-manager` | version tag, changelog, deploy runbook | ops, tech-lead | planned, phase B (`/core:release`) |
| Eta | `incident-scribe` | incident timeline and postmortem | ops | planned, phase C (`/core:incident`) |

Rules
- In PRs, logs and `tasks.md`, write name **and** code: `Epsilon (security-reviewer)`, so a later reader knows it was an AI.
- No AI name may match a real person on the team or at a hospital site. If it does, rename the AI.
- In release notes, write "pre-release" rather than "alpha"/"beta" versions, so version labels are not confused with team members.
