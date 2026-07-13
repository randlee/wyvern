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
- CI clones `sc-observability` beside repo root (required)

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

Adjust names to match the actual `sc-observability` API when wiring; event keys above are the required contract.

## This Sprint Does Not Close

- Logging inside `wyvern-schema`, `wyvern-window`, or other libs
- MCP/interactive logging

## Acceptance Criteria

- With `../../sc-observability` present, `cargo build -p wyvern` succeeds
- `WYVERN_LOG=debug` emits structured events on chrome E2E path
- `rg 'sc_observability' crates/wyvern-schema crates/wyvern-window crates/wyvern-wizard crates/wyvern-mcp` → empty
- CI workflow checks out `sc-observability` sibling (required — no feature-flag skip)
- `docs/observability.md` documents path layout and event list

## Required Validation

- `cargo build -p wyvern` (sibling present)
- `rg 'sc_observability' crates/wyvern-schema crates/wyvern-window crates/wyvern-wizard crates/wyvern-mcp` → empty
- `cargo clippy --workspace -- -D warnings`
