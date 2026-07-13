---
id: b.2
title: Message level icons, icon/image fields, markdown body
status: pending
branch: feature/phase-B-b2-message-icons
target: integrate/phase-B
---

# Sprint b.2 — `message` level icons, icon/image fields, markdown body

## Goal

- Complete `message` type validation and rendering: `level`, `icon`, `image`, and `markdown: true` body.
- Ship **placeholder SVGs** for `level` values only — full curated icon set is Phase C (REQ-0030).

## Hard Dependencies

- b.1 message structure, buttons, IPC

## Exact Targets

- `crates/wyvern-schema/src/command.rs` — extend `Command::Message` with `level`, `icon`, `image`, `markdown`
- `crates/wyvern-schema/src/validate.rs` — unlock `level`, `icon`, `image`, `markdown` on message
- `crates/wyvern-schema/tests/validation_message.rs` — extended rules
- `crates/wyvern-window/src/message/render.rs` — icon/image/markdown body layout
- `crates/wyvern-window/src/message/template.html` — content area structure
- `crates/wyvern-window/assets/icons/placeholder/` — four level SVGs (`info`, `warning`, `error`, `question`)

## Deliverables

- `Command::Message` gains optional `level`, `icon`, `image`, and `markdown` fields (see code sample)
- `level` renders distinct placeholder icon per value (REQ-0012)
- `icon` field: named icon, file path, or base64 data URI (REQ-0031 subset — named resolves to placeholder set in b.2)
- `image` decorative body image (REQ-0032)
- `markdown: true` renders `message` via built-in HTML markdown renderer
- All field combinations render without layout breakage; auto-size still applies (REQ-0041; max 800×600)
- `message` type fully executable per README incremental surface table

## Required Work — rendering rules (authoritative)

### `level` icons (placeholder only)

| `level` | Placeholder asset | Notes |
|---------|-------------------|-------|
| `info` | `info.svg` | Distinct color/shape from others |
| `warning` | `warning.svg` | |
| `error` | `error.svg` | |
| `question` | `question.svg` | Distinct from `message` type name collision |

- Omit `level` → no level icon column
- `level` + `icon` → both may render; `icon` takes precedence in the level-icon layout slot when both present

### `icon` resolution (b.2 scope)

- Named: `"warning"` or `"warning:2"` syntax accepted; b.2 maps to placeholder files only
- File path: load from filesystem at render time; io failure → `RunError` before window open (path validated at load layer when applicable)
- Base64 data URI: embed inline in HTML

### `image` field

- Renders below/alongside message body per template; does not affect button IPC

### `markdown: true`

- `message` string passed through markdown→HTML converter (same engine reused by b.5/b.6)
- `markdown: false` (default) → plain text with HTML escaping

### Validation additions

- `level` wrong enum → REQ-0054 style error with valid options
- Invalid `icon`/`image` path → io error at load if path-based (not validation error on schema)

## Explicit Code Samples

```rust
// crates/wyvern-schema/src/command.rs — b.2 extends Message
pub enum MessageLevel {
    Info,
    Warning,
    Error,
    Question,
}

pub enum Command {
    // ...
    Message {
        title: ChromeTitle,
        message: String,
        status: Option<ChromeStatus>,
        buttons: ButtonsPreset,
        custom_buttons: Option<Vec<String>>,
        default_button: Option<u32>,
        level: Option<MessageLevel>,
        icon: Option<String>,   // named, path, or data URI
        image: Option<String>,  // same resolution forms as icon
        markdown: bool,         // default false
    },
}
```

```json
{
  "type": "message",
  "title": "Disk space",
  "message": "**Low** on volume `/data`",
  "level": "warning",
  "markdown": true,
  "buttons": "ok_cancel"
}
```

```html
<!-- message template content area (conceptual) -->
<div id="content">
  <div id="level-icon"><!-- placeholder SVG or icon winner --></div>
  <div id="body"><!-- markdown HTML or plain text --></div>
  <img id="decorative-image" /><!-- when image set -->
</div>
```

## This Sprint Does Not Close

- Full shipped icon bundle with variants (Phase C c.1, REQ-0030)
- `input`, `markdown`, `question` types
- AI-generated icons (post-MVP)

## Acceptance Criteria

- Each `level` value renders a visually distinct placeholder SVG
- `icon` accepts named, path, and base64 forms
- `image` renders decorative body image without breaking button bar layout
- `markdown: true` renders formatted markdown in body
- `level` + `icon` together: `icon` wins the level-icon slot; body layout unchanged
- Combinations (`level` + `markdown` + `image` + `icon`) render without layout breakage
- b.1 button/IPC/dismiss behavior unchanged

## Required Validation

- `cargo test --workspace -- --test-threads=1`
- `cargo test -p wyvern-schema` — level enum, markdown flag, icon field presence
- Render unit tests: markdown output contains expected tags; placeholder SVG referenced per level
- Visual/manual spot-check on macOS optional; CI uses DOM/assertion hooks where possible
