# Live team activity card with task id (v0.5.20) — design, awaiting owner approval

## Intent
The owner wants the card to say what each session is doing now, e.g. `T38 · Uno: running Bash · 20 s`.
Replaces the earlier task-id-only design.

## Goals / non-goals
- Goal: per open session show state, tool or agent, task id and time in that state; page follows within about a second.
- Never store commands, paths, prompts, messages, branch names or file content.
- Not covered: progress or percent, task title, anything hooks do not report.

## Approaches
- Task id: A, from the branch name (chosen by the owner). B (tasks.md status) can name the wrong task; C (/core:work records it) depends on the agent.
- Activity: (1) keep the one-per-minute heartbeat: cheap, not live. (2) write on every state change from hooks (chosen by the owner).
  (3) read Claude's transcripts: reads conversation content and watches outside ~/UNO/data; rejected.

## States
| Hook event | State shown |
|---|---|
| UserPromptSubmit, PostToolUse, PostToolUseFailure | working between tools |
| PreToolUse | running <tool> |
| PreToolUse of AskUserQuestion, PermissionRequest, Notification permission_prompt / elicitation_dialog | waiting for the owner |
| Stop, Notification idle_prompt | finished the turn |
| StopFailure | stopped by an error |
| SessionEnd | session removed |

## Design
- trace_heartbeat.py handles these events, writes the same per-session file atomically on every change (no throttle),
  prints nothing and always exits 0, so it never decides a permission or blocks. Runs synchronously to keep order.
- Task id: `git rev-parse --abbrev-ref HEAD` (2 s timeout) on UserPromptSubmit and PreToolUse, guard_plan's regex;
  other events keep the previous value; failure or no id gives "".
- uno-report: one card line per open session. Feed and counts unchanged. Session open/closed rule unchanged (30 minutes).

## Step 1: check which events Cowork sends (before building)
Only PreToolUse is verified there. Temporary hook in the project's .claude/settings.local.json (backed up, not committed)
logging only time and event name, for: a prompt, a tool, a permission prompt, a question, end of turn, interrupt.
Then remove it. If events are missing, return to the owner with the reduced design before building.

## Limits
- The docs describe no event for a user interrupt, so a state can stick until the next event.
- A tool blocked or denied may leave "running" or "waiting" until the next event.
- PermissionRequest may fire when no dialog reaches the owner (to check in step 1).
- Parallel tools or subagents share one session line: the last event wins.
- Branchless tasks show no id; a finished task's branch keeps its id.
- About two Python starts per tool call.

## Files
trace_heartbeat.py, hooks.json, tests/test_hooks.py, tests/test_hooks_json.py if needed; uno-report domain,
adapters-fs, adapters-web, golden page; plugin.json (0.5.20), CHANGELOG, ARCHITECTURE, SECURITY, ONBOARDING;
manual.md and manual.th.md (backed up first).

## Verification
Tests first, seen failing: each event to state, nothing printed, no sensitive fields stored, task id cases,
several sessions, counts unchanged. Full hook and Rust suites, live --serve check, then a check in Cowork.

## Rollback
Before commit: restore files and manual copies. After push: git revert, push, update the plugin.
No commit or push without the owner asking.
