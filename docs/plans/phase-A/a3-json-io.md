---
id: a.3
title: CLI arg detection and JSON loading
status: complete
branch: feature/phase-A-a3-json-io
worktree: /Volumes/Extreme Pro/github/wyvern-worktrees/feature/phase-A-a3-json-io
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

## Deliverables (authoritative checklist)

- `load_command_input(args, stdin) -> Result<Value, LoadError>`
- **Inline JSON** string arg → parsed `Value`
- **`.json` file** path → read + parse to `Value`
- **`.md` path** → `Value` with `type: "markdown"` and `file` path (load only)
- **No arg → read stdin** (REQ-0004)
- **Usage** argv shapes → `LoadError::Usage`, plain stderr usage text, exit ≠ 0 (no JSON):

| Args | Result |
|------|--------|
| `wyvern` (no args, empty stdin) | `LoadError::Usage` |
| `wyvern arg1 arg2` (two+ positional args) | `LoadError::Usage` |
| `wyvern --unknown-flag` | `LoadError::Usage` |
| `wyvern file.json other.json` (two file paths) | `LoadError::Usage` |

- `emit_load_error` maps `Parse`/`Io` to stderr JSON via `serde_json::json!`
- Unit tests per loader mode (no window)

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
            serde_json::json!({ "error": "parse", "message": message }).to_string()
        }
        LoadError::Io { field, message } => {
            serde_json::json!({ "error": "io", "field": field, "message": message }).to_string()
        }
        LoadError::Usage { .. } => unreachable!("Usage handled in main"),
    }
}
```

## This Sprint Does Not Close

- Schema validation (`wyvern-schema`)
- Window open
- Executing loaded commands

## Acceptance Criteria

- All Deliverables verified by `cargo test -p wyvern -- input`
- Load/parse/io failures exit ≠ 0 with correct stderr shapes
- Usage cases exit ≠ 0 with plain stderr (no JSON)

## Required Validation

- `cargo test -p wyvern -- input`
- Unit test: message containing `"` in parse/io paths still yields valid JSON stderr
- `cargo build --workspace`
- `cargo clippy --workspace -- -D warnings`
