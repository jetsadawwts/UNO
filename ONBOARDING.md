# UNO project playbook (v0.4.0)

UNO is **JWT's personal tool**, not shared with the team. Its files live on this machine only: the plugin in `~/UNO/uno`, and per project `.uno/`, `CLAUDE.md` and `.claude/settings.local.json`, all kept out of shared repos through `.git/info/exclude`. Project data lives in `~/UNO/data/<project>/` (linked into each project as `.uno`): back it up locally, never push it. Review is `self` by default (you review your own PRs, guided by the `review: careful | quick` label); a repo that requires another approver is set to `second-reviewer`.

UNO is the **org layer**: which project, how risky, what agents may touch, how much autonomy. How a single task is coded is delegated to the [Superpowers](https://github.com/obra/superpowers) skills when they are installed; without them `/core:work` falls back to a built-in Plan → Act → Verify → Reflect loop.

Four ways a project enters or changes under UNO:

| Situation | Skill | First gate |
|---|---|---|
| Many existing projects, where to start? | `/core:portfolio-scan` | CTO confirms registry |
| One existing project | `/core:onboard` | owner approves project profile |
| Brand-new project | `/core:new-project` | CTO approves intent |
| Do a task in an onboarded project | `/core:work <task-id>` | owner marks the task `ready` in `.uno/tasks.md` |
| Restructure code in an onboarded project | `/core:refactor` | Tech Lead approves ADR; characterization tests merged |

The rest of this file covers legacy onboarding. See each skill's `SKILL.md` for the others.

## Flow

```
portfolio-scan  →  registry (human)  →  onboard, per project, wave by wave
                                          0 registry check
                                          1 discovery (read-only)
                                          2 human review gate   ← stop until approved
                                          3 setup PR (CLAUDE.md, zones, settings)
                                          4 baseline
                                          5 first L1 tasks → .uno/tasks.md
                                                       ↓
                                    /core:work T1, T2, …  (one branch + one PR per task)
                                       small mode: plan → change → verify → PR
                                       full mode:  brainstorm → plan → worktree → subagents
                                                   → 2-stage review → verify → PR
                                                       ↓
                                    human reviews PR (security-reviewer verdict for review zones)
```

## Loops

| Loop | Runs | Stop condition | Human decides |
|---|---|---|---|
| Task | each `/core:work` | acceptance check passes, or 3 failed attempts → `escalated` | merge |
| Project | after each merge | backlog empty or baseline target met | which tasks become `ready`; L1 → L2 after 2 sprints without escalations |
| Portfolio | monthly `/core:portfolio-scan` | — | registry: tier, wave, autonomy |
| Org | after each project onboarding | — | fold `Corrections` and `UNO gaps` into the skills; tag a release |

## Files

| Path | What |
|---|---|
| `plugins/core/skills/portfolio-scan/` | scans a folder of repos → `.uno/inventory.md` with suggested tier and wave |
| `plugins/core/skills/onboard/` | per-project onboarding skill + templates |
| `plugins/core/skills/new-project/` | greenfield flow + templates: intent, ADR, CLAUDE.md, settings (internal / sensitive) |
| `plugins/core/skills/work/` | run one task: mode choice, branch rules, Superpowers mapping, `tasks.md` template, subagent brief |
| `plugins/core/skills/refactor/` | behaviour-preserving refactor flow + ADR and plan templates |
| `plugins/core/skills/work/references/team.md` | team roster: Uno + Alpha…Eta, their codes, jobs and status |
| `plugins/core/agents/security-reviewer.md` | **Epsilon** — read-only subagent: patient data, secrets, auth review of a diff |
| `plugins/core/agents/ui-designer.md` | **Gamma** — UI design for any project: frontend-design for direction, design intent match/refresh/new, accessibility baseline, extra rules for `sensitive` tier |
| `plugins/core/hooks/guard_zones.py` | blocks agent edits in forbidden zones, logs every edit to `.uno/audit.log` |
| `plugins/core/hooks/trace_agents.py` | records every skill and subagent call (UNO, Superpowers, Gamma, Epsilon) to `.uno/audit.log`, so you can see afterwards which flow a task followed |
| `plugins/core/hooks/guard_git.py` | blocks commit/merge on the default branch, pushes to it, and plain force push |
| `registry/projects.example.yaml` | shape of the project registry (tier, wave, owner, autonomy) |

## Week-by-week

| When | Do | Who decides |
|---|---|---|
| Week 1 | Run `portfolio-scan`, fill `registry/projects.yaml` | CTO |
| Week 2–3 | Onboard 1–2 wave-1 projects end to end; note what the skill got wrong | Project owners |
| Week 4 | Fix the onboard skill and templates from those lessons; tag a UNO release | Platform owner |
| Week 5+ | Wave 2 (sensitive), one project at a time; Security joins the review gate | CTO + Security |
| Ongoing | Wave 3 only when work arrives on that project | — |

## Usage

```
/core:portfolio-scan ~/work/projects
/core:onboard            # run inside one project
/core:work T3            # run one task from .uno/tasks.md
```

## Tiers (every kind of project)

| Tier | Use for | Extra rules on top of the always-on guards |
|---|---|---|
| `open` | your own or open-source code, no company or personal data | none; L3 allowed; UNO files may be committed if the repo is yours |
| `internal` | company or client code without personal data | UNO files stay local; risk-based review label; up to L3 |
| `sensitive` | personal or regulated data: health, identity, finance, HR, children | Epsilon on every PR; UI tasks count as review zone; always `review: careful`; max L2 |

**Always-on guards, every tier:** secrets cannot be read (`.env`, `*.ini`, `*.pem`, `*.key`, credentials), installs ask first, `guard_zones`, `guard_git`, `trace_agents`, third-party plugins listed in `THIRD_PARTY.md`, and this repo refuses commits that carry company or personal data (`scripts/check_no_company_data.py`).

Project-specific display rules (e.g. how a hospital number or an amount must look) live in that project's `.uno/project-profile.md`, not in the skills.

## Install, update and rollback

**After cloning or `git init`:** run `git config core.hooksPath .githooks` once so the data check runs on every commit.

**Install once:** in Claude Code, `/plugin marketplace add ~/UNO/uno`, then `/plugin install core@uno` (user scope). Superpowers is installed separately from its own marketplace; note its version in `CHANGELOG.md` when you test with it.

**Nothing updates by itself.** Claude Code uses the copy made at install time. A change in `~/UNO/uno` reaches your sessions only when you run `/plugin marketplace update uno` and update `core@uno`. Read `CHANGELOG.md` before you do.

**Versioning:** `~/UNO/uno` is a private git repo. Every change bumps `plugins/core/.claude-plugin/plugin.json`, adds a `CHANGELOG.md` entry, and gets a tag `vX.Y.Z`.

**Rollback:**
```
cd ~/UNO/uno
git checkout vX.Y.Z          # the last good version
# in Claude Code:
/plugin marketplace update uno
/plugin uninstall core@uno
/plugin install core@uno
# restart Claude Code; /plugin should show the old version
git checkout main            # when you want to go forward again
```
Project files (`.uno/`) are not touched by a rollback.

**Third-party plugins (Superpowers, frontend-design):** check in `/plugin` whether their marketplace auto-updates; turn it off if you want to update only on purpose. To roll one back, uninstall and reinstall the version you noted, if its marketplace offers it.

## Known limits

- Patient-data detection is keyword based; treat it as a hint.
- `guard_zones` covers Claude's file edit tools. A shell command can still write files, so also deny risky shell commands in `.claude/settings.local.json` permissions for every project.
- The scanner reads source only up to 512 KB per file and skips vendored folders and top-level folders listed in `.gitignore`.
- `guard_git` reads simple command shapes. Aliases, scripts or `sh -c "..."` can get around it; branch protection on the remote is the real control.
- `ui-designer` preloads Anthropic's `frontend-design` skill via `skills:` and reports `loaded` / `not loaded`. Plugin agents preloading skills has had bugs in some Claude Code versions; check the report line. The skill is used only if it is installed (`/plugin install frontend-design@claude-plugins-official`). Enable it per project.
- Superpowers is a third-party project; its skill names may change. `/core:work` falls back to the built-in loop if a skill is missing.
