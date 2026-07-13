---
id: b.3
title: Input text mode validation and render
status: pending
branch: feature/phase-B-b3-input-text
target: integrate/phase-B
---

# Sprint b.3 — `input` text mode validation and render

## Goal

- Executable `type: "input"` for `mode: text` (default) with single-line and multiline fields.
- Wire `input_submitted` IPC per [ipc-dialog-contract.md](ipc-dialog-contract.md).
- Enforce text-mode cross-field rules (REQ-0059); reject file/folder modes until b.4.

## Hard Dependencies

- b.2 message complete (shared chrome, markdown renderer, icon placeholders)

## Exact Targets

- `crates/wyvern-schema/src/command.rs` — `Command::Input { ... }`
- `crates/wyvern-schema/src/validate.rs` — input text-mode validation + REQ-0059 subset
- `crates/wyvern-schema/src/result.rs` — `InputResult`, `CommandResult::Input`
- `crates/wyvern-schema/tests/validation_input.rs`
- `crates/wyvern-window/src/input/` — template, render, IPC handler
- `crates/wyvern-window/src/run.rs` — dispatch `Command::Input` (text only)

## Deliverables

- `Command::Input` with text-mode fields: `title`, `message`, optional `status`, optional `icon`, optional `markdown`, `multiline`, optional `placeholder`, optional `default`, `buttons`
- `mode` omitted or `mode: text` → executable; `mode: file` / `mode: folder` → validation error until b.4
- `InputResult { button: ButtonLabel, input: Option<String> }` → wire `{ "button": "ok", "input": "..." }` (REQ-0065)
- HTML: prompt text, single-line `<input>` or multiline `<textarea>`, button bar
- IPC `input_submitted` with `button` + `value`; cancel omits value
- Cross-field: `filter`, `multiple`, `start_path` rejected unless `mode: file`/`folder` (REQ-0059); at b.3 those modes are themselves rejected

## Required Work — text input behavior (authoritative)

### Validation

- `{"type":"input","title":"Name","message":"Enter name"}` passes (`mode` defaults text)
- `placeholder` and `default` allowed when `mode` omitted or `text`
- `filter`, `multiple`, `start_path` with `mode: text` or omitted → validation error (REQ-0059)
- `mode: file` or `mode: folder` → validation error: not implemented until b.4
- `multiline: true` with text mode → textarea rendered
- REQ-0057 not triggered until file/folder modes exist (b.4)

### Render + IPC

```json
{ "kind": "input_submitted", "button": "ok", "value": "user text" }
```

- `default` pre-fills field; `placeholder` shown as hint (empty value allowed on OK)
- `markdown: true` on `message` prompt uses shared markdown renderer from b.2
- OS close → `{ "button": "dismissed" }` without `input` field

### Window

- Modal attrs (REQ-0083); auto-size (REQ-0041)
- Platform chrome policy unchanged from b.1

## Explicit Code Samples

```rust
// crates/wyvern-schema/src/result.rs
#[derive(Serialize)]
pub struct InputResult {
    pub button: ButtonLabel,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<String>,
}
```

```json
// stdout — OK
{ "button": "ok", "input": "Ada Lovelace" }

// stdout — cancel
{ "button": "cancel" }
```

## This Sprint Does Not Close

- `mode: file` / `mode: folder` and native picker (b.4)
- `filter`, `multiple`, `start_path` (b.4, REQ-0059 file-mode rules)
- File picker ADR implementation

## Acceptance Criteria

- Single-line input renders; OK returns `{"button":"ok","input":"<value>"}`
- `multiline: true` renders textarea with same return shape
- `placeholder` displays as hint; `default` pre-fills
- Cancel → `{"button":"cancel"}` without `input`
- OS close → `{"button":"dismissed"}`
- `mode: file` / `folder` → validation stderr, exit ≠ 0, no window
- `filter`/`multiple`/`start_path` with text mode → validation error

## Required Validation

- `cargo test --workspace -- --test-threads=1`
- `cargo test -p wyvern-schema -- validation_input`
- Unit tests: REQ-0059 text-mode cross-field rejections
- IPC test: `input_submitted` → `InputResult` serialization
- README phase acceptance #2 subset (text input only)
