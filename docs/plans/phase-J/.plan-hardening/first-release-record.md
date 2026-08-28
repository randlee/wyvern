# First kit-managed release record (j.3)

**Status:** pending  
**Target version:** `0.6.0` (TBD at cut time)  
**Branch:** `integrate/phase-J` → `develop` → `main`

Fill this during j.3 execution. j.4 go/no-go reads the final row.

## Pre-cut gates

| Gate | Status | Evidence |
|------|--------|----------|
| Org pin @ `25668ec` | pending | atm qualification receipt |
| Wyvern pin bumped + sync 0 | pending | `release/sc-publish-pin.toml` |
| CR-001/002 resolved | pending | upstream-tracking |
| Winget bootstrap submitted | **submitted** | [winget-pkgs #425477](https://github.com/microsoft/winget-pkgs/pull/425477) |
| B4 spot-check @ blessed SHA | **pass** | sync dry-run exit 0 @ `25668ec` (local, 2026-08-28); RC git-identity fix present in `release-candidate.yml` |

## State machine

| Step | Workflow | Run ID | SHA/tag | Result |
|------|----------|--------|---------|--------|
| RC dispatch | `release-candidate.yml` | | `release-candidate-vX.Y.Z` | |
| Release branch merge | PR → `main` | | `release/vX.Y.Z` | |
| Preflight | `release-preflight.yml` | | exact `main` SHA | |
| Production | `release.yml` | | `vX.Y.Z` | |

## Channel outcomes

| Channel | Workflow | Run ID | Result | Notes |
|---------|----------|--------|--------|-------|
| GitHub Release | `release.yml` | | | |
| crates.io | `crates-publish.yml` | | | |
| Homebrew | `homebrew-publish.yml` | | | |
| Scoop | `scoop-publish.yml` | | | |
| Winget | `winget-publish.yml` | | | |
| PyPI | — | — | **N/A** | omitted from `install.json` |

## Post-release verification

```bash
gh release view vX.Y.Z --json assets
python3 .github/scripts/release_artifacts.py channel-dispatch-plan \
  --manifest release/publish-artifacts.toml --tag vX.Y.Z \
  | jq -e '([.channels[]?.name] // []) | index("pypi") | not'
```

## j.4 go/no-go

| Decision | Rationale |
|----------|-----------|
| **pending** | Complete after all channels recorded |
