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

Repo-root [`.sc-lint.toml`](../.sc-lint.toml) declares the analyzer CLI contract.
Python analyzers (`line-counts`, `identity-literals`) use
[`sc-lint-analyzers.toml`](../sc-lint-analyzers.toml) via
[`scripts/sc_lint_python_gate.sh`](../scripts/sc_lint_python_gate.sh) — do not
merge that policy into `.sc-lint.toml`.

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
| Clippy wrapper | `sc-lint clippy native` | **Yes** — all build matrix legs |
| Boundary graph | `sc-lint lint sc-boundary` | **Yes** — boundaries CI job |
| Portability | `sc-lint lint sc-portability` | **Yes** — boundaries CI job |
| Runtime liveness | `sc-lint lint sc-runtime` | **Yes** — Boundary lint job |
| line-counts | [`scripts/sc_lint_python_gate.sh`](../scripts/sc_lint_python_gate.sh) + [`sc-lint-analyzers.toml`](../sc-lint-analyzers.toml) | **Yes** — Boundary lint job |
| identity-literals | same Python gate | **Yes** — Boundary lint job |
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
| `sc-lint clippy native` | **Indirect** — wraps `cargo clippy -D warnings`; honors crate `#![deny(...)]` | Yes (matrix) |
| `sc-lint lint sc-boundary` | **No** — dependency/ownership graph | Yes |
| `sc-lint lint sc-runtime` | **No** — condvar liveness only | Yes (Boundary lint) |

Authoritative regression gate:

1. Crate-root `#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unreachable, clippy::todo, clippy::unimplemented))]` on the four roots above
2. `sc-lint clippy native --config .sc-lint.toml` in [`.github/workflows/ci.yml`](../.github/workflows/ci.yml) (JSON-gated)

`#![allow(...)]` for these lints is permitted only inside `#[cfg(test)]` modules.

Optional local alias for the same clippy gate:

```bash
sc-lint clippy native --config .sc-lint.toml
```

## CI

Every matrix leg (`ubuntu-latest`, `macos-latest`, `windows-latest`) installs
sc-lint **0.5.0** from the GitHub release bundle, runs **`sc-lint clippy native`**
(JSON gate via [`scripts/sc_lint_json_gate.sh`](../scripts/sc_lint_json_gate.sh)),
then **`sc-lint check native`**. See [`.github/workflows/ci.yml`](../.github/workflows/ci.yml).

The **Boundary lint** job runs `sc-lint lint sc-boundary`, `sc-lint lint
sc-portability`, `sc-lint lint sc-runtime` (JSON `data.status == pass` via
[`scripts/sc_lint_json_gate.sh`](../scripts/sc_lint_json_gate.sh); the 0.5.0 CLI
exits 0 even when findings exist), Python **`line-counts`** and
**`identity-literals`** (policy in [`sc-lint-analyzers.toml`](../sc-lint-analyzers.toml);
see [j5 sprint plan](plans/phase-J/j5-sc-lint-extended-analyzers.md)), then
`scripts/check-boundaries.py` (io_forbidden greps) and ui/share sync checks.

### Debugging analyzer output

```bash
sc-lint view findings --config .sc-lint.toml   # after a lint run wrote artifacts
sc-lint view graph --config .sc-lint.toml      # boundary graph (when supported)
```

`sc-lint lint sc-portability` needs no extra policy file. The analyzer ships
built-in `unix_path_prefixes` (`/tmp/`, `/var/tmp/`, `/private/tmp/`) and only
reads optional `[portability].config_home_env` from `sc-lint.toml` or
`.just/lint-config.toml` — not from [`.sc-lint.toml`](../.sc-lint.toml). There
is no rule-disable / allowlist surface in 0.5.0.
