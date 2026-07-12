---
id: S1.4
title: sc-observability integration
status: planned
branch: feature/p1-s4-sc-observability
target: integrate/phase-A
---

# Sprint S1.4 — sc-observability integration

## Goal

- Add structured logging at the `wyvern` binary entry point only, using the sibling `sc-observability` checkout.

## Hard Dependencies

- S1.3a chrome E2E path (events to log)

## Exact Targets

- `wyvern/Cargo.toml` (path dependency)
- `wyvern/src/main.rs`
- `docs/observability.md`

## Deliverables

- Path dependency: `sc-observability = { path = "../sc-observability" }`
- Log events: process start, command received, validation pass/fail, window open/close, result emitted, error
- `WYVERN_LOG` env var documented and wired
- `docs/observability.md` usage guidelines

## Required Work

- Document sibling repo layout in `docs/observability.md` and `CLAUDE.md` Environment section if missing
- CI note: Ubuntu/macOS jobs need `sc-observability` cloned beside wyvern (or skip obs build with feature flag — prefer clone)
- No `sc-observability` imports in `wyvern-window`, `wyvern-schema`, or other lib crates

## Explicit Code Samples

```toml
# wyvern/Cargo.toml
sc-observability = { path = "../sc-observability" }
```

```rust
// wyvern/src/main.rs only
fn main() {
    init_observability_from_env("WYVERN_LOG");
    // load → validate → run → emit
}
```

## This Sprint Does Not Close

- Logging inside library crates
- MCP or interactive mode logging semantics

## Acceptance Criteria

- With `../sc-observability` present, `cargo build -p wyvern` succeeds
- Structured logs emitted for chrome E2E path when `WYVERN_LOG=debug`
- `rg 'sc.observability|sc_observability' wyvern-schema wyvern-window wyvern-wizard wyvern-mcp` returns no matches
- `docs/observability.md` exists and documents env var + event list

## Required Validation

- `cargo build --workspace` (with sibling dep present)
- `cargo clippy --workspace -- -D warnings`
- `rg 'sc_observability' wyvern-schema wyvern-window` → empty
