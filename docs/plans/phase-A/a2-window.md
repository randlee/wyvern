---
id: a.2
title: Native window opens and closes
status: planned
branch: feature/phase-A-a2-window
target: integrate/phase-A
---

# Sprint a.2 — Native window opens and closes

## Goal

- Prove `winit` + `wry` in `wyvern-window` via crate tests on **macOS, Linux, and Windows** — **no product CLI path**.

## Hard Dependencies

- a.1 scaffold

## Exact Targets

- `crates/wyvern-window/src/lib.rs`
- `crates/wyvern-window/src/window.rs`
- `crates/wyvern-window/src/error.rs` (`RunError` — production type from a.2)
- `crates/wyvern-window/tests/support.rs`
- `crates/wyvern-window/tests/blank_window.rs`

## Deliverables

- Production `RunError` in `crates/wyvern-window/src/error.rs` (re-exported from `lib.rs` for a.5 CLI matching)
- `#[cfg(test)]` helper `open_blank_window_for_test() -> Result<(), RunError>` in `tests/support.rs` — **not** exported from `lib.rs`
- **Platform interim policy (Phase A):** macOS uses transparent title bar + full-size content (ADR-0010). **Windows/Linux use native OS window decorations** until Phase C (`decorations: false` + HTML close/minimize supersede ADR-0010a/REQ-0085 in Phase C only)
- Integration test opens window, closes via API/OS chrome, asserts `Ok(())` (dismissed) on all CI platforms

## Explicit Code Samples

```rust
// crates/wyvern-window/src/error.rs — production type (a.2); a.5 extends usage, does not redefine
pub enum RunError {
    WindowCreate { message: String },
    EventLoop { message: String },
}

// crates/wyvern-window/tests/support.rs — test-only
#[cfg(test)]
pub fn open_blank_window_for_test() -> Result<(), RunError>;

// crates/wyvern-window/tests/blank_window.rs
mod support;
#[test]
fn blank_window_dismisses() { /* open, close via OS chrome, assert Ok(()) */ }
```

a.5 absorbs this `RunError` and maps OS close → `CommandResult::Chrome(ChromeResult { button: "dismissed" })`.

## This Sprint Does Not Close

- Any change to `crates/wyvern/src/main.rs` beyond a.1 usage stub
- JSON loading, validation, stdout emission
- Public `wyvern_window::run` (a.5)
- Win/Linux `decorations: false` + HTML window controls (Phase C)

## Acceptance Criteria

- `cargo test -p wyvern-window -- blank_window` passes locally on macOS, Linux, and Windows
- `Ok(())` from test helper on OS close
- No `wyvern` CLI subcommand added for window testing

## Required Validation

- `cargo test -p wyvern-window -- blank_window`
- Phase CI matrix: see [README.md — CI validation](README.md#ci-validation-authoritative)
- `cargo build --workspace`
- `cargo clippy --workspace -- -D warnings`
