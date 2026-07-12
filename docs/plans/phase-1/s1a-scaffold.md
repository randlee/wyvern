---
id: S1.1a
title: Rust workspace scaffold (five crates)
status: planned
branch: feature/p1-s1a-scaffold
target: integrate/phase-A
---

# Sprint S1.1a — Rust workspace scaffold (five crates)

## Goal

- Establish the ADR-0011 five-crate workspace on day one so later sprints add logic inside fixed boundaries — no mid-phase crate split.

## Hard Dependencies

- None (first sprint)

## Exact Targets

- `Cargo.toml` (workspace root)
- `wyvern-schema/Cargo.toml`, `wyvern-schema/src/lib.rs`
- `wyvern-wizard/Cargo.toml`, `wyvern-wizard/src/lib.rs`
- `wyvern-window/Cargo.toml`, `wyvern-window/src/lib.rs`
- `wyvern/Cargo.toml`, `wyvern/src/main.rs`
- `wyvern-mcp/Cargo.toml`, `wyvern-mcp/src/lib.rs`

## Deliverables

- Workspace manifest with five member crates
- Pinned deps: `wry`, `winit`, `serde`, `serde_json`, `strsim` (where each crate needs them)
- Empty/stub `lib.rs` in `wyvern-wizard` and `wyvern-mcp`
- `wyvern` binary prints usage and exits 0

## Required Work

- Create crate directories and workspace `Cargo.toml`
- Wire dependency edges per ADR-0011; `wry`/`winit` only in `wyvern-window`
- `cargo build --workspace` clean on macOS

## Explicit Code Samples

Workspace dependency edges:

```toml
# wyvern/Cargo.toml
wyvern-window = { path = "../wyvern-window" }
wyvern-schema = { path = "../wyvern-schema" }

# wyvern-window/Cargo.toml
wyvern-schema = { path = "../wyvern-schema" }
wyvern-wizard = { path = "../wyvern-wizard" }
wry = "..."
winit = "..."
```

## This Sprint Does Not Close

- Window opening, JSON I/O, validation, chrome rendering
- `sc-observability`, `sc-lint`
- Any dialog `type` handling

## Acceptance Criteria

- `cargo build --workspace` succeeds with no warnings on macOS
- All five crates present and listed in workspace members
- `wyvern-wizard` and `wyvern-mcp` compile as stubs with no I/O
- `wry`/`winit` appear only in `wyvern-window/Cargo.toml`
- `cargo run -p wyvern` prints usage/help and exits 0

## Required Validation

- `cargo build --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
- `git diff --check`
