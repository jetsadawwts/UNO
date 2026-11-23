---
name: incident
description: Record a production problem the owner reports in an onboarded UNO project — Eta (incident-scribe) builds a timeline, links it to recent releases and code, lists likely causes with confidence, writes .uno/incidents/, and proposes a fix task and a regression test. Use when the user says "users report…", "it broke after deploy", "/core:incident".
---

# Incident: report → record → proposed fix (closes the loop)

UNO does not touch servers, databases or production logs. It works from the owner's report and the repo.

1. Get from the owner, in one message if missing: symptom, start time, where (sites/users/screens), how often, workaround, anything already tried.
2. Ask Beta (`codebase-explorer`) where the symptom points in the code, if that is not obvious.
3. Call Eta (`incident-scribe`) with the report, Beta's answer and the latest `.uno/releases/` file.
4. **Stop for the owner**: show likely causes with confidence, the rollback option and the proposed tasks. The owner confirms the cause (or says what to check on the server), decides rollback vs fix forward, and sets the fix task to `ready`.
5. The fix goes through `/core:work` like any task; link the incident id in its plan.

Template: `templates/incident.md`.
