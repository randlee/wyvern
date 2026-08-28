# First kit-managed release record (j.3)

**Status:** preflight blocked on `WINGET_GITHUB_TOKEN` refresh  
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

## Preflight remediation log

| Run | Result | Remaining blocker |
|-----|--------|-------------------|
| [33141018872](https://github.com/randlee/wyvern/actions/runs/33141018872) | failed | WINGET 401, crates_io liveness kind, test flake |
| [33141348678](https://github.com/randlee/wyvern/actions/runs/33141348678) | failed | sc-lint smoke, WINGET 401 |
| [33141760349](https://github.com/randlee/wyvern/actions/runs/33141760349) | failed | sc-lint boundary schema |
| [33142179164](https://github.com/randlee/wyvern/actions/runs/33142179164) | failed | **WINGET 401**, wyvern-mcp package check |

**Fixed on `release/v0.6.0` @ `277d75c`+:** sc-lint smoke (`sc-runtime`), boundary TOML, crates_io liveness contract, test flake retry, publish-plan ordering, preflight package smoke (first crate only).

**Operator action required:** refresh `WINGET_GITHUB_TOKEN` on `randlee/wyvern` (401 from `api.github.com/user`). Classic or fine-grained PAT with fork/PR rights to `microsoft/winget-pkgs`.

| Step | Workflow | Run ID | SHA/tag | Result |
|------|----------|--------|---------|--------|
| RC dispatch | `release-candidate.yml` | [33140961859](https://github.com/randlee/wyvern/actions/runs/33140961859) | `release-candidate-v0.6.0` | **success** |
| Readiness preflight | `release-preflight.yml` | [33142179164](https://github.com/randlee/wyvern/actions/runs/33142179164) | `release/v0.6.0` | **failed** — WINGET token |
| Release branch merge | PR → `main` | [#149](https://github.com/randlee/wyvern/pull/149) | `release/v0.6.0` | open |
| Production | `release.yml` | | `v0.6.0` | pending preflight green |

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
