---
id: a.6
title: sc-observability integration
status: planned
branch: feature/phase-A-a6-sc-observability
target: integrate/phase-A
---

# Sprint a.6 — sc-observability integration

## Goal

- Structured logging at `crates/wyvern/src/main.rs` only, using **`sc-observability` from crates.io**.

## Hard Dependencies

- a.5 chrome E2E (events to log)

## Exact Targets

- `Cargo.toml` (workspace dependency pin)
- `crates/wyvern/Cargo.toml`
- `crates/wyvern/src/main.rs`
- `crates/wyvern/src/observability.rs` (thin wrapper)
- `docs/observability.md`
- `.github/workflows/ci.yml` (cross-platform test matrix)

## Deliverables

- Workspace + crate dep: `sc-observability = "1.2"` (crates.io — **no path/sibling checkout**)
- Log events: `process_start`, `command_received`, `validation_result`, `window_open`, `window_close`, `result_emitted`, `error`
- `WYVERN_LOG` env var documented
- CI runs full `cargo test --workspace` on all matrix legs (see policy below)

## Phase A CI policy (authoritative)

| Leg | `cargo build` / `clippy` | `cargo test --workspace` |
|-----|--------------------------|--------------------------|
| `ubuntu-latest` | full workspace | full workspace (incl. window tests; install Linux webview deps) |
| `macos-latest` | full workspace | full workspace |
| `windows-latest` | full workspace | full workspace |

No platform skips or `#[cfg]` no-op substitutes for window integration tests. a.7 `sc-lint` step runs on **all** legs.

### CI YAML sample (`.github/workflows/ci.yml` — no sibling checkouts)

```yaml
jobs:
  build:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4

      - name: Install Linux webview deps
        if: runner.os == 'Linux'
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev

      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt

      - run: cargo build --workspace
      - run: cargo test --workspace
      - run: cargo clippy --workspace -- -D warnings
```

## Explicit Code Samples

```toml
# Cargo.toml (workspace)
[workspace.dependencies]
sc-observability = "1.2"

# crates/wyvern/Cargo.toml
sc-observability = { workspace = true }
```

```rust
// crates/wyvern/src/observability.rs
use sc_observability::{init_from_env, log_event};

pub fn init() -> Result<(), sc_observability::Error> {
    init_from_env("WYVERN_LOG")
}

pub fn log_process_start() {
    log_event("wyvern.process_start", &[]);
}

pub fn log_command_received(cmd: &serde_json::Value) {
    log_event("wyvern.command_received", &[("type", cmd.get("type").and_then(|v| v.as_str()).unwrap_or(""))]);
}

pub fn log_validation_result(ok: bool) {
    log_event("wyvern.validation_result", &[("ok", if ok { "true" } else { "false" })]);
}

pub fn log_window_open() { log_event("wyvern.window_open", &[]); }
pub fn log_window_close() { log_event("wyvern.window_close", &[]); }
pub fn log_result_emitted() { log_event("wyvern.result_emitted", &[]); }
pub fn log_error(stage: &str, detail: &str) {
    log_event("wyvern.error", &[("stage", stage), ("detail", detail)]);
}
```

Event keys above are normative; wrapper may rename `sc-observability` call symbols only if event keys and `WYVERN_LOG` behavior remain identical.

## This Sprint Does Not Close

- Logging inside `wyvern-schema`, `wyvern-window`, or other libs
- MCP/interactive logging

## Acceptance Criteria

- `cargo build -p wyvern` succeeds with crates.io `sc-observability` only (no local path dep)
- `WYVERN_LOG=debug` emits structured events on chrome E2E path
- `rg 'sc_observability' crates/wyvern-schema crates/wyvern-window crates/wyvern-wizard crates/wyvern-mcp` → empty
- CI: `cargo test --workspace` passes on ubuntu, macOS, and Windows
- `docs/observability.md` documents crates.io version pin and event list

## Required Validation

- `cargo build -p wyvern`
- `rg 'path.*sc-observability' Cargo.toml crates/` → empty
- `rg 'sc_observability' crates/wyvern-schema crates/wyvern-window crates/wyvern-wizard crates/wyvern-mcp` → empty
- `cargo clippy --workspace -- -D warnings`
