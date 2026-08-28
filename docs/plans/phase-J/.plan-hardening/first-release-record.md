# First kit-managed release record (j.3)

**Status:** blocked on [PR #148](https://github.com/randlee/wyvern/pull/148) review  
**Target version:** `0.6.0`  
**Branch:** `develop` (phase-J merged via #147) → `main`

See [j3-rc-runbook.md](j3-rc-runbook.md) for dispatch commands after #148 merges.

## Pre-cut gates

| Gate | Status | Evidence |
|------|--------|----------|
| Org pin @ `25668ec` | **done** | atm #1069 + wyvern #146 |
| Wyvern pin bumped + sync 0 | **done** | `develop` pin @ `25668ec` |
| CR-001/002 resolved | **done** | upstream-tracking |
| Winget bootstrap submitted | **submitted** | [winget-pkgs #425477](https://github.com/microsoft/winget-pkgs/pull/425477) |
| B4 spot-check @ blessed SHA | **pass** | sync @ `25668ec` |
| RC workflow dispatchable | **blocked** | PR #148 CI green; review required |

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
