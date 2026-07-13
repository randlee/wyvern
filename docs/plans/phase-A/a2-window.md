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
- `crates/wyvern-window/src/error.rs` (`RunError` — production type from a.2)
- `crates/wyvern-window/tests/blank_window.rs` (or `#[cfg(test)]` module)

## Deliverables

- Production `RunError` in `crates/wyvern-window/src/error.rs` (re-exported from `lib.rs` for a.5 CLI matching)
- `#[cfg(test)]` helper `open_blank_window_for_test() -> Result<(), RunError>` in `crates/wyvern-window/tests/support.rs` — **not** exported from `lib.rs`
- macOS transparent title bar + full-size content view (ADR-0010)
- Integration test opens window, closes via API/event, asserts `Ok(())` (dismissed)

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
```

a.5 absorbs this `RunError` and maps OS close → `CommandResult::Chrome { button: "dismissed" }`. No second `RunError` or `CloseReason` in a.5.

## Phase A CI policy (window tests)

Window integration tests are **macOS-only** until Phase C. Non-macOS CI legs skip them (see a.6 CI sample). Local validation on Linux/Windows uses the cfg gate below.

## Explicit Code Samples (test gate)

```rust
// crates/wyvern-window/tests/blank_window.rs
#[cfg(target_os = "macos")]
#[test]
fn blank_window_dismisses() { /* ... */ }

#[cfg(not(target_os = "macos"))]
#[test]
fn blank_window_skipped_on_non_macos() {
    // intentional no-op — CI ubuntu/windows legs pass without webview
}
```

## This Sprint Does Not Close

- Any change to `crates/wyvern/src/main.rs` beyond a.1 usage stub
- JSON loading, validation, stdout emission
- Public `wyvern_window::run` (a.5) — a.2 does not export any window entry point from `lib.rs`

## Acceptance Criteria

- `cargo test -p wyvern-window -- blank_window` (or named test) passes on macOS
- `Ok(())` from test helper on OS close (maps to `CommandResult` in a.5)
- No `wyvern` CLI subcommand added for window testing

## Required Validation

- **macOS:** `cargo test -p wyvern-window -- blank_window`
- **Linux/Windows:** `cargo test -p wyvern-window` passes via `#[cfg(not(target_os = "macos"))]` no-op test (see Explicit Code Samples)
- CI: follow Phase A CI policy in [a6-sc-observability.md](a6-sc-observability.md) — window tests run only on `macos-latest`
- `cargo build --workspace`
- `cargo clippy --workspace -- -D warnings`
