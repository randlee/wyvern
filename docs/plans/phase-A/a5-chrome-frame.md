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

- a.2 window internals + production `RunError` (test-proven; absorbed into `run`)
- a.4 validation + `CommandResult` enum in schema

## Exact Targets

- `crates/wyvern-window/src/chrome/` (`template.html`, `render.rs`)
- `crates/wyvern-window/src/run.rs`
- `crates/wyvern/src/main.rs` (load → validate → run → stdout)
- `crates/wyvern/src/error.rs` (`emit_run_error`, `emit_stdout`)

## Deliverables

- HTML chrome shell: title bar, content placeholder, optional status bar — **not** the full chrome frame (button bar deferred to Phase B)
- Sole public window **entry point**: `wyvern_window::run(command) -> Result<CommandResult, RunError>`
- Public re-exports from `lib.rs`: `run`, `RunError`, `CHROME_DEFAULT_*` / `CHROME_MAX_*` size constants — no other window-open helpers
- a.2 test helper remains `#[cfg(test)]` only; a.2 `RunError` is the only production run-error type
- Single `match` on `Command` with one `Chrome` arm (dispatch internal to `run`)
- Phase A AC #2 end-to-end
- CLI maps `RunError` → stderr JSON; `CommandResult` → stdout JSON; exit ≠ 0 on run failure only

## Required Work

- Title bar shows `title`; optional `status` in status bar; **status bar hidden when `status` absent**
- Empty `<div id="button-bar" hidden>` reserved in chrome HTML — populated in Phase B dialog sprints
- 72px macOS safe zone (REQ-0081); `-webkit-app-region: drag`
- OS close → stdout `{"button":"dismissed"}` via `CommandResult::Chrome { button: "dismissed".into() }`
- Default window size **480×360px** on open; auto-size capped at **max 800×600px** (REQ-0041 Phase A bounds)

## Explicit Code Samples

```rust
// crates/wyvern-window/src/error.rs — defined in a.2; a.5 uses same type
pub enum RunError {
    WindowCreate { message: String },
    EventLoop { message: String },
}

pub const CHROME_DEFAULT_WIDTH_PX: u32 = 480;
pub const CHROME_DEFAULT_HEIGHT_PX: u32 = 360;
pub const CHROME_MAX_WIDTH_PX: u32 = 800;
pub const CHROME_MAX_HEIGHT_PX: u32 = 600;

// crates/wyvern-window/src/chrome/render.rs — title/status binding contract
const CHROME_HTML: &str = include_str!("template.html");

pub fn render_chrome_html(title: &str, status: Option<&str>) -> String {
    let status_block = status
        .map(|s| format!(r#"<div id="status-bar">{s}</div>"#))
        .unwrap_or_default();
    CHROME_HTML
        .replace("{{TITLE}}", title)
        .replace("{{STATUS_BLOCK}}", &status_block)
}

// template.html lives beside render.rs; wry loads via html:// or data URL from render_chrome_html()

// crates/wyvern-window/src/run.rs — sole public entry point
pub fn run(command: wyvern_schema::Command) -> Result<wyvern_schema::CommandResult, RunError> {
    dispatch(command)
}

fn dispatch(command: Command) -> Result<CommandResult, RunError> {
    match command {
        Command::Chrome { title, status } => run_chrome(title, status),
    }
}

// OS close in run_chrome returns:
// Ok(CommandResult::Chrome { button: "dismissed".into() })

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

pub fn emit_stdout(result: &wyvern_schema::CommandResult) -> String {
    serde_json::to_string(result).expect("CommandResult serializes")
}

// main.rs success path:
// let result = wyvern_window::run(command)?;
// println!("{}", emit_stdout(&result));
```

### RunError simulation (tests only — no product flags)

```rust
// crates/wyvern-window/src/run.rs
#[cfg(test)]
pub(crate) static mut INJECT_RUN_ERROR: Option<RunError> = None;

#[cfg(test)]
pub fn inject_run_error_for_test(err: RunError) {
    unsafe { INJECT_RUN_ERROR = Some(err); }
}

// run() checks INJECT_RUN_ERROR under #[cfg(test)] before opening window

// crates/wyvern/tests/run_error_mapping.rs — asserts CLI mapping without product flags
#[test]
fn window_create_maps_to_stderr_json_and_nonzero() {
    wyvern_window::inject_run_error_for_test(RunError::WindowCreate {
        message: "simulated".into(),
    });
    // invoke internal error-mapping helper or test binary entry with injected error
    // assert stderr contains {"error":"window_create",...} and exit code ≠ 0
}
```

## This Sprint Does Not Close

- Button bar content and dialog buttons (Phase B)
- Dialog content rendering (Phase B)
- Win/Linux chrome (Phase C)
- Interactive/MCP

## Acceptance Criteria

- `wyvern '{"type":"chrome","title":"Foundation"}'` opens chrome; OS close → `{"button":"dismissed"}`
- `wyvern '{"type":"message",...}'` still fails at validation (no window)
- Title bar reserves 72px left safe zone on macOS (visual/manual or DOM assertion)
- Window opens at 480×360px default; content area respects max 800×600px bounds
- Status bar not rendered when `status` omitted; empty hidden `#button-bar` present in DOM
- `inject_run_error_for_test(WindowCreate{...})` + unit test asserts stderr `{"error":"window_create",...}` mapping and non-zero exit semantics (no CLI flag)
- `lib.rs` exports `run` as sole entry point plus `RunError` and size constants only

## Required Validation

- `cargo test --workspace`
- Unit test: `emit_run_error` JSON shapes for each `RunError` variant
- Unit test: `emit_stdout` serializes `CommandResult::Chrome { button: "dismissed" }` to one JSON line
- Unit test: `run_error_mapping` (inject path above)
- Manual E2E (macOS): Phase A acceptance criteria #1–#3 in `docs/plans/project-plan.md`
- `cargo clippy --workspace -- -D warnings`
