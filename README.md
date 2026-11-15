# UNO

Personal Claude Code plugin marketplace: an AI engineering team that works under rules.
Owner: jetsadawwts. Not shared with any team or company.

UNO decides **what** may change and **how safe** it must be — registry, onboarding, zones,
autonomy levels, backlog and guard hooks. How one task gets coded is delegated to
[Superpowers](https://github.com/obra/superpowers) when it is installed.

## Contents

| Path | What |
|---|---|
| `plugins/core/skills/` | `portfolio-scan`, `onboard`, `work`, `new-project`, `refactor` |
| `plugins/core/agents/` | Gamma (`ui-designer`), Epsilon (`security-reviewer`) |
| `plugins/core/hooks/` | `guard_zones`, `guard_git`, `trace_agents` |
| `registry/projects.example.yaml` | shape of the local project registry |
| `ONBOARDING.md` | playbook: flows, loops, install, update, rollback, limits |
| `CHANGELOG.md` | what changed in each version |

## Install

```
/plugin marketplace add ~/UNO/uno
/plugin install core@uno
```

Then copy `registry/projects.example.yaml` to `registry/projects.yaml` (ignored by git).

## Versioning

Each change bumps `plugins/core/.claude-plugin/plugin.json`, adds a `CHANGELOG.md` entry,
and is tagged `vX.Y.Z`. Roll back with `git checkout vX.Y.Z`, then update and reinstall
`core@uno` in Claude Code (see `ONBOARDING.md`).

## Data rule

This repo holds only generic skills, agents, hooks and templates. No company names, project
details, patient data or project `.uno/` files are ever committed here.
