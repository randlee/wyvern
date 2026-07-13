---
id: b.1
title: Message structure, buttons, IPC, auto-size, modal attrs
status: complete
branch: feature/phase-B-b1-message-structure
worktree: /Volumes/Extreme Pro/github/wyvern-worktrees/feature/phase-B-b1-message-structure
target: integrate/phase-B
---

# Sprint b.1 — `message` structure, buttons, IPC, auto-size, modal attrs

## Goal

- First executable dialog type: `type: "message"` with plain-text body and button presets.
- Populate the Phase A empty `#button-bar`; wire [ipc-dialog-contract.md](ipc-dialog-contract.md) for `button_pressed` and `dismissed`.
- Begin window auto-size (REQ-0041) and enforce modal window attributes (REQ-0083).

## Hard Dependencies

- Phase A complete (`integrate/phase-A` merged): a.5 chrome E2E, a.4 validation, a.2 window internals

## Exact Targets

- `crates/wyvern-schema/src/command.rs` — `Command::Message { ... }`
- `crates/wyvern-schema/src/validate.rs` — message validation (text + buttons only)
- `crates/wyvern-schema/src/result.rs` — `MessageResult`, `CommandResult::Message`
- `crates/wyvern-schema/tests/validation_message.rs`
- `crates/wyvern-window/src/run.rs` — dispatch `Command::Message`
- `crates/wyvern-window/src/message/` — template, render, IPC handler
- `crates/wyvern-window/src/window.rs` — modal attrs + auto-size hooks
- `crates/wyvern-window/src/chrome/template.html` — `#button-bar` no longer always hidden
- `crates/wyvern/src/pipeline.rs` — unchanged shape; new variant flows through

## Deliverables

- `Command::Message` with fields: `title`, `message`, optional `status`, `buttons`, optional `custom_buttons`, optional `default_button`
- `MessageResult { button: ButtonLabel }` → wire `{ "button": "<label>" }` (REQ-0064)
- Validation unlocks `message` for execution; `level`, `icon`, `image`, `markdown` on message → validation error until b.2
- HTML message page: title bar, status bar (hidden when absent), text body, populated button bar
- IPC: `button_pressed` closes with mapped label; OS close → `ButtonLabel::dismissed()` per contract
- Window auto-size to content with **min 320×200** / **max 800×600** (replace Phase A fixed 480×360 for dialog types)
- Modal types: minimize and maximize disabled (REQ-0083)

## Required Work — message behavior (authoritative)

### Validation (b.1 subset)

- `{"type":"message","title":"T","message":"Hi","buttons":"ok"}` passes
- Missing `title` or `message` → validation error
- All button presets accepted: `ok`, `ok_cancel`, `yes_no`, `yes_no_cancel`, `retry_cancel`, `custom`
- `buttons: custom` without `custom_buttons` → error (REQ-0055)
- `custom_buttons` with non-`custom` `buttons` → error (REQ-0056)
- `default_button` out of range for active preset → validation error
- Unknown fields → error (REQ-0053)
- `level`, `icon`, `image`, `markdown: true` on message → validation error (deferred to b.2)

### Render + IPC

- Inject initial context per [ipc-dialog-contract.md](ipc-dialog-contract.md) host→page shape
- Button label mapping table in contract is authoritative for stdout `button` values
- `default_button` index receives keyboard focus on open
- Malformed IPC → log + fail-safe `dismissed` (contract error handling)

### Window policy

- macOS: transparent title bar + HTML chrome (ADR-0010); 72px safe zone (REQ-0081)
- Win/Linux Phase B: **native OS decorations** (see [README.md](README.md) platform policy); ADR-0010a deferred to Phase C
- Auto-size (REQ-0041 start): measure content with word-wrap; **min 320×200**, **max 800×600** (carry Phase A max; replace fixed 480×360 open size for dialog types)

## Explicit Code Samples

```rust
// crates/wyvern-schema/src/command.rs
pub enum ButtonsPreset {
    Ok,
    OkCancel,
    YesNo,
    YesNoCancel,
    RetryCancel,
    Custom,
}

pub enum Command {
    Chrome { title: ChromeTitle, status: Option<ChromeStatus> },
    Message {
        title: ChromeTitle,
        message: String,
        status: Option<ChromeStatus>,
        buttons: ButtonsPreset,
        custom_buttons: Option<Vec<String>>,
        default_button: Option<u32>,
    },
}

// crates/wyvern-schema/src/result.rs
#[derive(Serialize)]
pub struct MessageResult {
    pub button: ButtonLabel,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum CommandResult {
    Chrome(ChromeResult),
    Message(MessageResult),
}
```

```json
// IPC page → host
{ "kind": "button_pressed", "label": "ok" }

// stdout
{ "button": "ok" }
```

## This Sprint Does Not Close

- `level`, `icon`, `image`, `markdown` message body (b.2)
- `input`, `markdown`, `question` types
- Full icon set (Phase C)
- Win/Linux `decorations: false` (Phase C)

## Acceptance Criteria

- All button presets render and return correct stdout labels per ipc-dialog-contract mapping table
- `custom_buttons` renders verbatim; each label returned as-is on press
- `default_button` index focused on open
- OS close → `{"button":"dismissed"}`
- `level`/`icon`/`image`/`markdown` on message → validation stderr, exit ≠ 0, no window
- Modal attrs: minimize/maximize disabled on message window
- Window sizes to content within **min 320×200** and **max 800×600** (not fixed 480×360)

## Required Validation

- `cargo test --workspace -- --test-threads=1`
- `cargo test -p wyvern-schema -- validation_message`
- Unit tests: button label mapping, `default_button` bounds, preset/custom mutual exclusion
- IPC integration test: inject `button_pressed`, assert `MessageResult` wire shape
- `sc-lint check native --config .sc-lint.toml` (boundary rules reviewed at planning)
- Optional macOS manual smoke: README phase acceptance #1 (text-only message)
