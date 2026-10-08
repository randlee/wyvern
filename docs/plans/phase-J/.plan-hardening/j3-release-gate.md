# j.3 release gate (Phase J)

**Status:** RC dispatch blocked — register workflow on `main` ([PR #148](https://github.com/randlee/wyvern/pull/148))  
**Updated:** 2026-08-28

j.3 cannot start until every gate below is **green**. Do not merge
`integrate/phase-J` → `develop` for release until then.

## Org kit gates

| Gate | Owner | Status | Evidence |
|------|-------|--------|----------|
| sc-publish PR #63 merged → `develop` | atm/sc-publish | **done** | [PR #63](https://github.com/randlee/sc-publish/pull/63) merged; `develop` @ `5d7c749` |
| sc-publish `main` fast-forwarded | atm/sc-publish | **done** | [PR #64](https://github.com/randlee/sc-publish/pull/64) → `main` @ `25668ec` |
| atm-core AT-style qualification + publish from `develop` | atm-core | **done** | #1069 merged; RC [33139520613](https://github.com/randlee/atm-core/actions/runs/33139520613) success |
| Org pin published | atm/sc-publish | **done** | `25668ec` on sc-publish `main`; atm + wyvern pinned |
| Wyvern re-pin @ `25668ec` | wyvern | **done** | [PR #146](https://github.com/randlee/wyvern/pull/146) merged |
| wyvern sync dry-run @ blessed SHA | wyvern | **done** | exit 0 @ `25668ec` |
| `develop` has phase-J @ 0.6.0 | wyvern | **done** | [PR #147](https://github.com/randlee/wyvern/pull/147) merged |
| RC workflow dispatchable | wyvern | **blocked** | [PR #148](https://github.com/randlee/wyvern/pull/148) needs merge (main default branch) |

## Wyvern preflight gates (j.2 carryover)

| Gate | Status | Evidence |
|------|--------|----------|
| Secrets present (`WINGET_*`, `SCOOP_*`, …) | **done** | `gh secret list` |
| `randlee/scoop-bucket` cloneable | **done** | Public repo |
| `randlee.wyvern` winget bootstrap | **submitted** | [winget-pkgs #425477](https://github.com/microsoft/winget-pkgs/pull/425477) |
| PR #145 consumer pin merged to `integrate/phase-J` | **done** | [PR #145](https://github.com/randlee/wyvern/pull/145) @ `a042e3f` |

## j.3 execution (after gates)

1. Bump workspace to target semver (e.g. `0.6.0`)
2. Merge `integrate/phase-J` → `develop` (kit workflows on develop)
3. Dispatch `release-candidate.yml` on `develop`
4. `release/vX.Y.Z` → `main` → preflight → production `release.yml`
5. Post-release legs + record in `first-release-record.md`

## References

- [j3-first-kit-release.md](../j3-first-kit-release.md)
- [upstream-tracking.md](upstream-tracking.md)
- [sc-publish-extension-requests.md](sc-publish-extension-requests.md)
