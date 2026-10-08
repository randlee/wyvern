# First kit-managed release record (j.3)

**Status:** **go** — production v0.6.0 shipped; winget submission pending Microsoft merge  
**Target version:** `0.6.0`  
**Release PR:** [#149](https://github.com/randlee/wyvern/pull/149) (merged → `main` @ `de82405`)

## Pre-cut gates

| Gate | Status | Evidence |
|------|--------|----------|
| Org pin @ `25668ec` | **done** | atm #1069 + wyvern #146 |
| Wyvern pin bumped + sync 0 | **done** | pin @ `25668ec` |
| CR-001/002 resolved | **done** | upstream-tracking |
| Winget bootstrap submitted | **done** | [winget-pkgs #425477](https://github.com/microsoft/winget-pkgs/pull/425477) (0.5.0 legacy; superseded by #425526) |
| B4 spot-check @ blessed SHA | **pass** | sync @ `25668ec` |
| RC workflow dispatchable | **done** | PR #148; RC [33140961859](https://github.com/randlee/wyvern/actions/runs/33140961859) |

## State machine

| Step | Workflow | Run ID | SHA/tag | Result |
|------|----------|--------|---------|--------|
| RC dispatch | `release-candidate.yml` | [33140961859](https://github.com/randlee/wyvern/actions/runs/33140961859) | `release-candidate-v0.6.0` | **success** |
| Readiness preflight | `release-preflight.yml` | [33142970200](https://github.com/randlee/wyvern/actions/runs/33142970200) | `release/v0.6.0` | **success** |
| Release branch merge | PR → `main` | [#149](https://github.com/randlee/wyvern/pull/149) | `de82405` | **merged** |
| Final preflight | `release-preflight.yml` | [33143330784](https://github.com/randlee/wyvern/actions/runs/33143330784) | `main` | **success** |
| Production | `release.yml` | [33143601484](https://github.com/randlee/wyvern/actions/runs/33143601484) | `v0.6.0` | **success** |

## Channel outcomes

| Channel | Workflow | Run ID | Result | Notes |
|---------|----------|--------|--------|-------|
| GitHub Release | `release.yml` | [33143601484](https://github.com/randlee/wyvern/actions/runs/33143601484) | **success** | Tag `v0.6.0`; kit asset names |
| crates.io | `release.yml` + `crates-publish.yml` | [33143601484](https://github.com/randlee/wyvern/actions/runs/33143601484), [33144064389](https://github.com/randlee/wyvern/actions/runs/33144064389) | **success** | All 5 crates @ 0.6.0 live |
| Homebrew | `homebrew-publish.yml` | [33144060674](https://github.com/randlee/wyvern/actions/runs/33144060674) | **success** | `randlee/homebrew-tap` @ 0.6.0 |
| Scoop | `scoop-publish.yml` | [33144061956](https://github.com/randlee/wyvern/actions/runs/33144061956) | **success** | `randlee/scoop-bucket` @ 0.6.0 |
| Winget | `winget-publish.yml` | [33144063372](https://github.com/randlee/wyvern/actions/runs/33144063372) | **submission** | Automated leg failed (no bootstrap in upstream); manual PR [winget-pkgs #425526](https://github.com/microsoft/winget-pkgs/pull/425526) opened |
| PyPI | — | — | **N/A** | omitted from `install.json` |

## Remediation applied during j.3

| Issue | Fix |
|-------|-----|
| `WINGET_GITHUB_TOKEN` 401 | Refreshed org PAT on `randlee/wyvern` + `randlee/atm-core` |
| sc-lint boundary smoke | `setup-sc-lint` uses `sc-runtime`; boundary TOML struct edges |
| crates_io liveness kind | Removed from `publish-channel-contracts.toml` liveness_checks |
| First-release package check | Preflight packages first publishable crate only |
| Test flake | `report_review_finish` transient retry 8s |

## Post-release verification

```bash
gh release view v0.6.0 --json assets
curl -fsS -A wyvern-check "https://crates.io/api/v1/crates/wyvern-cli/0.6.0"
curl -fsS "https://raw.githubusercontent.com/randlee/scoop-bucket/main/bucket/wyvern.json" | jq -e '.version == "0.6.0"'
python3 .github/scripts/release_artifacts.py channel-dispatch-plan \
  --manifest release/publish-artifacts.toml --tag v0.6.0 \
  | jq -e '([.channels[]?.name] // []) | index("pypi") | not'
```

## j.4 go/no-go

| Decision | Rationale |
|----------|-----------|
| **go** | Production tag + GitHub Release + crates.io + Homebrew + Scoop verified; winget submission opened (#425526); PyPI N/A per manifest |
