---
id: c.8
title: Clippy deny unauthorized panics in lib src
status: pending
branch: feature/phase-C-c8-clippy-deny-unwrap
target: integrate/phase-C-fixes
---

# Sprint c.8 — Clippy deny unauthorized panics (regression gate)

## Goal

- Deny production `unwrap()`, `expect()`, `panic!()`, `unreachable!()`, `todo!()`, `unimplemented!()` in **`wyvern`**, **`wyvern-schema`**, **`wyvern-window`** library `src/`.
- Document panic policy in `docs/linting.md` (Clippy denies — not `.sc-lint.toml`).

## Hard Dependencies

- c.6 merged (production paths are `Result`-based)

## sc-lint note (verified)

`sc-lint` 0.4.x has **no** panic-detection config in `.sc-lint.toml`. Regression gate = crate `#![deny(clippy::...)]` + existing `cargo clippy -D warnings` in CI.

## Exact Targets

- `crates/wyvern/src/lib.rs`
- `crates/wyvern-schema/src/lib.rs`
- `crates/wyvern-window/src/lib.rs`
- `docs/linting.md`

## Deliverables

```rust
// Each of the three lib.rs roots
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

- `wyvern-wizard` / `wyvern-mcp` — pure-logic / thin crates with no production panic violations today; extend denies in a future sprint if lib `src/` grows
- New `sc-lint` source-scan panic rule (future sc-lint feature)
- `sc-lint check native` scope change (compile-only; unchanged)

## Acceptance Criteria

- `cargo clippy --workspace -- -D warnings` clean with denies on three lib roots
- `sc-lint check native --config .sc-lint.toml` still passes
- `cargo test --workspace -- --test-threads=1` passes; test modules compile

## Required Validation

- `cargo clippy --workspace -- -D warnings`
- `sc-lint check native --config .sc-lint.toml`
- `cargo test --workspace -- --test-threads=1`
