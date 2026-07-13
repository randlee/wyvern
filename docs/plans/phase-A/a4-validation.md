---
id: a.4
title: JSON schema validation (chrome only)
status: planned
branch: feature/phase-A-a4-validation
target: integrate/phase-A
---

# Sprint a.4 — JSON schema validation (`chrome` only)

## Goal

- `wyvern-schema` validates Phase A executable surface only; wire load → validate in CLI.

## Hard Dependencies

- a.3 JSON loading

## Exact Targets

- `crates/wyvern-schema/src/lib.rs`
- `crates/wyvern-schema/src/command.rs`
- `crates/wyvern-schema/src/validate.rs`
- `crates/wyvern-schema/src/error.rs`
- `crates/wyvern-schema/src/result.rs` (`CommandResult` protocol type)
- `crates/wyvern-schema/tests/validation_chrome.rs`
- `crates/wyvern/src/main.rs` (load → validate → emit or exit)

## Deliverables

- `Command::Chrome { title, status }`
- `validate(value) -> Result<Command, ValidationError>` (no `PhaseSurface` param — surface is the `Command` enum at compile time)
- `ValidationError` enum: `Validation { field, message }`, `State { field, message }`
- `CommandResult::Chrome(ChromeResult { button })` with `#[serde(untagged)]` — wire `{ "button": "dismissed" }`
- CLI: on validation failure → stderr JSON + exit ≠ 0; **no window**

## Required Work — validation rules (authoritative checklist)

- `{"type":"chrome","title":"T"}` passes
- `{"type":"chrome"}` fails: missing `title`
- `{}` fails: missing `type` (field `type`)
- `{"title":"T"}` fails: missing `type`
- `{"type":null,"title":"T"}` fails: non-string `type`
- `{"type":1,"title":"T"}` fails: wrong type on `type`
- Unknown fields on `chrome` → validation error (REQ-0053)
- Wrong JSON field types → explicit expected/got message
- `{"type":"message",...}` → validation error on `type` (not implemented)
- `{"type":"unknown"}` → validation error on `type` (Phase AC #3)
- `{"action":"show"}` → state error (REQ-0060)
- Each rule has a named unit test in `wyvern-schema`

## Explicit Code Samples

```rust
pub enum Command {
    Chrome { title: String, status: Option<String> },
}

pub enum ValidationError {
    Validation { field: String, message: String },
    State { field: String, message: String },
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum CommandResult {
    Chrome(ChromeResult),
    // Phase B+: Input(InputResult), Wizard(WizardResult), etc.
}

#[derive(Serialize)]
pub struct ChromeResult {
    pub button: String,
}

// Phase A wire: {"button":"dismissed"} via #[serde(untagged)] + ChromeResult
// CommandResult is Serialize-only for stdout; overlapping {button} shapes across
// Phase B variants (message, markdown) are intentional wire compatibility.

pub fn validate(value: &serde_json::Value) -> Result<Command, ValidationError>;

// crates/wyvern — CLI boundary
pub fn emit_validation_error(err: &ValidationError) -> String {
    match err {
        ValidationError::Validation { field, message } => {
            serde_json::json!({ "error": "validation", "field": field, "message": message }).to_string()
        }
        ValidationError::State { field, message } => {
            serde_json::json!({ "error": "state", "field": field, "message": message }).to_string()
        }
    }
}
```

## This Sprint Does Not Close

- Opening a window (deferred to a.5)
- Parse/io errors (owned by a.3 `LoadError`)
- Dialog field validation (Phase B per type)

## Interim CLI behavior (a.4 only)

- On valid `Command::Chrome`: exit 0 **without** opening a window and **without** dispatch stubs
- a.5 wires `wyvern_window::run`

## Acceptance Criteria

- All Required Work validation rules covered by `wyvern-schema` unit tests
- CLI integration tests cover validation rules + interim exit-0 for valid chrome (no window until a.5)

## Required Validation

- `cargo test -p wyvern-schema`
- Unit test: validation stderr with `message` containing `"` is valid JSON
- `cargo test -p wyvern` (CLI validation integration)
- `cargo clippy --workspace -- -D warnings`
