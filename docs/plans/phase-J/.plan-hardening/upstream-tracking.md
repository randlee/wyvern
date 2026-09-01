# Waiver and upstream disposition (j.2 closeout)

Each blocker is exactly one state:

| Blocker | State | PR / commit | Signed waiver | Effect |
|---------|-------|-------------|---------------|--------|
| CR-001 Linux webview deps | **resolved** | sc-publish `main` @ `25668ec`; wyvern sync @ `25668ec` (PR #146) |
| CR-002 Homebrew/Scoop renderer | **resolved** | same |
| CR-RC Git identity on RC tag | **resolved** | atm-core RC run 33139520613 success |

**Rules:**

- `resolved` ⇒ org blessed pin published **and** wyvern re-sync dry-run exit **0** at that SHA.
- `pending` ⇒ j.3/j.4 **blocked** until org pin advance (not a waiver).
- `waived` ⇒ **blocks j.3 and j.4 entirely** (no re-sign escape). Phase J pauses until resolved.
- Wyvern does **not** modify sc-publish for repo-specific quirks; consumer changes only.

## Wyvern kit pin (consumer)

| Item | Value |
|------|-------|
| Pin file | `release/sc-publish-pin.toml` |
| Current revision | `25668ecc164261be676c9414c4f603b18ab74c91` (org blessed, PR #146) |
| Sync entrypoint | `scripts/sync-sc-publish.sh` → isolated `.sc-publish-kit/` cache |
| Target org revision | `25668ec` — qualified via atm-core #1069 + RC 33139520613 |

**Note:** Org pin active on wyvern @ `25668ec`. j.3 RC dispatch blocked until [PR #148](https://github.com/randlee/wyvern/pull/148) registers workflow on `main` (default branch) or default branch switches to `develop`.

## j.2 closeout extras

| Item | Status |
|------|--------|
| `WINGET_GITHUB_TOKEN` | Present in `gh secret list` (shared org PAT) |
| `SCOOP_BUCKET_TOKEN` | Present in `gh secret list` (shared org PAT) |
| `randlee/scoop-bucket` | Public, cloneable; workflow seeds `bucket/wyvern.json` |
| `randlee.wyvern` in `winget-pkgs` | **Absent** — owner bootstrap **before j.3** |
| `homebrew_destination_components` | `["share","wyvern","ui"]` in `release/install.json` |
| `scripts/validate_release.py` | Deleted |
| Extension request doc | [sc-publish-extension-requests.md](sc-publish-extension-requests.md) |

Updated after org-wide kit policy correction (2026-08-28).
