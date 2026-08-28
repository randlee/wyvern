# Waiver and upstream disposition (j.2 closeout)

Each blocker is exactly one state:

| Blocker | State | PR / commit | Signed waiver | Effect |
|---------|-------|-------------|---------------|--------|
| CR-001 Linux webview deps | **pending** | sc-publish [PR #63](https://github.com/randlee/sc-publish/pull/63) (includes #61) | | Awaiting org-wide qualification before wyvern re-pin. Wyvern stays on `42e0fce` until blessed SHA published. |
| CR-002 Homebrew/Scoop renderer | **pending** | same as CR-001 | | Same gate. #61 `setup-renderer` path must pass atm-core AT-style qualification. |
| CR-RC Git identity on RC tag | **pending** | PR #63 (supersedes #62) | | atm-core v1.4.4 RC failed without this; included in PR #63. |

**Rules:**

- `resolved` ⇒ org blessed pin published **and** wyvern re-sync dry-run exit **0** at that SHA.
- `pending` ⇒ j.3/j.4 **blocked** until org pin advance (not a waiver).
- `waived` ⇒ **blocks j.3 and j.4 entirely** (no re-sign escape). Phase J pauses until resolved.
- Wyvern does **not** modify sc-publish for repo-specific quirks; consumer changes only.

## Wyvern kit pin (consumer)

| Item | Value |
|------|-------|
| Pin file | `release/sc-publish-pin.toml` |
| Current revision | `42e0fcea23f730fae0ef3d08b060cd4df6a2602e` (atm-core AT.2) |
| Sync entrypoint | `scripts/sync-sc-publish.sh` → isolated `.sc-publish-kit/` cache |
| Target org revision | PR #63 @ `928c8f9` after merge + qualification |

**Note:** sc-publish `main` @ `43552e4` (#61 merged unqualified) is **not** wyvern's pin. Do not re-sync to `main` until org blessed release.

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
