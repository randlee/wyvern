---
id: S1.2b
title: JSON schema validation (chrome only)
status: planned
branch: feature/p1-s2b-validation
target: integrate/phase-A
---

# Sprint S1.2b — JSON schema validation (`chrome` only)

## Goal

- Implement `wyvern-schema` validation for Phase 1 executable surface only. Reject unimplemented types before any window code runs.

## Hard Dependencies

- S1.2a JSON loading

## Exact Targets

- `wyvern-schema/src/lib.rs`
- `wyvern-schema/src/validate.rs`
- `wyvern-schema/src/command.rs`
- `wyvern-schema/src/error.rs`
- `wyvern-schema/tests/validation_chrome.rs`

## Deliverables

- `Command::Chrome { title, status }` enum variant
- `validate(value: &Value) -> Result<Command, ValidationError>`
- Structured stderr errors: `parse`, `validation`, `state` per REQ-0069–REQ-0072
- CLI wiring: load → validate → print error and exit (no window on failure)

## Required Work

- Accept `{"type":"chrome","title":"..."}` (+ optional `status`)
- Reject `type: message` and all other dialog types with explicit `type` field error
- Reject unknown fields on `chrome`
- Reject `{"action":"show"}` etc. outside `--interactive` (REQ-0060)
- Unit test every rule; no integration test requiring window

## Explicit Code Samples

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Chrome { title: String, status: Option<String> },
}

#[derive(Debug)]
pub struct ValidationError {
    pub error: &'static str, // "validation" | "parse" | "state"
    pub field: Option<String>,
    pub message: String,
}

pub fn validate(value: &Value) -> Result<Command, ValidationError>;
```

## This Sprint Does Not Close

- Opening a window for valid `chrome`
- Validating `message`/`input`/`markdown` field shapes (Phase 2 per type)
- Levenshtein enum suggestions (until enum fields ship with dialog types)

## Acceptance Criteria

- Valid `chrome` passes; invalid inputs fail with correct `error` kind on stderr
- `message` input fails validation without opening a window
- Exit code non-zero on all failures
- 100% of listed validation rules covered by `wyvern-schema` unit tests

## Required Validation

- `cargo test -p wyvern-schema`
- `cargo build --workspace`
- `cargo clippy --workspace -- -D warnings`
- CLI: `wyvern '{"type":"message","title":"t","message":"m","buttons":"ok"}'` → stderr JSON, exit ≠ 0
