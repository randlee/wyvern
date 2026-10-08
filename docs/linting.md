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

Two repo-root files, on purpose:

| File | Owner | Used by |
|------|-------|---------|
| [`.sc-lint.toml`](../.sc-lint.toml) | Wyvern | Analyzer CLI `--config` (`check native`, `lint sc-boundary`, `lint sc-portability`) |
| [`sc-lint.toml`](../sc-lint.toml) | `sc-lint init --just` | Consumer Just bootstrap (`just setup` / `lint` / `test` / `upgrade`) and `sc-lint lint --consumer` |
| [`sc-lint-analyzers.toml`](../sc-lint-analyzers.toml) | Wyvern | Python analyzers (`line-counts`, `identity-literals`) via [`scripts/sc_lint_python_gate.sh`](../scripts/sc_lint_python_gate.sh) |

[`.sc-lint.toml`](../.sc-lint.toml) is the stable `--config` contract:

```toml
[tool.sc-lint]
minimum_version = "0.5.0"

[workspace]
root = "."
```

Pass `--config .sc-lint.toml` explicitly for analyzer commands. sc-lint 0.5.0
default discovery looks for `sc-lint.toml` then `.just/lint-config.toml` — not
this dotfile.

`sc-lint.toml` holds the product-owned lint/test argv profiles (`fmt`,
`clippy`, `cargo test --workspace`). Do not copy those arrays into
`.sc-lint.toml`: an empty consumer profile fails `--consumer`, and mixing the
files would break `sc-lint init --just --check`.

Portability (`sc-lint lint sc-portability`) still uses built-in
`unix_path_prefixes`. Optional `[portability].config_home_env` is only read
from `sc-lint.toml` or `.just/lint-config.toml` — Wyvern does not set it.

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
| Portability | `sc-lint lint sc-portability` | **Yes** — boundaries CI job |
| Runtime liveness | `sc-lint lint sc-runtime` | **Yes** — Boundary lint job |
| Clippy wrapper | `sc-lint clippy native` | **Yes** — all build matrix legs |
| line-counts | [`scripts/sc_lint_python_gate.sh`](../scripts/sc_lint_python_gate.sh) + [`sc-lint-analyzers.toml`](../sc-lint-analyzers.toml) | **Yes** — Boundary lint job |
| identity-literals | same Python gate | **Yes** — Boundary lint job |
| Consumer lint | `sc-lint lint --consumer --config sc-lint.toml ci` (fmt + clippy) | **Yes** — Ubuntu + Windows consumer CI jobs |
| Local aggregate | `just lint` → same consumer `ci` profile | Local |

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

## Consumer Just bootstrap

`sc-lint init --just` (0.5.0) writes `sc-lint.toml`, `Justfile`,
`.sc-lint/bootstrap`, and `.sc-lint/bootstrap.ps1`. After a clean checkout:

```bash
just setup   # download/verify the minimum_version release if needed
just lint    # sc-lint lint --consumer --config sc-lint.toml ci
just test    # sc-lint test --config sc-lint.toml
```

`just test` is `cargo test --workspace` without `--test-threads=1`. Local and
CI workspace tests must still pass `--test-threads=1` on macOS (winit/objc
races). Prefer the build-matrix `cargo test` job for that gate.

Do **not** run top-level `sc-lint ci` in this repo. That command is
sc-lint's source-maintainer profile: `fmt`, `clippy`, then `.just/*.py`
helpers that `cargo run -p sc-lint-boundary` / `sc-lint-portability`. Those
crates are not Wyvern workspace members.

## CI

Every matrix leg (`ubuntu-latest`, `macos-latest`, `windows-latest`) installs
sc-lint **0.5.0** from the GitHub release bundle, runs **`sc-lint clippy native`**
(JSON gate), then **`sc-lint check native`**. See [`.github/workflows/ci.yml`](../.github/workflows/ci.yml).

The **sc-lint consumer CI** jobs (Ubuntu + Windows, `needs: [fmt]`) run
`sc-lint init --just --check`, then `sc-lint lint --consumer --config
sc-lint.toml ci` (the same aggregate profile as `just lint`; 0.5.0 does not
expose per-profile consumer targets such as `clippy`).
Steps use [`scripts/sc_lint_json_gate.sh`](../scripts/sc_lint_json_gate.sh) because
sc-lint 0.5.0 can exit 0 when `data.status` is not `pass`. `just` is installed
so local `just setup` / `just lint` match documented bootstrap; CI invokes
`sc-lint` directly for deterministic JSON gates.

Workspace tests stay on the build matrix (`--test-threads=1`). Consumer
`just test` / `sc-lint test` use the canonical profile in [`sc-lint.toml`](../sc-lint.toml)
(without `--test-threads=1`); keep macOS threading on the matrix job or pass flags locally.

The **Boundary lint** job runs `sc-lint lint sc-boundary`, `sc-lint lint
sc-portability`, `sc-lint lint sc-runtime`, Python **`line-counts`** and
**`identity-literals`** (policy in [`sc-lint-analyzers.toml`](../sc-lint-analyzers.toml);
see [j5 sprint plan](plans/phase-J/j5-sc-lint-extended-analyzers.md)), then
`scripts/check-boundaries.py` (io_forbidden greps) and ui/share sync checks.

### Debugging analyzer output

```bash
sc-lint view findings --config .sc-lint.toml   # after a lint run wrote artifacts
sc-lint view graph --config .sc-lint.toml      # boundary graph (when supported)
```

### Stack #160 landing (sc-lint 0.5.0)

After PR **#159** is green: `gh stack sync --remote origin`, confirm stack coherence on
`develop`, merge bottom **#154** upward (or `gh stack merge` per team workflow). Trunk
should include observability **#161** via the stack base merge commit.

## Release preflight tooling

[`.github/actions/setup-lint-toolchain`](../.github/actions/setup-lint-toolchain)
(cargo-deny, shear, codespell, etc.) is used only by
[`release-preflight.yml`](../.github/workflows/release-preflight.yml), not PR CI.
PR CI uses the sc-lint release bundle via `setup-sc-lint`.

## Portability policy

`sc-lint lint sc-portability` needs no extra policy file. The analyzer ships
built-in `unix_path_prefixes` (`/tmp/`, `/var/tmp/`, `/private/tmp/`) and only
reads optional `[portability].config_home_env` from `sc-lint.toml` or
`.just/lint-config.toml` — not from [`.sc-lint.toml`](../.sc-lint.toml). There
is no rule-disable / allowlist surface in 0.5.0.
