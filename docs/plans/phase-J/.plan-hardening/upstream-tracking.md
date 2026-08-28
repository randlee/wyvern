# Waiver and upstream disposition (j.2 closeout)

Each blocker is exactly one state:

| Blocker | State | PR / commit | Signed waiver | Effect |
|---------|-------|-------------|---------------|--------|
| CR-001 Linux webview deps | **resolved** | sc-publish [PR #61](https://github.com/randlee/sc-publish/pull/61) merged → `main` @ `43552e4c9e6d3435ed58a4a7eca42dd82f7edb74` | | Kit jobs install webkit/wayland apt packages via `.github/actions/install-linux-native-deps`. |
| CR-002 Homebrew/Scoop renderer | **resolved** | same merge as CR-001 | | `homebrew-publish.yml` and `scoop-publish.yml` bootstrap the pinned renderer via `.github/actions/setup-renderer` (not the product binary). `release/install.json` keeps non-empty `renderer_archive_path` = `bin/wyvern`. |

**Rules:**

- `resolved` ⇒ merged sc-publish (or wyvern) PR link recorded; j.3 may proceed.
- `waived` ⇒ **blocks j.3 and j.4 entirely** (no re-sign escape). Phase J pauses until resolved.
- CR-002 resolved does **not** unblock Homebrew while CR-001 is still `open`.
- This closeout does **not** sign waivers.

## Disposition notes

### CR-001

Kit composite action `.github/actions/install-linux-native-deps` installs
`libwebkit2gtk-4.1-dev`, `libwayland-dev`, and the matching runtime/Xvfb
packages. Wired into:

- `release.yml` build matrix (Linux no-op on macOS/Windows)
- `release.yml` crates publish job
- `release-preflight.yml` (when the manifest has crates)
- `crates-publish.yml`

### CR-002

Both channel workflows call `.github/actions/setup-renderer`, which runs
`bootstrap_sc_compose.py --write-cli` and exports `PUBLISHED_RENDERER`. They no
longer extract `project.renderer_archive_path` from the Linux product archive.
`renderer_archive_path` remains required by the kit schema while Scoop/Homebrew
are declared; Wyvern keeps `bin/wyvern` (archive binary path, not the renderer).

Wyvern is re-synced to sc-publish `43552e4` via `scripts/sync-sc-publish.sh`.

## j.2 closeout extras

| Item | Status |
|------|--------|
| `WINGET_GITHUB_TOKEN` | Present in `gh secret list` (shared org PAT — not created in j.2) |
| `SCOOP_BUCKET_TOKEN` | Present in `gh secret list` (shared org PAT — not created in j.2) |
| `randlee/scoop-bucket` | Public, cloneable: https://github.com/randlee/scoop-bucket ; `bucket/` has only `.gitkeep` (workflow seeds `bucket/wyvern.json`) |
| Scoop push probe | Authenticated `gh api repos/randlee/scoop-bucket` as `randlee` reports `permissions.push=true` |
| `randlee.wyvern` in `winget-pkgs` | **Absent** (`manifests/r/randlee/wyvern` 404). Owner bootstrap **before j.3**. |
| `homebrew_destination_components` | `["share","wyvern","ui"]` in `release/install.json` |
| `scripts/validate_release.py` | Deleted (already absent on this branch) |

Updated in j.2 sprint closeout.
