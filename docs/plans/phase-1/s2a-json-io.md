---
id: S1.2a
title: CLI arg detection and JSON loading
status: planned
branch: feature/p1-s2a-json-io
target: integrate/phase-A
---

# Sprint S1.2a — CLI arg detection and JSON loading

## Goal

- Implement the four input loaders as pure functions in `wyvern`; no window, no validation.

## Hard Dependencies

- S1.1a scaffold

## Exact Targets

- `wyvern/src/main.rs`
- `wyvern/src/input.rs` (or `wyvern-schema/src/load.rs` if shared — prefer `wyvern` for CLI-only loading)

## Deliverables

- `load_command_input(argv, stdin) -> Result<serde_json::Value, LoadError>`
- Usage text on stderr for ambiguous/missing input
- Unit tests for all four load modes

## Required Work

- Inline JSON string arg
- `.json` file path reads file contents as JSON
- `.md` path wraps `{ "type": "markdown", "file": "<path>" }` (loader only — type not executable until Phase 2)
- Empty argv reads stdin
- Non-zero exit on load failure

## Explicit Code Samples

```rust
pub enum InputSource {
    InlineArg,
    JsonFile(PathBuf),
    MarkdownFile(PathBuf),
    Stdin,
}

pub fn load_command_input(args: &[String], stdin: impl Read) -> Result<Value, LoadError>;
```

## This Sprint Does Not Close

- Schema validation
- Window open
- Executing any command type (including `markdown` shorthand)

## Acceptance Criteria

- All four input modes produce identical `Value` in unit tests
- `.md` shorthand sets `type: markdown` and `file` path
- Missing/ambiguous args → usage on stderr, exit ≠ 0
- Loaders callable without opening a window (unit tests only)

## Required Validation

- `cargo test -p wyvern -- input`
- `cargo build --workspace`
- `cargo clippy --workspace -- -D warnings`
