# Linting

Wyvern uses [`sc-lint`](https://crates.io/crates/sc-lint) **0.5.0** for workspace
policy checks in local development and CI. CI installs the verified GitHub release
bundle (`sc-lint`, `sc-lint-boundary`, `sc-lint-portability`, `sc-lint-runtime`)
via [`.github/actions/setup-sc-lint`](../.github/actions/setup-sc-lint).

Boundary dependency allowlists and forbidden edges are enforced by
`sc-lint lint sc-boundary` against ADR-004 records under `boundaries/<owner-package>/`.
Wyvern-specific `io_forbidden` grep policy lives in
[`scripts/io-forbidden.toml`](../scripts/io-forbidden.toml) and is checked by
`scripts/check-boundaries.py`.

## Install

Pin to **0.5.0** (local development):

```bash
cargo install sc-lint --version 0.5.0 --locked
```

Ensure `~/.cargo/bin` is on `PATH` so the crates.io binary is used (Homebrew
formulas may ship an older `sc-lint`).

## Config

Repo-root [`.sc-lint.toml`](../.sc-lint.toml) declares the consumer contract:

```toml
[tool.sc-lint]
minimum_version = "0.5.0"

[workspace]
root = "."
```

Pass `--config .sc-lint.toml` explicitly so CI and local runs share the same
file.

## Canonical command

```bash
sc-lint check native --config .sc-lint.toml
```

`check` requires a target (`native` or `xwin`). CI uses `native`, which
runs `cargo check --workspace` and must pass with zero warnings/failures.

Always pass `--test-threads=1` for workspace tests on macOS (winit/objc races when
multiple webview children spawn). CI already enforces this; local runs must match.

## Published analyzers (0.5.0)

| Backend | CLI target | Wyvern CI |
|---------|------------|-----------|
| Compile gate | `sc-lint check native` | **Yes** — all matrix legs |
| Boundary graph | `sc-lint lint sc-boundary` | **Yes** — boundaries CI job |
| Portability | `sc-lint lint sc-portability` | Not run |
| Runtime liveness | `sc-lint lint sc-runtime` | Setup smoke test only |
| Full consumer CI | `sc-lint ci` | Not run (requires `sc-lint init --just`) |

## Panic policy

Production paths must not panic. Panics are forbidden in non-test code in
`wyvern`, `wyvern-schema`, and `wyvern-window` (library roots and
`crates/wyvern/src/main.rs`). Test code may use `unwrap` / `expect` /
`panic!`.

**Enforcement is Clippy crate-root denies — not a `.sc-lint.toml` key.**
`sc-lint` 0.5.x has no panic/unwrap policy knobs.

| Surface | Detects production `unwrap`/`expect`/`panic!`? | Wyvern CI |
|---------|-----------------------------------------------|-----------|
| `sc-lint check native` | **No** — wraps `cargo check --workspace` | Yes |
| `sc-lint clippy native` | **Indirect** — wraps `cargo clippy -D warnings`; honors crate `#![deny(...)]` | No (direct `cargo clippy` instead) |
| `sc-lint lint sc-boundary` | **No** — dependency/ownership graph | Yes |
| `sc-lint lint sc-runtime` | **No** — condvar liveness only | Setup smoke only |

Authoritative regression gate:

1. Crate-root `#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unreachable, clippy::todo, clippy::unimplemented))]` on the four roots above
2. Existing `cargo clippy --workspace -- -D warnings` in [`.github/workflows/ci.yml`](../.github/workflows/ci.yml)

`#![allow(...)]` for these lints is permitted only inside `#[cfg(test)]` modules.

Optional local alias for the same clippy gate:

```bash
sc-lint clippy native --config .sc-lint.toml
```

## CI

Every matrix leg (`ubuntu-latest`, `macos-latest`, `windows-latest`) installs
sc-lint **0.5.0** from the GitHub release bundle and runs the canonical command
above. See [`.github/workflows/ci.yml`](../.github/workflows/ci.yml).

The **Boundary lint** job runs `sc-lint lint sc-boundary`, `scripts/check-boundaries.py`
(io_forbidden greps), and ui/share sync checks.
