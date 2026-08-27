---
id: j.4
title: Production cutover
status: planning
branch: feature/phase-J-j4-cutover
worktree: ../wyvern-worktrees/feature/phase-J-j4-cutover
target: integrate/phase-J
depends_on: j.3
---

# Sprint j.4 — Production cutover

## Goal

Merge `integrate/phase-J` → `develop`. Cut first **production** release fully on
sc-publish. Retire tag-push release workflow.

## Hard dependencies

- j.3 `rehearsal-record.md` go/no-go = **go**
- No open CR-001/CR-002 waivers unless explicitly re-signed

## Deliverables

| Path | Purpose |
|------|---------|
| `.github/workflows/release.yml` (kit) | Sole release entry; tag-push trigger removed |
| `release/release-notes.md` | Breaking archive URL change called out |
| `docs/plans/phase-J/.plan-hardening/cutover-record.md` | Production run IDs, channel verification |

## Paths to delete

| Path | Reason |
|------|--------|
| Tag-push `on.push.tags: v*` trigger | Replaced by kit `workflow_dispatch` on `main` |

Implementation: remove legacy trigger from any remaining bespoke release workflow file; kit `release.yml` must be the only root release workflow.

## Breaking change (normative)

| Old (v0.5.0) | New (kit) |
|--------------|-----------|
| `wyvern-macos-aarch64.tar.gz` | `wyvern_X.Y.Z_aarch64-apple-darwin.tar.gz` |
| `wyvern-windows.zip` | `wyvern_X.Y.Z_x86_64-pc-windows-msvc.zip` |
| Binaries at archive root | `bin/wyvern`, `bin/wyvern-viewer` |

## Acceptance criteria

1. `integrate/phase-J` merged to `develop` after phase-end QA PASS.
2. No workflow on `develop` triggers release on tag push alone.
3. Production `vX.Y.Z` shipped via kit state machine (same steps as j.3).
4. Release notes document archive rename and winget review lag.
5. Verified: crates.io, GitHub Release (four targets), Homebrew tap bump (unless j.2 Homebrew waiver still active — then **fail**), winget submission success.
6. `main` → `develop` back-merge PR opened post-release (publisher policy).

## Non-closure (explicit)

- **j.4 does not** merge back-merge PR — publisher/owner after CI green.

## Required validation

```bash
! gh workflow view release.yml --yaml | rg 'push:\s*$' -A1 | rg 'tags:'
gh release view vX.Y.Z --json assets
brew info wyvern  # tap version matches X.Y.Z when Homebrew channel active
```
