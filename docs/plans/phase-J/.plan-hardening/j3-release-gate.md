# j.3 release gate (Phase J)

**Status:** blocked (org qualification in flight)  
**Updated:** 2026-08-28

j.3 cannot start until every gate below is **green**. Do not merge
`integrate/phase-J` → `develop` for release until then.

## Org kit gates

| Gate | Owner | Status | Evidence |
|------|-------|--------|----------|
| sc-publish PR #63 merged → `develop` | atm/sc-publish | **done** | [PR #63](https://github.com/randlee/sc-publish/pull/63) merged; `develop` @ `5d7c749` |
| sc-publish `main` fast-forwarded | atm/sc-publish | **done** | [PR #64](https://github.com/randlee/sc-publish/pull/64) → `main` @ `25668ec` |
| atm-core AT-style qualification + publish from `develop` | atm-core | **in progress** | [#1069](https://github.com/randlee/atm-core/pull/1069) CI green — merge + v1.4.4 RC retry pending |
| Org pin published | atm/sc-publish | **pending** | Candidate `25668ec` (`main`); await atm qualification receipt |
| Wyvern re-pin @ `25668ec` | wyvern | **draft CI green** | [PR #146](https://github.com/randlee/wyvern/pull/146) — merge after atm RC |
| wyvern `release/sc-publish-pin.toml` bumped + sync dry-run 0 | wyvern | **pending** | Merge #146 after org receipt |

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
