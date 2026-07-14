---
id: c.8
title: Clippy deny unauthorized panics in lib src
status: complete
branch: feature/phase-C-c8-clippy-deny-unwrap
worktree: /Volumes/Extreme Pro/github/wyvern-worktrees/feature/phase-C-c8-clippy-deny-unwrap
target: integrate/phase-C-fixes
---

# Sprint c.8 — Clippy deny unauthorized panics (regression gate)

> **Historical** — denied panics in deleted `wyvern-window` lib (merged c.8). Crate removed c.9; c.10+ extends deny to `wyvern-host` / `wyvern` roots.

## Goal

- Deny production `unwrap`/`expect`/`panic`/`unreachable` in **`wyvern`**, **`wyvern-schema`**, **`wyvern-window`** library roots **and** `crates/wyvern/src/main.rs` (binary root; policy includes main)
- Document panic policy in `docs/linting.md` (Clippy denies — not `.sc-lint.toml`).

## Hard Dependencies

- c.6 merged (production paths are `Result`-based)

## sc-lint note (verified)

`sc-lint` 0.4.x has **no** panic-detection config in `.sc-lint.toml`. Regression gate = crate `#![deny(clippy::...)]` + existing `cargo clippy -D warnings` in CI.

## Exact Targets

- `crates/wyvern/src/lib.rs`
- `crates/wyvern/src/main.rs` — same deny block as lib roots (binary is production code)
- `crates/wyvern-schema/src/lib.rs`
- `crates/wyvern-window/src/lib.rs`
- `docs/linting.md`

## Deliverables

```rust
// Each of wyvern lib.rs, wyvern main.rs, wyvern-schema lib.rs, wyvern-window lib.rs
#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable,
        clippy::todo,
        clippy::unimplemented
    )
)]
```

- `#![allow(...)]` only inside `#[cfg(test)] mod tests` where needed
- `docs/linting.md` — **Panic policy** section (see [ERROR-HANDLING-PLAN.md](ERROR-HANDLING-PLAN.md) enforcement table)

## This Sprint Does Not Close

- `wyvern-wizard` / `wyvern-mcp` — no lib-root deny in c.8 (no violations today); MCP REQ-0074 unchanged
- New `sc-lint` source-scan panic rule (future sc-lint feature)
- `sc-lint check native` scope change (compile-only; unchanged)

## Acceptance Criteria

- `cargo clippy --workspace -- -D warnings` clean with denies on **four** roots (three lib + `main.rs`)
- `sc-lint check native --config .sc-lint.toml` still passes
- `cargo test --workspace -- --test-threads=1` passes; test modules compile

## Required Validation

- `cargo clippy --workspace -- -D warnings`
- `sc-lint check native --config .sc-lint.toml`
- `cargo test --workspace -- --test-threads=1`
