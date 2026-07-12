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
- `docs/observability.md`
- `.github/workflows/ci.yml` (clone sibling before build)

## Deliverables

- Path dep: `sc-observability = { path = "../../sc-observability" }` in `crates/wyvern/Cargo.toml`
- Log events: start, command received, validation result, window open/close, result emitted, error
- `WYVERN_LOG` env var documented
- CI clones `sc-observability` beside repo root (documented in `docs/observability.md`)

## Explicit Code Samples

```toml
# crates/wyvern/Cargo.toml
sc-observability = { path = "../../sc-observability" }
```

```rust
// crates/wyvern/src/main.rs only — no lib crate imports
```

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
- `rg 'sc_observability' crates/wyvern-schema crates/wyvern-window` → empty
- `cargo clippy --workspace -- -D warnings`
