# Waiver and upstream disposition (j.2 closeout)

Each blocker is exactly one state:

| Blocker | State | PR / commit | Signed waiver | Effect |
|---------|-------|-------------|---------------|--------|
| CR-001 Linux webview deps | **ready** | sc-publish `main` @ `25668ec` | | Merge wyvern re-pin after atm RC receipt |
| CR-002 Homebrew/Scoop renderer | **ready** | same | | same |
| CR-RC Git identity on RC tag | **ready** | `release-candidate.yml` @ `25668ec` | | same |

**Rules:**

- `resolved` ⇒ org blessed pin published **and** wyvern re-sync dry-run exit **0** at that SHA.
- `pending` ⇒ j.3/j.4 **blocked** until org pin advance (not a waiver).
- `waived` ⇒ **blocks j.3 and j.4 entirely** (no re-sign escape). Phase J pauses until resolved.
- Wyvern does **not** modify sc-publish for repo-specific quirks; consumer changes only.

## Wyvern kit pin (consumer)

| Item | Value |
|------|-------|
| Pin file | `release/sc-publish-pin.toml` |
| Current revision | `42e0fce` on `integrate/phase-J`; re-pin staged on `feature/phase-J-repin-25668ec` |
| Sync entrypoint | `scripts/sync-sc-publish.sh` → isolated `.sc-publish-kit/` cache |
| Target org revision | `25668ec` — [atm-core #1069](https://github.com/randlee/atm-core/pull/1069) |

**Note:** sc-publish `main` @ `25668ec` (PR #64, reconciled kit from PR #63). Wyvern stays on `42e0fce` until org pin receipt + sync dry-run at blessed SHA.

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
