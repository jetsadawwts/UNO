# <project-name>

<One line: what this system does and for whom.> Tier: <standard | clinical>. Owner: <name>.

## Commands
- Install: `<cmd>`
- Run: `<cmd>`
- Test: `<cmd>`
- Lint/format: `<cmd>`

## Structure
- `<path>` — <what lives there>

## Conventions
- <naming, error handling, logging, patterns to follow>

## Zones
See `.uno/zones`. Forbidden paths are human-only; review paths need Security sign-off.

## Ask before
- adding a dependency
- changing a database schema or migration
- anything touching auth, patient data, or external integrations

## Never
- use real patient data in code, tests, fixtures or logs
- commit secrets
- push to the default branch, or commit on it

## Docs
- Intent: `docs/intent.md` · Spec: `docs/spec.md` · ADRs: `docs/adr/`
