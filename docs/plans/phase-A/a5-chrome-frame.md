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

- a.2 window internals + production `RunError`
- a.4 validation + `CommandResult` enum in schema

## Exact Targets

- `crates/wyvern-window/src/chrome/template.html`
- `crates/wyvern-window/src/chrome/render.rs`
- `crates/wyvern-window/src/run.rs`
- `crates/wyvern/src/lib.rs`, `crates/wyvern/src/pipeline.rs`
- `crates/wyvern/src/main.rs` (thin wrapper)
- `crates/wyvern/src/error.rs` (`emit_run_error`, `emit_stdout`, `handle_run_failure`)

## Deliverables

- HTML chrome shell per **Required Work — chrome behavior** below
- Sole public window **entry point**: `wyvern_window::run(command) -> Result<CommandResult, RunError>`
- Public re-exports: `run`, `RunError`, `CHROME_DEFAULT_*` / `CHROME_MAX_*` — no other window-open helpers
- `pipeline.rs` owns load → validate → run → emit; each stage failure exits ≠ 0
- Phase A AC #2 wiring (see README manual gates)

## Required Work — chrome behavior (authoritative)

- Title bar shows `title`; optional `status` in status bar; **status bar hidden when `status` absent**
- Empty `<div id="button-bar" hidden>` — populated Phase B
- macOS: 72px left safe zone (REQ-0081); `-webkit-app-region: drag` on title bar
- Win/Linux Phase A: native OS decorations (see README platform policy); OS close → dismissed
- OS close → stdout `{"button":"dismissed"}` via `CommandResult::Chrome(ChromeResult { button: "dismissed" })`
- Fixed open **480×360px**; max **800×600px**; no content auto-size until Phase B
- wry loads chrome via **data URL** from `render_chrome_html()`

## Explicit Code Samples

```html
<!-- crates/wyvern-window/src/chrome/template.html -->
<!DOCTYPE html>
<html>
<head>
  <style>
    #title-bar { -webkit-app-region: drag; padding-left: 72px; }
    #button-bar[hidden] { display: none; }
    #status-bar:empty { display: none; }
  </style>
</head>
<body>
  <div id="title-bar">{{TITLE}}</div>
  {{STATUS_BLOCK}}
  <div id="content"></div>
  <div id="button-bar" hidden></div>
</body>
</html>
```

```rust
// RunError + size constants: see a.2 error.rs (do not redefine here)

// crates/wyvern-window/src/chrome/render.rs
const CHROME_HTML: &str = include_str!("template.html");

pub fn render_chrome_html(title: &str, status: Option<&str>) -> String {
    let status_block = status
        .map(|s| format!(r#"<div id="status-bar">{s}</div>"#))
        .unwrap_or_default();
    CHROME_HTML.replace("{{TITLE}}", title).replace("{{STATUS_BLOCK}}", &status_block)
}

// crates/wyvern-window/src/run.rs
pub fn run(command: wyvern_schema::Command) -> Result<wyvern_schema::CommandResult, RunError>;

// crates/wyvern/src/error.rs
pub fn emit_run_error(err: &wyvern_window::RunError) -> String { /* serde_json::json! */ }
pub fn emit_stdout(result: &wyvern_schema::CommandResult) -> String {
    serde_json::to_string(result).expect("CommandResult serializes")
}
pub fn handle_run_failure(err: &wyvern_window::RunError) -> (String, i32) {
    (emit_run_error(err), 1)
}

// crates/wyvern/src/pipeline.rs
pub fn run_from_loaded(value: serde_json::Value) -> Result<String, (String, i32)> {
    let command = wyvern_schema::validate(&value).map_err(|e| (emit_validation_error(&e), 1))?;
    let result = wyvern_window::run(command).map_err(|e| handle_run_failure(&e))?;
    Ok(emit_stdout(&result))
}
```

### Unit tests (automated)

```rust
#[test]
fn emit_stdout_chrome_wire_shape() {
    let result = CommandResult::Chrome(ChromeResult { button: "dismissed".into() });
    assert_eq!(emit_stdout(&result), r#"{"button":"dismissed"}"#);
}
```

## This Sprint Does Not Close

- Button bar content (Phase B)
- Win/Linux `decorations: false` chrome (Phase C)
- Interactive/MCP

## Acceptance Criteria

- All Required Work chrome behaviors implemented
- `handle_run_failure` unit tests pass (stderr JSON + exit ≠ 0 mapping)
- `lib.rs` exports `run` + `RunError` + size constants only

## Required Validation

- `cargo test --workspace` (automated; see README CI validation)
- Unit tests: `emit_run_error`, `emit_stdout`, `handle_run_failure`
- **Manual** (per README): chrome open/close gates #1–#3 on macOS, Linux, Windows before phase merge
