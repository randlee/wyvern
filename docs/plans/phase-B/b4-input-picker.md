---
id: b.4
title: Input file and folder picker via rfd
status: pending
branch: feature/phase-B-b4-input-picker
target: integrate/phase-B
---

# Sprint b.4 — `input` file and folder picker via `rfd`

## Goal

- Unlock `mode: file` and `mode: folder` on `input` using native OS pickers via `rfd` in `wyvern-window` only.
- Complete REQ-0059 and REQ-0057 cross-field validation for input.
- Return paths as strings (single or array for `multiple: true`).

## Hard Dependencies

- b.3 input text mode

## Exact Targets

- `crates/wyvern-schema/src/validate.rs` — file/folder mode rules, REQ-0057, REQ-0059 complete
- `crates/wyvern-schema/tests/validation_input.rs` — picker field rules
- `crates/wyvern-window/Cargo.toml` — `rfd` dependency (window crate only)
- `crates/wyvern-window/src/input/picker.rs` — `rfd` integration
- `crates/wyvern-window/src/run.rs` — open picker before/instead of text field per mode
- `docs/wyvern-window/architecture.md` — file picker ADR (authoritative)

## Deliverables

- `mode: file` opens native file picker; returns selected path string
- `mode: folder` opens native folder picker; returns selected path string
- `filter: ["*.json", "*.txt"]` restricts file picker extensions (REQ-0015)
- `multiple: true` on file mode → JSON array in `input` field (REQ-0065)
- `start_path` sets initial picker directory (file/folder only — REQ-0059)
- REQ-0057: `multiline: true` + `mode: file|folder` → validation error
- REQ-0059 complete:
  - `filter` or `multiple` only when `mode: file`
  - `placeholder` or `default` only when `mode: text` (or omitted)
  - `start_path` only when `mode: file` or `mode: folder`
- Picker cancellation → `{"button":"cancel"}`; no `input` field

## Required Work — picker behavior (authoritative)

### Native picker (`rfd` ADR)

- `rfd` used **only** in `wyvern-window`; `wyvern-schema` never depends on `rfd`
- Paths returned as plain strings (absolute when OS provides absolute)
- File mode with `multiple: false` → single string in `input`
- File mode with `multiple: true` → `input` serializes as JSON array
- Folder mode ignores `filter` and `multiple` (validation error if present per REQ-0059)

### Headless CI strategy (authoritative)

- **All CI platforms:** picker integration tests set `WYVERN_MOCK_PICKER_PATH` to a fixture path; `picker.rs` reads this test-only env var and skips `rfd` UI when set.
- **Linux CI (xvfb):** mock env required for non-ignored picker tests; tests that require real picker UI without mock are `#[ignore]` with reason `requires native picker UI`.
- **macOS/Windows CI:** same mock env pattern; no real picker UI in CI matrix.
- Document mock hook in `picker.rs` module docs (one paragraph; no alternate strategies).

### UX flow (authoritative — picker-on-OK)

1. HTML renders message prompt + button bar only (no text field, no Browse button).
2. User clicks **OK** → page sends `{ "kind": "input_submitted", "button": "ok" }` without `value`.
3. Host opens `rfd` picker synchronously (`FileDialog` for file, folder dialog for folder).
4. **Selection:** host closes window and writes `{ "button": "ok", "input": "<path>" }` (or array when `multiple: true`).
5. **Picker cancel:** dialog stays open; no stdout until user submits Cancel or OS close.
6. **Cancel button:** `{ "button": "cancel" }` without `input`; no picker opened.

Host→page live IPC is not used in Phase B; picker runs entirely in Rust on `input_submitted`.

## Explicit Code Samples

```rust
// crates/wyvern-schema — b.4 unlocks picker fields on Input
pub enum Command {
    // ...
    Input {
        title: ChromeTitle,
        message: String,
        status: Option<ChromeStatus>,
        icon: Option<String>,
        markdown: bool,
        multiline: bool,
        placeholder: Option<String>,
        default: Option<String>,
        mode: InputMode,
        filter: Option<Vec<String>>,      // file mode only
        multiple: bool,                   // file mode only; default false
        start_path: Option<String>,       // file|folder only
        buttons: ButtonsPreset,
    },
}

// wyvern-window only — not in schema
use rfd::FileDialog;

pub fn pick_file(filter: &[String], multiple: bool, start_path: Option<&Path>) -> Option<Vec<PathBuf>> {
    // ...
}
```

```json
// validation error
{ "error": "validation", "field": "filter", "message": "filter is only valid when mode is file" }

// stdout — multi-select
{ "button": "ok", "input": ["/path/a", "/path/b"] }
```

## This Sprint Does Not Close

- Custom native picker styling
- Drag-and-drop path entry
- `wizard` / `question` types

## Acceptance Criteria

- File/folder mode renders message + button bar only (no text field)
- OK opens native picker; selected path returned on successful pick
- Picker cancel leaves dialog open (no premature stdout)
- `mode: file` returns selected file path on OK
- `mode: folder` returns selected folder path on OK
- `filter` restricts extensions; `multiple: true` returns array
- `start_path` opens picker at given directory
- `multiline: true` + file/folder → validation error (REQ-0057)
- `placeholder`/`default` + file/folder → validation error (REQ-0059)
- `filter`/`multiple` + text or folder (where invalid) → validation error
- `input` type fully executable for all modes per README table

## Required Validation

- `cargo test --workspace -- --test-threads=1`
- `cargo test -p wyvern-schema` — REQ-0057, REQ-0059 matrix
- Picker tests with `WYVERN_MOCK_PICKER_PATH` on ubuntu CI
- `sc-lint check native` confirms `rfd` only in `wyvern-window` boundary
