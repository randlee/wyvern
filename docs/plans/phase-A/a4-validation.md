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
- `CommandResult` for Phase A: `{ "button": "dismissed" }` shape defined in schema
- CLI: on validation failure → stderr JSON + exit ≠ 0; **no window**

## Required Work — validation rules (authoritative checklist)

- `{"type":"chrome","title":"T"}` passes
- `{"type":"chrome"}` fails: missing `title`
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

pub struct CommandResult {
    pub button: String,
}

pub fn validate(value: &serde_json::Value) -> Result<Command, ValidationError>;
```

## This Sprint Does Not Close

- Opening a window (deferred to a.5)
- Parse/io errors (owned by a.3 `LoadError`)
- Dialog field validation (Phase B per type)

## Interim CLI behavior (a.4 only)

- On valid `Command::Chrome`: exit 0 **without** opening a window and **without** dispatch stubs
- a.5 wires `wyvern_window::run`

## Acceptance Criteria

- All validation rules above covered by `wyvern-schema` unit tests
- CLI rejects invalid inputs with correct stderr `error` kind; no window
- `wyvern '{"type":"message",...}'` → validation stderr, exit ≠ 0
- `wyvern '{"type":"unknown"}'` → validation stderr, exit ≠ 0
- Valid chrome JSON → exit 0, no stdout result yet (until a.5)

## Required Validation

- `cargo test -p wyvern-schema`
- `cargo test -p wyvern` (CLI validation integration)
- `cargo clippy --workspace -- -D warnings`
