# First kit-managed release record (j.3)

**Status:** preflight failed — remediation required  
**Target version:** `0.6.0`  
**Release PR:** [#149](https://github.com/randlee/wyvern/pull/149) (`release/v0.6.0` → `main`)

See [j3-rc-runbook.md](j3-rc-runbook.md) for dispatch commands after #148 merges.

## Pre-cut gates

| Gate | Status | Evidence |
|------|--------|----------|
| Org pin @ `25668ec` | **done** | atm #1069 + wyvern #146 |
| Wyvern pin bumped + sync 0 | **done** | `develop` pin @ `25668ec` |
| CR-001/002 resolved | **done** | upstream-tracking |
| Winget bootstrap submitted | **submitted** | [winget-pkgs #425477](https://github.com/microsoft/winget-pkgs/pull/425477) |
| B4 spot-check @ blessed SHA | **pass** | sync @ `25668ec` |
| RC workflow dispatchable | **done** | PR #148 merged; RC [33140961859](https://github.com/randlee/wyvern/actions/runs/33140961859) success |

## Preflight failure remediation (run 33141018872)

| Check | Failure | Remediation |
|-------|---------|-------------|
| credential-liveness | `WINGET_GITHUB_TOKEN` 401 | Refresh org PAT on `randlee/wyvern` secrets |
| credential-liveness | `Unsupported credential liveness check kind: crates_io` | Kit @ `25668ec` preflight gap — escalate to sc-publish org |
| workspace-tests | `report_review_duplicate_finish_is_409` flake | Increase transient retry on `release/v0.6.0` |
| package-checks | exit 101 | Cascade from workspace-tests |


| Step | Workflow | Run ID | SHA/tag | Result |
|------|----------|--------|---------|--------|
| RC dispatch | `release-candidate.yml` | [33140961859](https://github.com/randlee/wyvern/actions/runs/33140961859) | `release-candidate-v0.6.0` | **success** |
| Readiness preflight | `release-preflight.yml` | [33141018872](https://github.com/randlee/wyvern/actions/runs/33141018872) | `release/v0.6.0` | **failed** — see remediation |
| Release branch merge | PR → `main` | [#149](https://github.com/randlee/wyvern/pull/149) | `release/v0.6.0` | open |
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
