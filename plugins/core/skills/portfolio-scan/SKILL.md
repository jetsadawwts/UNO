---
name: portfolio-scan
description: Scan a folder that holds many existing repos and produce a triage inventory (activity, size, tests, CI, personal-data signals, suggested tier and onboarding wave). Use when the user wants to bring several legacy projects under UNO, asks which projects to onboard first, or says "scan all projects".
---

# Portfolio scan

Read-only triage of many repos so the CTO can decide the onboarding order. This skill never edits a repo.

## Steps

1. Ask for the parent folder if the user has not given it. Confirm it holds one repo per subfolder.
2. Run the scanner:
   ```bash
   python3 "${CLAUDE_PLUGIN_ROOT}/skills/portfolio-scan/scripts/scan_portfolio.py" <parent> \
     --out .uno/inventory.md --json .uno/projects.json
   ```
3. Read `.uno/inventory.md` and report back in UNO's short format:
   - count per wave
   - the 3 projects to onboard first and why
   - every project with personal-data signals (these are hints from keywords, not proof)
   - flags that block onboarding (`hold`, no tests on a sensitive project, bus factor 1)
4. Draft entries for `registry/projects.yaml` in the UNO repo using `projects.example.yaml` as the shape. Leave `tier`, `wave` and `owner` marked `# confirm` — a human sets them.
5. Stop. Do not start onboarding any project until the CTO confirms the registry.

## Wave rules (suggestions only)

| Wave | Rule | Reason |
|---|---|---|
| 1 | active in last 90 days, no personal-data signals | tune UNO where mistakes are cheap |
| 2 | active, has personal-data signals | apply the process after wave 1 lessons |
| 3 | no commits in 90 days | onboard only when work arrives |
| hold | not a git repo | put under version control first |

## Rules

- Keyword hits for personal data can be false positives or misses. Say so in the report.
- Never copy file contents that look like real personal data into the report; list file paths only.
- Large repos (>100k LOC): recommend splitting discovery by module in the onboard skill.
