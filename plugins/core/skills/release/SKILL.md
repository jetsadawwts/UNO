---
name: release
description: Prepare a release for an onboarded UNO project — Zeta (release-manager) collects merged PRs since the last tag and writes release notes, deploy steps and a rollback plan to .uno/releases/. The owner tags and deploys. Use when the user says "prepare a release", "what's in the next release", "/core:release".
---

# Release: merged work → release file → owner deploys

UNO never tags, pushes tags or deploys. This skill only writes `.uno/releases/<version>.md`.

1. `git fetch origin --tags`, then call Zeta (`release-manager`) with the default branch and the profile's deploy notes (`.uno/project-profile.md` §3).
2. Show Zeta's summary: version, changes, flags, what a redeploy would not undo.
3. **Stop for the owner**: they review the file, then tag and deploy themselves. If any flag needs a decision (a merge without Epsilon PASS, an irreversible step), present it as a joint decision with options.
4. After the owner reports the deploy, add `Deployed: <date, where>` to the release file.

Template: `templates/release.md`.
