# Release secrets (wyvern)

Wyvern uses the **same GitHub repository secrets and environments as
[`atm-core`](https://github.com/randlee/atm-core)**. No wyvern-specific token
names are required.

## Repository secrets

| Secret | Purpose | Used in |
|--------|---------|---------|
| `CARGO_REGISTRY_TOKEN` | crates.io publish auth | `.github/workflows/release.yml` → `publish-crates` |
| `HOMEBREW_TAP_TOKEN` | Push formula updates to `randlee/homebrew-tap` | `.github/workflows/release.yml` → `update-homebrew` |
| `GITHUB_TOKEN` | GitHub Release assets + winget-releaser API | `.github/workflows/release.yml` (built-in; no manual setup) |

## GitHub Environment

| Environment | Secret gated | Job |
|-------------|--------------|-----|
| `crates-io` | `CARGO_REGISTRY_TOKEN` | `publish-crates` |

Configure the `crates-io` environment on `randlee/wyvern` the same way as on
`randlee/atm-core` (environment secret binding for `CARGO_REGISTRY_TOKEN`).

## Workflow env vars (not secrets)

| Name | Value | Purpose |
|------|-------|---------|
| `RELEASE_ARTIFACT_MANIFEST` | `release/publish-artifacts.toml` | Publish inventory SSoT |

## Distribution channels

- **crates.io** — crates listed in `release/publish-artifacts.toml`
- **GitHub Releases** — `wyvern`, `wyvern-viewer`, `share/wyvern/ui/` per platform
- **Homebrew** — `randlee/homebrew-tap` → `Formula/wyvern.rb` (Apple Silicon tarball)
- **winget** — `randlee.wyvern` via `winget-releaser` on `wyvern-windows.zip`

See also: [`docs/WINGET_SETUP.md`](WINGET_SETUP.md), [`release/publish-artifacts.toml`](../release/publish-artifacts.toml).
