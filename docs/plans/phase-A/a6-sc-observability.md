---
id: a.6
title: sc-observability integration
status: planned
branch: feature/phase-A-a6-sc-observability
target: integrate/phase-A
---

# Sprint a.6 — sc-observability integration

## Goal

- Structured logging at `crates/wyvern/src/main.rs` only, using sibling `sc-observability`.

## Hard Dependencies

- a.5 chrome E2E (events to log)

## Exact Targets

- `crates/wyvern/Cargo.toml`
- `crates/wyvern/src/main.rs`
- `crates/wyvern/src/observability.rs` (thin wrapper)
- `docs/observability.md`
- `.github/workflows/ci.yml` (clone sibling before build)

## Deliverables

- Path dep: `sc-observability = { path = "../../sc-observability" }` in `crates/wyvern/Cargo.toml`
- Log events: `process_start`, `command_received`, `validation_result`, `window_open`, `window_close`, `result_emitted`, `error`
- `WYVERN_LOG` env var documented
- CI clones siblings beside repo; Phase A CI policy below (authoritative until Phase C)

## Phase A CI policy (authoritative)

Phase A window/chrome work is **macOS-only**. CI keeps the cross-platform matrix but gates window-dependent steps:

| Leg | `cargo build` / `clippy` | `cargo test --workspace` | Sibling checkout |
|-----|--------------------------|--------------------------|------------------|
| `macos-latest` | full workspace | full workspace (incl. window tests) | `sc-observability`, `sc-lint` |
| `ubuntu-latest`, `windows-latest` | full workspace | **skip** `wyvern-window` macOS integration tests via `#[cfg(target_os = "macos")]` no-op pattern (a.2) | `sc-observability`, `sc-lint` |

a.7 lint step runs on **all** legs after sibling checkout.

### CI YAML sample (append to `.github/workflows/ci.yml`)

```yaml
    steps:
      - uses: actions/checkout@v4

      - name: Checkout sc-observability sibling
        uses: actions/checkout@v4
        with:
          repository: ${{ github.repository_owner }}/sc-observability
          path: sc-observability

      - name: Checkout sc-lint sibling
        uses: actions/checkout@v4
        with:
          repository: ${{ github.repository_owner }}/sc-lint
          path: sc-lint

      # Repo root is $GITHUB_WORKSPACE; siblings sit beside it:
      #   $GITHUB_WORKSPACE/../sc-observability  → ../../sc-observability from crates/wyvern
      - name: Link siblings for path deps
        run: |
          ln -sf "$GITHUB_WORKSPACE/../sc-observability" "${{ github.workspace }}/../sc-observability" || true
          ln -sf "$GITHUB_WORKSPACE/../sc-lint" "${{ github.workspace }}/../sc-lint" || true

      # ... Rust toolchain, build, clippy ...

      - name: cargo test (macOS full)
        if: runner.os == 'macOS'
        run: cargo test --workspace

      - name: cargo test (non-macOS — window tests no-op)
        if: runner.os != 'macOS'
        run: cargo test --workspace
        # wyvern-window macOS tests are #[cfg]-gated; non-macOS legs pass via a.2 no-op test
```

Adjust checkout `repository` to the actual org/name; layout must resolve `../../sc-observability` from `crates/wyvern/Cargo.toml`.

## Explicit Code Samples

```toml
# crates/wyvern/Cargo.toml
sc-observability = { path = "../../sc-observability" }
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

- With `../../sc-observability` present, `cargo build -p wyvern` succeeds
- `WYVERN_LOG=debug` emits structured events on chrome E2E path
- `rg 'sc_observability' crates/wyvern-schema crates/wyvern-window crates/wyvern-wizard crates/wyvern-mcp` → empty
- CI workflow implements Phase A CI policy (sibling checkout + macOS/full test gate per a.6)
- `docs/observability.md` documents path layout and event list

## Required Validation

- `cargo build -p wyvern` (sibling present)
- `rg 'sc_observability' crates/wyvern-schema crates/wyvern-window crates/wyvern-wizard crates/wyvern-mcp` → empty
- `cargo clippy --workspace -- -D warnings`
