---
id: a.1
title: Rust workspace scaffold (five crates)
status: planned
branch: feature/phase-A-a1-scaffold
target: integrate/phase-A
---

# Sprint a.1 — Rust workspace scaffold (five crates)

## Goal

- Establish ADR-0011 five-crate workspace under `crates/` on day one.

## Hard Dependencies

- None

## Exact Targets

- `Cargo.toml` (workspace root)
- `crates/wyvern-schema/Cargo.toml`, `crates/wyvern-schema/src/lib.rs`
- `crates/wyvern-wizard/Cargo.toml`, `crates/wyvern-wizard/src/lib.rs`
- `crates/wyvern-window/Cargo.toml`, `crates/wyvern-window/src/lib.rs`
- `crates/wyvern/Cargo.toml`, `crates/wyvern/src/main.rs`
- `crates/wyvern-mcp/Cargo.toml`, `crates/wyvern-mcp/src/lib.rs`

## Deliverables

- Workspace members: all five crates under `crates/`
- Pinned workspace deps: `wry`, `winit`, `serde`, `serde_json`, `strsim`
- Stub `wyvern-wizard` and `wyvern-mcp` as **library-only** crates (`lib.rs` only; **no** `[[bin]]` — MCP binary ships in Phase E)
- `wyvern` binary prints usage and exits 0

## Explicit Code Samples

```toml
# Cargo.toml
[workspace]
members = [
    "crates/wyvern",
    "crates/wyvern-schema",
    "crates/wyvern-window",
    "crates/wyvern-wizard",
    "crates/wyvern-mcp",
]

# crates/wyvern/Cargo.toml
wyvern-window = { path = "../wyvern-window" }
wyvern-schema = { path = "../wyvern-schema" }

# crates/wyvern-window/Cargo.toml
wyvern-schema = { path = "../wyvern-schema" }
wyvern-wizard = { path = "../wyvern-wizard" }
wry = { workspace = true }
winit = { workspace = true }

# crates/wyvern-wizard/Cargo.toml
wyvern-schema = { path = "../wyvern-schema" }

# crates/wyvern-mcp/Cargo.toml — library stub only (no [[bin]] until Phase E)
wyvern-window = { path = "../wyvern-window" }
wyvern-schema = { path = "../wyvern-schema" }
```

## This Sprint Does Not Close

- Window, JSON I/O, validation, chrome, observability, lint

## Acceptance Criteria

- `cargo build --workspace` succeeds on macOS with no warnings
- `wry`/`winit` only in `crates/wyvern-window/Cargo.toml`
- ADR-0011 edges present: `wyvern` → `{window, schema}`; `window` → `{schema, wizard}`; `wizard` → `schema`; `mcp` → `{window, schema}`; `schema` has no internal wyvern crate deps
- `cargo run -p wyvern` prints usage and exits 0

## Required Validation

- `cargo build --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
- `git diff --check`
- `rg 'wry|winit' crates/*/Cargo.toml` lists only `crates/wyvern-window/Cargo.toml`
- `rg 'wyvern-window|wyvern-wizard|wry|winit' crates/wyvern-schema/Cargo.toml` → empty
