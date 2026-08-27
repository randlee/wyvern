---
id: j.2
title: Upstream blockers, secrets, and consumer docs
status: planning
branch: feature/phase-J-j2-upstream-docs
worktree: ../wyvern-worktrees/feature/phase-J-j2-upstream-docs
target: integrate/phase-J
depends_on: j.1
---

# Sprint j.2 — Upstream blockers, secrets, and consumer docs

## Goal

Close or **explicitly waive** Grok blockers before j.3 rehearsal. Update Wyvern
consumer docs and secret inventory for sc-publish (especially winget). Fix
`install.json` manifest issues without forking kit YAML.

## Hard dependencies

- j.1 merged to `integrate/phase-J`

## Deliverables

| Path | Purpose |
|------|---------|
| `release/install.json` | Homebrew UI path fix (`homebrew_destination_components`) |
| `docs/RELEASE_SECRETS.md` | Kit secret names incl. `WINGET_GITHUB_TOKEN` |
| `docs/WINGET_SETUP.md` | sc-publish winget leg + bootstrap + token model |
| `README.md` | Kit archive naming (`wyvern_<ver>_<target>.*`, `bin/` layout) |
| `docs/plans/phase-J/.plan-hardening/upstream-tracking.md` | Links to sc-publish PRs/issues for CR-001/CR-002 |
| GitHub repo secrets (owner) | `WINGET_GITHUB_TOKEN` provisioned or documented gap |

## Paths to delete

| Path | Reason |
|------|--------|
| `scripts/validate_release.py` | Or rewrite to kit CLI only; must not reference removed subcommands |

## Upstream outcomes (choose one per blocker — document in closeout)

| Blocker | Resolved | Waived |
|---------|----------|--------|
| CR-001 Linux webview deps | sc-publish PR merged; kit jobs install apt packages | j.3/j.4 blocked; emergency tag-push retained on `develop` until merged |
| CR-002 Homebrew renderer | sc-publish PR: bootstrap sc-compose on runner | `channels.homebrew` removed from `install.json`; tap manual until upstream |

## Acceptance criteria

1. `release/install.json` sets `homebrew_destination_components` to `["share","wyvern","ui"]` for bundled UI; re-sync dry-run exit **0**.
2. `docs/RELEASE_SECRETS.md` documents `CARGO_REGISTRY_TOKEN`, `HOMEBREW_TAP_TOKEN`, **`WINGET_GITHUB_TOKEN`** with purpose (no secret values).
3. `docs/WINGET_SETUP.md` documents: `winget-publish.yml` dispatch, `WINGET_GITHUB_TOKEN`, asset pattern `wyvern_*_x86_64-pc-windows-msvc.zip`, one-time bootstrap, Microsoft review lag.
4. `README.md` artifact table matches kit archive names and `bin/` layout.
5. `scripts/validate_release.py` deleted **or** rewritten to use `.github/scripts/release_artifacts.py` subcommands that exist in kit @ synced version.
6. `upstream-tracking.md` records CR-001/CR-002 disposition (resolved PR link **or** signed waiver text).
7. `gh secret list` shows `WINGET_GITHUB_TOKEN` **or** sprint closeout documents owner provisioning as gate before j.3.

## Non-closure (explicit)

- **j.2 does not** execute release rehearsal — **j.3**.
- **j.2 does not** submit winget bootstrap PR to `microsoft/winget-pkgs` unless owner chooses to do so as prep; j.3 AC covers submission attempt.
- If CR-001 **and** CR-002 are both waived, j.3 must not merge until waiver is signed in closeout.

## Required validation

```bash
./scripts/sync-sc-publish.sh
python3 .github/scripts/release_artifacts.py validate-manifest \
  --manifest release/publish-artifacts.toml --workspace-toml Cargo.toml
rg 'WINGET_GITHUB_TOKEN' docs/RELEASE_SECRETS.md docs/WINGET_SETUP.md
rg 'wyvern_.*_x86_64-pc-windows-msvc' docs/WINGET_SETUP.md README.md
```
