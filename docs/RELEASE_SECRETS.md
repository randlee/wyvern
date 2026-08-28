# Release secrets (wyvern)

Wyvern uses the **sc-publish** kit secret names. These names are fixed by
`release/publish-channel-contracts.toml` and must match every other kit
consumer (including [`atm-core`](https://github.com/randlee/atm-core)).

Do **not** invent per-repo token names. Repository `GITHUB_TOKEN` is **not** a
substitute for the PAT secrets below.

## Repository secrets

| Secret | Purpose | Used in | Minimum scope |
|--------|---------|---------|---------------|
| `CARGO_REGISTRY_TOKEN` | crates.io publish auth | `release.yml` publish job; `crates-publish.yml` | crates.io API token with publish rights for the declared crates |
| `HOMEBREW_TAP_TOKEN` | Push formula updates to `randlee/homebrew-tap` | `homebrew-publish.yml` | PAT with `contents:write` on `randlee/homebrew-tap` |
| `WINGET_GITHUB_TOKEN` | Fork `microsoft/winget-pkgs` and open PRs | `winget-publish.yml` | Classic PAT with `public_repo`, **or** fine-grained PAT that can **fork** `microsoft/winget-pkgs` and **open PRs** |
| `SCOOP_BUCKET_TOKEN` | Push `bucket/wyvern.json` to `randlee/scoop-bucket` | `scoop-publish.yml` | PAT with `contents:write` on `randlee/scoop-bucket` |

`GITHUB_TOKEN` (built-in) is used only for checkout, GitHub Release assets, and
read-only API probes. It **cannot** fork `microsoft/winget-pkgs` and is **not**
used to push the Homebrew tap or Scoop bucket.

Kit preflight (`release-preflight.yml`) fails closed if a declared-channel
secret is missing or rejected by GitHub `GET /user`.

## GitHub Environment

| Environment | Secret gated | Job |
|-------------|--------------|-----|
| `crates-io` | `CARGO_REGISTRY_TOKEN` | `release.yml` publish job; `crates-publish.yml` |

Configure the `crates-io` environment on `randlee/wyvern` the same way as on
`randlee/atm-core` (environment secret binding for `CARGO_REGISTRY_TOKEN`).

## Workflow env vars (not secrets)

| Name | Value | Purpose |
|------|-------|---------|
| `RELEASE_ARTIFACT_MANIFEST` | `release/publish-artifacts.toml` | Publish inventory SSoT |

## Distribution channels

- **crates.io** — crates listed in `release/publish-artifacts.toml`
- **GitHub Releases** — `wyvern_<version>_<target>.{tar.gz,zip}` with `bin/` + `share/wyvern/ui/`
- **Homebrew** — `randlee/homebrew-tap` → `Formula/wyvern.rb` via `homebrew-publish.yml`
- **Scoop** — `randlee/scoop-bucket` → `bucket/wyvern.json` via `scoop-publish.yml`
- **winget** — `randlee.wyvern` via `winget-publish.yml` (requires one-time
  `microsoft/winget-pkgs` bootstrap before the first automated submit)

PyPI is **not** declared; preflight does not require PyPI tokens.

## j.2 closeout — secret presence

`gh secret list` on `randlee/wyvern` already includes:

- `CARGO_REGISTRY_TOKEN`
- `HOMEBREW_TAP_TOKEN`
- `WINGET_GITHUB_TOKEN`
- `SCOOP_BUCKET_TOKEN`

j.2 does **not** create or rotate secrets.

See also: [`docs/WINGET_SETUP.md`](WINGET_SETUP.md),
[`docs/SCOOP_SETUP.md`](SCOOP_SETUP.md),
[`release/publish-artifacts.toml`](../release/publish-artifacts.toml).
