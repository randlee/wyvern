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
- `crates/wyvern-window/tests/blank_window.rs` (or `#[cfg(test)]` module)

## Deliverables

- Production `RunError` in `crates/wyvern-window/src/error.rs` (re-exported from `lib.rs` for a.5 CLI matching)
- `#[cfg(test)]` helper `open_blank_window_for_test() -> Result<(), RunError>` in `crates/wyvern-window/tests/support.rs` — **not** exported from `lib.rs`
- macOS: transparent title bar + full-size content view (ADR-0010)
- Integration test opens window, closes via API/event, asserts `Ok(())` (dismissed) on all CI platforms

## Explicit Code Samples

```rust
// crates/wyvern-window/src/error.rs — production type (a.2); a.5 extends usage, does not redefine
pub enum RunError {
    WindowCreate { message: String },
    EventLoop { message: String },
}

// crates/wyvern-window/tests/support.rs — test-only, not pub in lib.rs
#[cfg(test)]
pub fn open_blank_window_for_test() -> Result<(), RunError>;

// crates/wyvern-window/tests/blank_window.rs — runs on all platforms (no #[cfg] skip)
#[test]
fn blank_window_dismisses() { /* open, close, assert Ok(()) */ }
```

a.5 absorbs this `RunError` and maps OS close → `CommandResult::Chrome(ChromeResult { button: "dismissed" })`. No second `RunError` or `CloseReason` in a.5.

## This Sprint Does Not Close

- Any change to `crates/wyvern/src/main.rs` beyond a.1 usage stub
- JSON loading, validation, stdout emission
- Public `wyvern_window::run` (a.5) — a.2 does not export any window entry point from `lib.rs`
- Win/Linux custom chrome decorations (Phase C)

## Acceptance Criteria

- `cargo test -p wyvern-window -- blank_window` passes on macOS, Linux, and Windows
- `Ok(())` from test helper on OS close (maps to `CommandResult` in a.5)
- No `wyvern` CLI subcommand added for window testing

## Required Validation

- `cargo test -p wyvern-window -- blank_window` (local + all CI matrix legs)
- CI: `cargo test --workspace` on `ubuntu-latest`, `macos-latest`, `windows-latest` (see a.6)
- Linux CI: `libwebkit2gtk-4.1-dev` install step (existing `.github/workflows/ci.yml`)
- `cargo build --workspace`
- `cargo clippy --workspace -- -D warnings`
