# Subagent brief

Every subagent UNO starts gets these four parts. A brief missing any of them leads to duplicated work, wrong output, or a subagent wandering outside its task.

```
OBJECTIVE
One or two sentences: what to find or change, and why it matters for task <ID>.

OUTPUT FORMAT
Exactly what to return, e.g. "a list of file:line with one-line reason each",
"a unified diff plus the test command you ran and its result", "PASS/FAIL + findings table".

TOOLS AND SOURCES
Which tools to use and which to avoid, e.g. "read-only: Read, Grep, Glob, git log.
No edits. Do not run the app or touch real data."

BOUNDARIES
Files/folders in scope, what is out of scope, zones to respect, when to stop
(e.g. "stop after 20 files or when you find 3 call sites"), what to do if blocked.
```

Rules
- One subagent per independent piece of work. Do not split work that touches the same files.
- Prefer one agent when the work is sequential. Subagents cost several times more tokens.
- Subagents never merge, push to the default branch, or edit `.uno/zones`.
