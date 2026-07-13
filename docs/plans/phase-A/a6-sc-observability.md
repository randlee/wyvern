---
id: a.6
title: sc-observability integration
status: planned
branch: feature/phase-A-a6-sc-observability
target: integrate/phase-A
---

# Sprint a.6 — sc-observability integration

## Goal

- Structured logging via **`sc-observability` from crates.io**, wired at the CLI pipeline boundary.

## Hard Dependencies

- a.5 chrome E2E (events to log)

## Exact Targets

- `Cargo.toml` (workspace dependency pin)
- `crates/wyvern/Cargo.toml`
- `crates/wyvern/src/pipeline.rs` (stage hooks)
- `crates/wyvern/src/observability.rs` (thin wrapper)
- `docs/observability.md`
- `.github/workflows/ci.yml` (xvfb on Linux per README CI section)

## Deliverables

- Workspace + crate dep: `sc-observability = "1.2"` (crates.io — no path dep)
- Normative events: `process_start`, `command_received`, `validation_result`, `window_open`, `window_close`, `result_emitted`, `error`
- `WYVERN_LOG` env var documented
- Pipeline integration sample (below) — logging calls live in `pipeline.rs`; `main.rs` calls `observability::init()` only

## Explicit Code Samples

```toml
# Cargo.toml (workspace)
[workspace.dependencies]
sc-observability = "1.2"

# crates/wyvern/Cargo.toml
sc-observability = { workspace = true }
```

```rust
// crates/wyvern/src/pipeline.rs — observability hooks at each stage
pub fn run_from_loaded(value: serde_json::Value) -> Result<String, (String, i32)> {
    observability::log_command_received(&value);
    let command = match wyvern_schema::validate(&value) {
        Ok(cmd) => { observability::log_validation_result(true); cmd }
        Err(e) => { observability::log_validation_result(false); observability::log_error("validate", &format!("{e:?}")); return Err((emit_validation_error(&e), 1)); }
    };
    observability::log_window_open();
    let result = match wyvern_window::run(command) {
        Ok(r) => { observability::log_window_close(); r }
        Err(e) => { observability::log_error("run", &format!("{e:?}")); return Err(handle_run_failure(&e)); }
    };
    observability::log_result_emitted();
    Ok(emit_stdout(&result))
}

// crates/wyvern/src/main.rs
fn main() {
    observability::init().ok();
    observability::log_process_start();
    // load → run_from_loaded → stdout / stderr + exit
}
```

Event keys are normative; `sc-observability` symbol names may differ if behavior matches.

## This Sprint Does Not Close

- Logging inside `wyvern-schema`, `wyvern-window`, or other libs (rg gate: no observability in non-binary crates)
- `--interactive` / MCP logging (Phase E)
- Phase CI matrix definition (owned by [README.md](README.md#ci-validation-authoritative))

## Acceptance Criteria

- `cargo build -p wyvern` with crates.io dep only
- `WYVERN_LOG=debug` emits events on chrome path
- **No observability in non-binary crates:** `rg 'sc_observability' crates/wyvern-schema crates/wyvern-window crates/wyvern-wizard crates/wyvern-mcp` → empty (`wyvern-mcp` listed as a lib crate boundary check, not MCP server work)
- Linux CI uses `xvfb-run` per README CI section
- `docs/observability.md` documents version pin + event list + pipeline hook map

## Required Validation

- `cargo build -p wyvern`
- `rg 'path.*sc-observability' Cargo.toml crates/` → empty
- `rg 'sc_observability' crates/wyvern-schema crates/wyvern-window crates/wyvern-wizard crates/wyvern-mcp` → empty (non-binary crate boundary gate)
- CI matrix: [README.md — CI validation](README.md#ci-validation-authoritative)
- `cargo clippy --workspace -- -D warnings`
