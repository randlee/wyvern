# Linting

Wyvern uses [`sc-lint`](https://crates.io/crates/sc-lint) from crates.io for
workspace policy checks in local development and CI.

Boundary TOML under `boundaries/` is inventory for later phases; Phase A does
not enforce `sc-lint-boundary` in CI (see Phase B planning).

## Install

Pin to the 0.4 line (exact release `0.4.0`):

```bash
cargo install sc-lint --version 0.4.0 --locked
```

Ensure `~/.cargo/bin` is on `PATH` so the crates.io binary is used (Homebrew
formulas may still ship an older `sc-lint`).

## Config

Repo-root [`.sc-lint.toml`](../.sc-lint.toml) scopes the tool to this
workspace:

```toml
[workspace]
root = "."
```

Pass `--config .sc-lint.toml` explicitly so CI and local runs share the same
file.

## Canonical command

```bash
sc-lint check native --config .sc-lint.toml
```

`check` requires a target (`native` or `xwin`). Phase A CI uses `native`, which
runs `cargo check --workspace` and must pass with zero warnings/failures.

Always pass `--test-threads=1` for workspace tests on macOS (winit/objc races when
multiple webview children spawn). CI already enforces this; local runs must match.

## CI

Every matrix leg (`ubuntu-latest`, `macos-latest`, `windows-latest`) installs
`sc-lint` 0.4.0 from crates.io and runs the canonical command above. See
[`.github/workflows/ci.yml`](../.github/workflows/ci.yml).
