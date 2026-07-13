---
id: a.5
title: HTML chrome frame + chrome command E2E
status: planned
branch: feature/phase-A-a5-chrome-frame
target: integrate/phase-A
---

# Sprint a.5 — HTML chrome frame + `chrome` command E2E

## Goal

- Complete Phase A acceptance: validate → run → emit for `type: "chrome"`.

## Hard Dependencies

- a.2 window internals + production `RunError` (test-proven; absorbed into `run`)
- a.4 validation + `CommandResult` enum in schema

## Exact Targets

- `crates/wyvern-window/src/chrome/` (`template.html`, `render.rs`)
- `crates/wyvern-window/src/run.rs`
- `crates/wyvern/src/lib.rs`, `crates/wyvern/src/pipeline.rs` (testable CLI stages)
- `crates/wyvern/src/main.rs` (thin wrapper around `pipeline`)
- `crates/wyvern/src/error.rs` (`emit_run_error`, `emit_stdout`, `handle_run_failure`)

## Deliverables

- HTML chrome shell: title bar, content placeholder, optional status bar — **not** the full chrome frame (button bar deferred to Phase B)
- Sole public window **entry point**: `wyvern_window::run(command) -> Result<CommandResult, RunError>`
- Public re-exports from `wyvern-window` `lib.rs`: `run`, `RunError`, `CHROME_DEFAULT_*` / `CHROME_MAX_*` size constants — no other window-open helpers
- a.2 test helper remains `#[cfg(test)]` only; a.2 `RunError` is the only production run-error type
- Single `match` on `Command` with one `Chrome` arm (dispatch internal to `run`)
- Phase A AC #2 end-to-end
- CLI maps `RunError` → stderr JSON; `CommandResult` → stdout JSON; exit ≠ 0 on run failure only

## Required Work

- Title bar shows `title`; optional `status` in status bar; **status bar hidden when `status` absent**
- Empty `<div id="button-bar" hidden>` reserved in chrome HTML — populated in Phase B dialog sprints
- 72px macOS safe zone (REQ-0081); `-webkit-app-region: drag`
- OS close → stdout `{"button":"dismissed"}` via `CommandResult::Chrome(ChromeResult { button: "dismissed".into() })`
- **Fixed** open size **480×360px**; enforce **max 800×600px** bounds. Content-driven auto-size algorithms are out of scope until Phase B dialog content exists.

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

// wry loads chrome via data URL built from render_chrome_html() output (Phase A normative path)

// crates/wyvern-window/src/run.rs — sole public entry point
pub fn run(command: wyvern_schema::Command) -> Result<wyvern_schema::CommandResult, RunError> {
    dispatch(command)
}

// OS close in run_chrome returns:
// Ok(CommandResult::Chrome(ChromeResult { button: "dismissed".into() }))

// crates/wyvern/src/error.rs — structured stderr JSON (no format! interpolation)
pub fn emit_run_error(err: &wyvern_window::RunError) -> String {
    match err {
        RunError::WindowCreate { message } => {
            serde_json::json!({ "error": "window_create", "message": message }).to_string()
        }
        RunError::EventLoop { message } => {
            serde_json::json!({ "error": "event_loop", "message": message }).to_string()
        }
    }
}

pub fn emit_stdout(result: &wyvern_schema::CommandResult) -> String {
    serde_json::to_string(result).expect("CommandResult serializes")
}

/// Maps run failure to stderr line + non-zero exit code (testable without opening a window)
pub fn handle_run_failure(err: &wyvern_window::RunError) -> (String, i32) {
    (emit_run_error(err), 1)
}

// crates/wyvern/src/pipeline.rs — library entry for tests; main.rs calls this
pub fn run_from_loaded(value: serde_json::Value) -> Result<String, (String, i32)> {
    let command = wyvern_schema::validate(&value).map_err(|e| (emit_validation_error(&e), 1))?;
    let result = wyvern_window::run(command).map_err(|e| handle_run_failure(&e))?;
    Ok(emit_stdout(&result))
}
```

### RunError mapping tests (no cross-crate inject, no product flags)

```rust
// crates/wyvern/src/error.rs — unit tests colocated or in tests/emit_errors.rs
#[test]
fn emit_run_error_window_create_escapes_quotes() {
    let err = RunError::WindowCreate { message: r#"say "hi""#.into() };
    let json = emit_run_error(&err);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["error"], "window_create");
    assert_eq!(v["message"], r#"say "hi""#);
}

#[test]
fn handle_run_failure_returns_nonzero_exit() {
    let err = RunError::WindowCreate { message: "simulated".into() };
    let (stderr, code) = handle_run_failure(&err);
    assert!(stderr.contains("window_create"));
    assert_ne!(code, 0);
}

#[test]
fn emit_stdout_chrome_wire_shape() {
    let result = CommandResult::Chrome(ChromeResult { button: "dismissed".into() });
    assert_eq!(emit_stdout(&result), r#"{"button":"dismissed"}"#);
}
```

## This Sprint Does Not Close

- Button bar content and dialog buttons (Phase B)
- Dialog content rendering (Phase B)
- Content-driven window auto-size (Phase B+)
- Win/Linux chrome (Phase C)
- Interactive/MCP

## Acceptance Criteria

- `wyvern '{"type":"chrome","title":"Foundation"}'` opens chrome; OS close → `{"button":"dismissed"}`
- `wyvern '{"type":"message",...}'` still fails at validation (no window)
- Title bar reserves 72px left safe zone on macOS (visual/manual or DOM assertion)
- Window opens at fixed 480×360px; respects max 800×600px bounds (no content auto-size)
- Status bar not rendered when `status` omitted; empty hidden `#button-bar` present in DOM
- `handle_run_failure` unit tests assert stderr JSON + exit code ≠ 0 without window or CLI flags
- `wyvern-window` `lib.rs` exports `run` as sole entry point plus `RunError` and size constants only

## Required Validation

- `cargo test --workspace`
- Unit test: `emit_run_error` JSON shapes for each `RunError` variant (incl. escaped `"` in message)
- Unit test: `emit_stdout` asserts exact wire `{"button":"dismissed"}`
- Unit test: `handle_run_failure` non-zero exit mapping
- Manual E2E: Phase A acceptance criteria #1–#3 on macOS, Linux, and Windows (CI matrix)
- `cargo clippy --workspace -- -D warnings`
