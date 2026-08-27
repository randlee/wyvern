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

Resolve CR-001 and CR-002 **before** j.3. Update consumer docs; provision
`WINGET_GITHUB_TOKEN` and `SCOOP_BUCKET_TOKEN`; publish `docs/SCOOP_SETUP.md`.
Waivers **block** j.3/j.4 (see upstream-tracking).

## Hard dependencies

- j.1 merged to `integrate/phase-J`

## Deliverables

| Path | Purpose |
|------|---------|
| `release/install.json` | Homebrew UI path; `renderer_archive_path` names kit renderer path when CR-002 resolved |
| `docs/RELEASE_SECRETS.md` | Kit secrets incl. `WINGET_GITHUB_TOKEN`, `SCOOP_BUCKET_TOKEN` + PAT scope notes |
| `docs/SCOOP_SETUP.md` | Scoop bucket repo, manifest leg, token model |
| `docs/WINGET_SETUP.md` | sc-publish winget leg, bootstrap, token model |
| `README.md` | Kit archive naming |
| `docs/plans/phase-J/publish-architecture-decision.md` | ADR-linked decision record |
| `docs/plans/phase-J/.plan-hardening/upstream-tracking.md` | CR-001/CR-002 disposition table |

## Paths to delete

| Path | Reason |
|------|--------|
| `scripts/validate_release.py` | Superseded by kit preflight + manifest CLI |

## Acceptance criteria

1. `release/install.json` sets `homebrew_destination_components` to `["share","wyvern","ui"]`; re-sync dry-run exit **0**.
2. CR-001 **resolved**: sc-publish PR merged; Linux webview apt packages run in kit release/preflight/crates jobs — link in upstream-tracking.
3. CR-002 **resolved**: sc-publish PR merged; **both** `homebrew-publish.yml` and
   `scoop-publish.yml` bootstrap sc-compose on the runner (not product binary as
   renderer); `release/install.json` keeps a **non-empty** `renderer_archive_path`
   naming the kit renderer archive path per upstream contract (must not remove while
   Scoop is declared).
4. `docs/RELEASE_SECRETS.md` documents `CARGO_REGISTRY_TOKEN`, `HOMEBREW_TAP_TOKEN`, `WINGET_GITHUB_TOKEN` (classic or fine-grained PAT; minimum: fork `microsoft/winget-pkgs`, open PRs), and `SCOOP_BUCKET_TOKEN` (push to `randlee/scoop-bucket`).
5. `docs/WINGET_SETUP.md` matches kit: post-release dispatch, token, asset pattern, bootstrap requirement, review lag.
6. `docs/SCOOP_SETUP.md` documents: public cloneable `randlee/scoop-bucket`;
   `bucket/wyvern.json` layout; post-release `scoop-publish.yml` dispatch;
   **`SCOOP_BUCKET_TOKEN`** (PAT with `contents:write` on that repo — **not**
   repository `GITHUB_TOKEN`); Windows asset name
   `wyvern_<version>_x86_64-pc-windows-msvc.zip`; first-run may create manifest
   via workflow (empty bucket acceptable if doc states workflow seeds manifest).
7. `README.md` artifact table uses `wyvern_<version>_<target>.*` and `bin/` layout.
8. `gh secret list` includes **`WINGET_GITHUB_TOKEN`** and **`SCOOP_BUCKET_TOKEN`** (j.2 **cannot** close without them).
9. If `randlee.wyvern` absent from `winget-pkgs`, owner completes one-time bootstrap **before** j.3 (document completion in closeout).
10. Scoop bootstrap **closed before j.3**: `randlee/scoop-bucket` exists and is
    cloneable; `SCOOP_BUCKET_TOKEN` provisioned (j.2 AC #8); closeout records
    repo URL + successful authenticated push probe (or first manifest commit URL);
    j.2 AC #6 checklist complete.
11. upstream-tracking shows CR-001 and CR-002 both **`resolved`** (not waived).

## Non-closure (explicit)

- **j.2 does not** run kit release — **j.3**.
- **Waivers block the phase** — do not sign waivers to “unblock” j.3.

## Required validation

```bash
./scripts/sync-sc-publish.sh
python3 .github/scripts/release_artifacts.py validate-manifest \
  --manifest release/publish-artifacts.toml --workspace-toml Cargo.toml
gh secret list | rg 'WINGET_GITHUB_TOKEN|SCOOP_BUCKET_TOKEN'
test ! -f scripts/validate_release.py
rg 'WINGET_GITHUB_TOKEN' docs/RELEASE_SECRETS.md docs/WINGET_SETUP.md
rg 'SCOOP_BUCKET_TOKEN' docs/RELEASE_SECRETS.md docs/SCOOP_SETUP.md
```
