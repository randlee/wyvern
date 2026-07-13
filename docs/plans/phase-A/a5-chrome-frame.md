---
id: a.5
title: HTML chrome frame + chrome command E2E
status: planned
branch: feature/phase-A-a5-chrome-frame
target: integrate/phase-A
---

# Sprint a.5 — HTML chrome frame + `chrome` command E2E (macOS)

## Goal

- Complete Phase A acceptance: validate → run → emit for `type: "chrome"`.

## Hard Dependencies

- a.2 window internals (test-proven; absorbed into `run`)
- a.4 validation + `CommandResult` in schema

## Exact Targets

- `crates/wyvern-window/src/chrome/`
- `crates/wyvern-window/src/run.rs`
- `crates/wyvern-window/src/error.rs` (`RunError`)
- `crates/wyvern/src/main.rs` (load → validate → run → stdout)
- `crates/wyvern/src/error.rs` (`emit_run_error`)

## Deliverables

- HTML chrome shell (title bar, content placeholder, optional status bar)
- **Only** public UI entry: `wyvern_window::run(command) -> Result<CommandResult, RunError>`
- a.2 test helper remains `#[cfg(test)]` only — not exported from `lib.rs`
- Single `match` on `Command` with one `Chrome` arm
- Phase A AC #2 end-to-end
- CLI maps `RunError` → stderr JSON; exit ≠ 0; no stdout success line on failure

## Required Work

- Title bar shows `title`; optional `status` in status bar; **status bar hidden when `status` absent**
- 72px macOS safe zone (REQ-0081); `-webkit-app-region: drag`
- OS close → stdout `{"button":"dismissed"}`
- Auto-size with Phase A caps: **max width 800px, max height 600px** (REQ-0041 defaults for foundation)

## Explicit Code Samples

```rust
// crates/wyvern-window/src/error.rs
pub enum RunError {
    WindowCreate { message: String },
    EventLoop { message: String },
}

pub const CHROME_MAX_WIDTH_PX: u32 = 800;
pub const CHROME_MAX_HEIGHT_PX: u32 = 600;

// crates/wyvern-window/src/run.rs — sole public UI entry
pub fn run(command: wyvern_schema::Command) -> Result<wyvern_schema::CommandResult, RunError>;

// internal dispatch (not a second public API)
fn dispatch(command: Command) -> Result<CommandResult, RunError> {
    match command {
        Command::Chrome { title, status } => run_chrome(title, status),
    }
}

// crates/wyvern/src/error.rs
pub fn emit_run_error(err: &wyvern_window::RunError) -> String {
    match err {
        RunError::WindowCreate { message } => {
            format!(r#"{{"error":"window_create","message":"{message}"}}"#)
        }
        RunError::EventLoop { message } => {
            format!(r#"{{"error":"event_loop","message":"{message}"}}"#)
        }
    }
}
```

## This Sprint Does Not Close

- Dialog content rendering (Phase B)
- Win/Linux chrome (Phase C)
- Interactive/MCP

## Acceptance Criteria

- `wyvern '{"type":"chrome","title":"Foundation"}'` opens chrome; OS close → `{"button":"dismissed"}`
- `wyvern '{"type":"message",...}'` still fails at validation (no window)
- Title bar reserves 72px left safe zone on macOS (visual/manual or DOM assertion)
- Window content area respects max 800×600px bounds (REQ-0041 Phase A defaults)
- Status bar not rendered when `status` omitted
- Simulated `RunError::WindowCreate` path emits `{"error":"window_create",...}` on stderr, exit ≠ 0
- `wyvern_window::lib.rs` exports only `run` as the public window API (no `open_blank_window`)

## Required Validation

- `cargo test --workspace`
- Unit test: `emit_run_error` JSON shapes for each `RunError` variant
- Manual E2E: Phase A acceptance criteria #1–#3 in `docs/plans/project-plan.md`
- `cargo clippy --workspace -- -D warnings`
