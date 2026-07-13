---
id: b.6
title: Markdown inline content and default stylesheet
status: pending
branch: feature/phase-B-b6-markdown-inline
target: integrate/phase-B
---

# Sprint b.6 — `markdown` inline `content` and default stylesheet

## Goal

- Complete `markdown` type: inline `content` field with no file required.
- Ship polished default stylesheet for markdown viewer (file and inline).
- REQ-0058 fully enforced: exactly one of `file` or `content`.

## Hard Dependencies

- b.5 markdown file rendering

## Exact Targets

- `crates/wyvern-schema/src/validate.rs` — unlock `content`; enforce mutual exclusion
- `crates/wyvern-schema/tests/validation_markdown.rs` — REQ-0058 matrix
- `crates/wyvern-window/src/markdown/template.html` — stylesheet link
- `crates/wyvern-window/src/markdown/styles.css` — default typography theme
- `crates/wyvern-window/src/markdown/render.rs` — inline content path

## Deliverables

- `{"type":"markdown","content":"# Hello\n\nBody"}` renders without file
- REQ-0058: exactly one of `file` or `content`; both or neither → validation error
- Stylesheet: readable typography, code block styling, responsive to window width
- `status` renders below content when provided
- Long documents scroll inside content area without resizing window beyond max bounds
- `markdown` type fully executable per README incremental surface table

## Required Work — inline markdown behavior (authoritative)

### Validation

| Case | Result |
|------|--------|
| `file` only | pass |
| `content` only | pass |
| both `file` and `content` | validation error |
| neither | validation error |
| empty `content` string | pass (renders empty viewer) |

### Stylesheet (default)

- System font stack or bundled web font
- Headings, paragraphs, lists, tables, blockquotes styled
- Code: monospace + subtle background; fenced blocks scroll horizontally when needed
- Max content width ~720px centered; window auto-size respects REQ-0041 caps
- Styles apply identically to file-loaded and inline content

### Scroll behavior

- `#content` region `overflow-y: auto`
- Window height capped; user scrolls for long inline strings
- Auto-size measures above-the-fold sensibly but does not expand to full document height

## Explicit Code Samples

```rust
// crates/wyvern-schema/src/command.rs — b.6 unlocks content
pub enum Command {
    // ...
    Markdown {
        title: Option<ChromeTitle>, // omitted → filename (file) or "Markdown" (inline)
        file: Option<String>,
        content: Option<String>, // exactly one of file|content required (REQ-0058)
        status: Option<ChromeStatus>,
        buttons: ButtonsPreset, // default Ok when omitted
    },
}
```

```json
{
  "type": "markdown",
  "content": "## Notes\n\n- item one\n- item two",
  "title": "Inline doc",
  "status": "Read-only",
  "buttons": "ok"
}
```

```css
/* crates/wyvern-window/src/markdown/styles.css — illustrative */
#markdown-body {
  max-width: 720px;
  margin: 0 auto;
  line-height: 1.6;
}
#markdown-body pre { overflow-x: auto; }
```

## This Sprint Does Not Close

- Custom themes / user CSS
- Mermaid or math rendering
- `question` type

## Acceptance Criteria

- `content` field renders inline markdown without file
- Stylesheet applied: typography, code blocks, responsive width
- `status` bar visible when provided
- Long content scrolls inside viewer; window does not grow unbounded
- REQ-0058 mutual exclusion tests pass
- File shorthand + JSON file form from b.5 still work unchanged

## Required Validation

- `cargo test --workspace -- --test-threads=1`
- `cargo test -p wyvern-schema` — REQ-0058 both/neither/none cases
- Render tests: inline `content` produces expected HTML elements
- README phase acceptance #3 covers both file and inline paths
