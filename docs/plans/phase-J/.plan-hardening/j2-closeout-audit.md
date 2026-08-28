# j.2 closeout audit (2026-08-28)

Evidence snapshot for sprint j.2 acceptance criteria on `integrate/phase-J` @ `8490a19`.

## AC status

| # | Criterion | Status | Evidence |
|---|-----------|--------|----------|
| 1 | `homebrew_destination_components`; sync dry-run 0 | **done** | `release/install.json`; `./scripts/sync-sc-publish.sh` exit 0 @ `42e0fce` |
| 2 | CR-001 resolved in upstream-tracking | **pending** | sc-publish PR #63 on `main` @ `25668ec`; wyvern pin not advanced until org receipt |
| 3 | CR-002 resolved; `renderer_archive_path` non-empty | **pending** | Same org gate; `renderer_archive_path`: `"bin/wyvern"` |
| 4 | `RELEASE_SECRETS.md` documents tokens | **done** | `rg WINGET_GITHUB_TOKEN\|SCOOP_BUCKET_TOKEN docs/RELEASE_SECRETS.md` |
| 5 | `WINGET_SETUP.md` matches kit | **done** | Post-release dispatch, token, asset pattern documented |
| 6 | `SCOOP_SETUP.md` complete | **done** | Bucket repo, token model, asset naming |
| 7 | `README.md` artifact table kit names | **done** | `wyvern_<version>_<target>.*` |
| 8 | Secrets provisioned | **done** | `gh secret list`: `WINGET_GITHUB_TOKEN`, `SCOOP_BUCKET_TOKEN` |
| 9 | Winget bootstrap if absent | **submitted** | [winget-pkgs #425477](https://github.com/microsoft/winget-pkgs/pull/425477) (merge lag OK for j.3 leg) |
| 10 | Scoop bootstrap closed | **done** | `randlee/scoop-bucket` public; token present; manifest seeded by workflow on first run |
| 11 | upstream-tracking CR-001/002 **resolved** | **pending** | Blocked on org pin receipt + wyvern re-sync @ blessed SHA |

## j.2 closure rule

j.2 **cannot close** until AC #2, #3, #9, and #11 are green. CR items flip to
`resolved` only after wyvern `release/sc-publish-pin.toml` advances to org blessed
SHA and `./scripts/sync-sc-publish.sh` exits 0.

## Wyvern-only gates already green

- [PR #145](https://github.com/randlee/wyvern/pull/145) merged: isolated `.sc-publish-kit/` cache, pin file @ `42e0fce`
- `scripts/validate_release.py` deleted
- `crates-io` GitHub environment exists

## Next unblock sequence

1. atm-core v1.4.4 RC + publish succeeds on kit @ `25668ec`
2. Org pin receipt published
3. Wyvern bump `release/sc-publish-pin.toml` → sync dry-run → upstream-tracking **resolved**
4. Owner winget bootstrap (or staged manifest submit using v0.5.0 asset)
5. j.3 production release
