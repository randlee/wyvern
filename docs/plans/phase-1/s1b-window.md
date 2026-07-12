---
id: S1.1b
title: Native window opens and closes (macOS)
status: planned
branch: feature/p1-s1b-window
target: integrate/phase-A
---

# Sprint S1.1b — Native window opens and closes (macOS)

## Goal

- Prove `winit` + `wry` integration in `wyvern-window` with a minimal API — no JSON CLI yet.

## Hard Dependencies

- S1.1a workspace scaffold

## Exact Targets

- `wyvern-window/src/lib.rs`
- `wyvern-window/src/window.rs` (or equivalent)
- `wyvern/src/main.rs` (add `--window-demo` flag only)

## Deliverables

- `open_blank_window() -> Result<CloseReason, WindowError>` public API in `wyvern-window`
- macOS transparent title bar + full-size content view (ADR-0010)
- `--window-demo` subcommand on `wyvern` binary that calls the API

## Required Work

- Event loop + webview lifecycle in `wyvern-window`
- Map OS close to `CloseReason::Dismissed` enum variant
- Integration test or manual gate documented in validation

## Explicit Code Samples

```rust
pub enum CloseReason {
    Dismissed,
}

pub fn open_blank_window() -> Result<CloseReason, WindowError>;
```

## This Sprint Does Not Close

- JSON loading or validation
- HTML chrome frame content
- Windows/Linux builds
- stdout JSON emission

## Acceptance Criteria

- `cargo run -p wyvern -- --window-demo` opens a blank native window on macOS
- Window closes without panic on OS × button
- `CloseReason::Dismissed` returned to caller
- No JSON parsing in this sprint's code path

## Required Validation

- `cargo build --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
- Manual: `cargo run -p wyvern -- --window-demo` (document result in PR)
