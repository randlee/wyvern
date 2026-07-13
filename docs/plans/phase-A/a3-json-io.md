---
id: a.3
title: CLI arg detection and JSON loading
status: planned
branch: feature/phase-A-a3-json-io
target: integrate/phase-A
---

# Sprint a.3 — CLI arg detection and JSON loading

## Goal

- Four input loaders in `crates/wyvern` with `LoadError` discriminated union and stderr JSON emission.

## Hard Dependencies

- a.1 scaffold

## Exact Targets

- `crates/wyvern/src/main.rs`
- `crates/wyvern/src/input.rs`
- `crates/wyvern/src/error.rs` (`LoadError`, `emit_load_error`)

## Deliverables

- `load_command_input(args, stdin) -> Result<Value, LoadError>`
- CLI maps `LoadError` variants to stderr JSON (REQ-0069 parse, REQ-0071 io)
- Unit tests per input mode (no window)

## Required Work

- Inline JSON string arg
- `.json` file path → read + parse
- `.md` path → `{ "type": "markdown", "file": "<path>" }` shorthand (load only)
- **No arg → read stdin** (REQ-0004)
- Missing/ambiguous args → usage on stderr, exit ≠ 0 (`LoadError::Usage`)

**Usage argv shapes (authoritative):**

| Args | Result |
|------|--------|
| `wyvern` (no args, empty stdin) | `LoadError::Usage` |
| `wyvern arg1 arg2` (two+ positional args) | `LoadError::Usage` |
| `wyvern --unknown-flag` | `LoadError::Usage` |
| `wyvern file.json other.json` (two file paths) | `LoadError::Usage` |

Single arg (inline JSON, `.json`, or `.md`) and stdin-with-no-args remain valid loaders.

## Explicit Code Samples

```rust
pub enum LoadError {
    Parse { message: String },
    Io { field: String, message: String },
    Usage { message: String },
}

pub fn load_command_input(args: &[String], stdin: impl Read) -> Result<Value, LoadError>;

pub fn emit_load_error(err: &LoadError) -> String {
    match err {
        LoadError::Parse { message } => {
            format!(r#"{{"error":"parse","message":"{message}"}}"#)
        }
        LoadError::Io { field, message } => {
            format!(r#"{{"error":"io","field":"{field}","message":"{message}"}}"#)
        }
        LoadError::Usage { .. } => {
            // Usage: main prints plain stderr usage text + exit ≠ 0 — no JSON
            unreachable!("Usage handled in main, not emit_load_error")
        }
    }
}
```

## This Sprint Does Not Close

- Schema validation (`wyvern-schema`)
- Window open
- Executing loaded commands

## Acceptance Criteria

- Inline arg: valid JSON `Value` returned
- `.json` file: file contents parsed to `Value`
- `.md` file: `Value` with `type: "markdown"` and `file` path (not identical to inline payloads)
- Stdin (no args): reads and parses JSON
- Missing file: stderr `{ "error": "io", "field": "...", "message": "..." }`, exit ≠ 0
- Invalid JSON: stderr `{ "error": "parse", "message": "..." }`, exit ≠ 0
- Usage cases (table above): plain stderr usage text, exit ≠ 0, **no** JSON on stderr
- `wyvern` with no args and empty stdin → usage, exit ≠ 0
- `wyvern arg1 arg2` → usage, exit ≠ 0
- Loaders testable without opening a window

## Required Validation

- `cargo test -p wyvern -- input`
- `cargo build --workspace`
- `cargo clippy --workspace -- -D warnings`
