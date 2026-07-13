---
id: a.2
title: Native window opens and closes (macOS)
status: planned
branch: feature/phase-A-a2-window
target: integrate/phase-A
---

# Sprint a.2 — Native window opens and closes (macOS)

## Goal

- Prove `winit` + `wry` in `wyvern-window` via crate tests — **no product CLI path**.

## Hard Dependencies

- a.1 scaffold

## Exact Targets

- `crates/wyvern-window/src/lib.rs`
- `crates/wyvern-window/src/window.rs`
- `crates/wyvern-window/tests/blank_window.rs` (or `#[cfg(test)]` module)

## Deliverables

- `#[cfg(test)]` helper `open_blank_window_for_test() -> Result<CloseReason, RunError>` in `crates/wyvern-window/tests/support.rs` (or `src/test_support.rs`) — **not** exported from `lib.rs`
- macOS transparent title bar + full-size content view (ADR-0010)
- Integration test opens window, closes via API/event, asserts `CloseReason::Dismissed`

## Explicit Code Samples

```rust
// crates/wyvern-window/tests/support.rs — test-only, not pub in lib.rs
pub enum CloseReason {
    Dismissed,
}

pub enum RunError {
    WindowCreate { message: String },
    EventLoop { message: String },
}

#[cfg(test)]
pub fn open_blank_window_for_test() -> Result<CloseReason, RunError>;
```

## This Sprint Does Not Close

- Any change to `crates/wyvern/src/main.rs` beyond a.1 usage stub
- JSON loading, validation, stdout emission
- Public `wyvern_window::run` (a.5) — a.2 does not export any window entry point from `lib.rs`

## Acceptance Criteria

- `cargo test -p wyvern-window -- blank_window` (or named test) passes on macOS
- `CloseReason::Dismissed` returned on OS close
- No `wyvern` CLI subcommand added for window testing

## Required Validation

- `cargo test -p wyvern-window`
- `cargo build --workspace`
- `cargo clippy --workspace -- -D warnings`
