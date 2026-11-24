---
name: intake
description: Turn a request, idea or bug report into a spec and proposed tasks for an onboarded UNO project, with Beta (codebase-explorer) and Alpha (intake-triager). Use when the user says "new request", "bug report", "users want…", "/core:intake", or describes work that is not yet in .uno/tasks.md.
---

# Intake: request → spec → proposed tasks

Nothing in this skill changes project code. Output lives in `.uno/specs/` and `.uno/tasks.md`.

1. **Preconditions**: `.uno/project-profile.md` and `.uno/tasks.md` exist; otherwise tell the user to run `/core:onboard`.
2. **Locate** (if the request points at existing behaviour): ask Beta (`codebase-explorer`) one concrete question, e.g. "where is the IPD report built and exported?". Pass its answer on.
3. **Architectural?** If the request needs a new service or module, spans several parts of the Stack, changes language or framework, or adds a major dependency, ask Lambda (`architect`) for an ADR first and give Alpha the ADR's task breakdown.
4. **Spec**: call Alpha (`intake-triager`) with the request in the owner's words, the profile, the task list and Beta's answer.
5. **Questions**: if Alpha returns questions, ask the owner (max 3, in one message) and send the answers back to Alpha to finish the spec.
6. **Stop for approval**: show the spec's goal, acceptance criteria, out-of-scope, sensitive-data flag and proposed tasks. The owner approves by changing a task's status from `proposed` to `ready` (or says so, and you change it). Do not start `/core:work` yourself.

Templates: `templates/spec.md`.
