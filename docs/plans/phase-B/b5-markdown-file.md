---
id: b.5
title: Markdown file rendering and .md shorthand
status: pending
branch: feature/phase-B-b5-markdown-file
target: integrate/phase-B
---

# Sprint b.5 — `markdown` file rendering and `.md` shorthand

## Goal

- Executable `type: "markdown"` with `file` field only.
- CLI shorthand: `wyvern my-doc.md` loads and renders the file.
- REQ-0058: exactly one of `file` or `content` — b.5 allows `file` only; `content` rejected until b.6.

## Hard Dependencies

- b.4 input complete (shared pipeline, markdown engine from b.2)

## Exact Targets

- `crates/wyvern/src/input.rs` — `.md` path detection / shorthand routing
- `crates/wyvern/src/main.rs` — argv dispatch for `.md` files
- `crates/wyvern-schema/src/command.rs` — `Command::Markdown { ... }`
- `crates/wyvern-schema/src/validate.rs` — markdown file-only validation
- `crates/wyvern-schema/src/result.rs` — `MarkdownResult`, `CommandResult::Markdown`
- `crates/wyvern-schema/tests/validation_markdown.rs`
- `crates/wyvern-window/src/markdown/` — template, render, scrollable viewer
- `crates/wyvern-window/src/run.rs` — dispatch `Command::Markdown`

## Deliverables

- `wyvern path/to/doc.md` equivalent to `{"type":"markdown","file":"path/to/doc.md"}`
- File read via `LoadError::Io` on missing/unreadable path (REQ-0071) — before window open
- `title` defaults to filename when omitted
- Optional `status` bar below chrome
- `buttons` defaults to `ok`; IPC same as message (`button_pressed` / `dismissed`)
- `MarkdownResult { button: ButtonLabel }` → `{ "button": "ok" }` (REQ-0064)
- Scrollable content area for long documents; window auto-size capped (REQ-0041)
- `content` field present → validation error until b.6

## Required Work — file markdown behavior (authoritative)

### Shorthand routing

- Arg ends with `.md` and is not valid inline JSON → treat as file path shorthand
- Shorthand and JSON form produce identical `Command::Markdown` after load

### Validation (b.5 subset)

- `{"type":"markdown","file":"doc.md"}` passes
- Neither `file` nor `content` → error
- Both `file` and `content` → error (REQ-0058)
- `content` alone → validation error: not implemented until b.6
- Unknown fields → error (REQ-0053)

### Render

- Read file UTF-8; markdown→HTML via shared renderer
- Default stylesheet: minimal readable typography (extended in b.6)
- Content area scrolls; outer window does not grow unbounded for long docs

## Explicit Code Samples

```rust
// crates/wyvern-schema/src/command.rs
pub enum Command {
    // ...
    Markdown {
        title: Option<ChromeTitle>, // omitted → filename default at load/validate
        file: Option<String>,   // b.5: required; content rejected until b.6
        content: Option<String>, // rejected at b.5
        status: Option<ChromeStatus>,
        buttons: ButtonsPreset, // default Ok when omitted
    },
}

// crates/wyvern/src/input.rs — conceptual
pub fn load_command_from_args(args: &[String]) -> Result<serde_json::Value, LoadError> {
    // if single arg ends with .md → build {"type":"markdown","file": arg}
}
```

```json
// stdout
{ "button": "ok" }

// validation — b.5
{ "error": "validation", "field": "content", "message": "content is not supported until inline markdown ships (b.6)" }
```

## This Sprint Does Not Close

- Inline `content` field (b.6)
- Polished default stylesheet (b.6)
- Syntax highlighting theme variants

## Acceptance Criteria

- `wyvern my-doc.md` opens viewer with rendered markdown
- JSON `{"type":"markdown","file":"path.md"}` behaves identically
- Headings, code blocks, tables, lists render
- `title` defaults to filename when omitted
- `buttons: "ok"` default; OK → `{"button":"ok"}`; OS close → `{"button":"dismissed"}`
- `content` field → validation error at b.5
- Missing file → io stderr, exit ≠ 0, no window

## Required Validation

- `cargo test --workspace -- --test-threads=1`
- `cargo test -p wyvern` — shorthand routing tests
- `cargo test -p wyvern-schema -- validation_markdown`
- README phase acceptance #3 (file form)
