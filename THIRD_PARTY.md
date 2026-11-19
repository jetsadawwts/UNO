# Third-party plugins and skills

Every plugin or skill not written in this repo is listed here before it is used with UNO.
Agents may search for skills; only the owner installs them.

## Review checklist (before install)
- Read `SKILL.md` and every script it ships.
- Hooks: what events, what commands?
- Network: does it call anything outside the machine?
- Permissions and tools it asks for.
- Where it stores data (memory plugins especially).
- License, author, last update, open issues.
- Which tiers it may be used with (`open` / `internal` / `sensitive`).

## Installed

| Name | Source | Version | Installed | Tiers | Notes |
|---|---|---|---|---|---|
| Superpowers | obra/superpowers-marketplace | _fill in from /plugin_ | _date_ | all | used by `/core:work` full mode |

## Planned / under evaluation

- **Ponytail** (`DietrichGebert/ponytail`, MIT) — minimal-change skill with always-on hooks and `/ponytail-review`. Status: benchmark arm D (UNO + Ponytail lite) before adoption. Checks before install: hook order with UNO and Superpowers; it must not remove validation, logging or masking in `sensitive` projects; do not use `band import` until it is clear where session data is stored. Install at user scope only after the benchmark; pin the version here.

| Name | Source | Purpose | Status |
|---|---|---|---|
| frontend-design | anthropics/claude-plugins-official | Gamma design direction | install when a UI task arrives |
| Impeccable | pbakaus/impeccable | richer design skill for Gamma (audit, critique, polish) | evaluate on a UI task vs frontend-design |
| claude-mem | thedotmack/claude-mem | automatic cross-session memory | optional per project; stores summaries of everything read |
| task-observer | rebelytics/one-skill-to-rule-them-all | learn from corrections | idea used for `/core:retro`; not installed |
| find-skills | vercel-labs/skills | search the skills registry | search only; installs go through this file |
