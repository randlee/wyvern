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

- `open_blank_window() -> Result<CloseReason, RunError>` in `wyvern-window`
- macOS transparent title bar + full-size content view (ADR-0010)
- Integration test opens window, closes via API/event, asserts `CloseReason::Dismissed`

## Explicit Code Samples

```rust
pub enum CloseReason {
    Dismissed,
}

pub enum RunError {
    WindowCreate { message: String },
    EventLoop { message: String },
}

pub fn open_blank_window() -> Result<CloseReason, RunError>;
```

## This Sprint Does Not Close

- Any change to `crates/wyvern/src/main.rs` beyond a.1 usage stub
- JSON loading, validation, stdout emission
- `--window-demo` or any CLI flag (forbidden — second execution path)

## Acceptance Criteria

- `cargo test -p wyvern-window -- blank_window` (or named test) passes on macOS
- `CloseReason::Dismissed` returned on OS close
- No `wyvern` CLI subcommand added for window testing

## Required Validation

- `cargo test -p wyvern-window`
- `cargo build --workspace`
- `cargo clippy --workspace -- -D warnings`
