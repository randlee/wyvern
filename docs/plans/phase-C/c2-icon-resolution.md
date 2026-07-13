---
id: c.2
title: Full icon field resolution and validation
status: pending
branch: feature/phase-C-c2-icon-resolution
target: integrate/phase-C
---

# Sprint c.2 — Full icon field resolution and validation

## Goal

- Complete REQ-0031: named icons with variant index, file path, and base64 data URI.
- Unknown named icons → **validation error** with list of valid names (replacing b.2 run-time info placeholder fallback).
- Honor `"role:2"` variant syntax against c.1 asset catalog.

## Hard Dependencies

- c.1 production icon asset bundle and `icons` module

## Exact Targets

- `crates/wyvern-schema/src/icons.rs` — role catalog (`ROLES`, `variant_count`, `parse_icon_spec`) from c.1; shared by validation and render
- `crates/wyvern-window/src/icons/mod.rs` — `svg_markup(role, variant)` embed lookup (consumes schema catalog bounds)
- `crates/wyvern-window/src/message/media.rs` — remove unknown-name fallback; delegate named resolution to `icons`
- `crates/wyvern-schema/src/validate.rs` — validate `icon` on `message` and `input`; validate `image` on **`message` only** (`Command::Message` has `image`; `Command::Input` does not) when value is a named spec (not path/data URI)
- `crates/wyvern-schema/tests/validation_message.rs` — unknown icon, variant bounds, `image` named-icon cases
- `crates/wyvern-schema/tests/validation_input.rs` — icon field parity
- `crates/wyvern-window/src/input/render.rs` — uses shared named resolution (input supports `icon` field per REQ-0013)

## Deliverables

- `"warning"` → variant 1; `"warning:2"` → variant 2
- `"/path/to/icon.svg"` → load from disk at render time (unchanged b.2 behavior; `RunError` on io failure)
- `"data:image/..."` → inline `<img>` (unchanged)
- Unknown named icon (e.g. `"nonexistent"`) → `ValidationError` before window open, stderr lists `ROLES` from schema catalog
- Variant out of range (e.g. `"info:99"`) → validation error with valid range for that role
- **`message` only:** `image` field decorative resolution — named icons use same catalog; unknown named → validation error (input has no `image` field)
- Remove b.2 behavior: unknown named icon must **not** silently render info placeholder

## Required Work — resolution rules (authoritative)

### Named spec detection (validation layer)

A string is a **named icon spec** when it does **not**:
- start with `data:`
- contain `/` or `\`
- start with `.`
- have a filesystem extension (same heuristic as b.2 `looks_like_path`)

Named specs are validated in `wyvern-schema` against `crate::icons::ROLES` and `crate::icons::variant_count` (schema-local catalog — no `wyvern-window` import per ADR-0011).

### Variant index

- Omitted or `:1` → variant 1
- `:N` where N is 1-based integer within role's variant count
- Invalid N → validation error: `"icon variant 3 out of range for 'warning' (valid: 1–2)"`

### Level vs icon interaction (unchanged from b.2)

- `icon` wins `#level-icon` slot when both set
- `level` alone → production variant 1 for that level role

## Explicit Code Samples

```rust
// crates/wyvern-schema/src/validate.rs
use crate::icons;

fn validate_named_icon(field: &str, spec: &str) -> Result<(String, u32), ValidationError> {
    let (role, variant) = icons::parse_icon_spec(spec); // "warning:2" -> ("warning", 2)
    if !icons::ROLES.contains(&role.as_str()) {
        return Err(ValidationError::field(
            field,
            format!("unknown icon '{role}'; valid names: {}", icons::ROLES.join(", ")),
        ));
    }
    let max = icons::variant_count(&role);
    if variant < 1 || variant > max {
        return Err(ValidationError::field(
            field,
            format!("variant {variant} out of range for '{role}' (valid: 1–{max})"),
        ));
    }
    Ok((role, variant))
}

// message validation — `icon` on message + input; `image` on message only
if let Some(spec) = icon.as_deref() {
    if is_named_icon_spec(spec) {
        validate_named_icon("icon", spec)?;
    }
}
// validate_message only:
if let Some(spec) = image.as_deref() {
    if is_named_icon_spec(spec) {
        validate_named_icon("image", spec)?;
    }
}
```

```rust
// media.rs — named resolution after c.2
use wyvern_schema::icons;

fn resolve_named_icon_html(spec: &str) -> Result<IconHtml, RunError> {
    let (role, index) = icons::parse_icon_spec(spec); // validated at schema layer
    let svg = crate::icons::svg_markup(&role, index)
        .expect("schema validated variant exists");
    Ok(svg.to_string())
}
```

```json
// validation failure stdout (stderr)
{
  "error": "validation",
  "field": "icon",
  "message": "unknown icon 'nonexistent'; valid names: info, warning, error, question, success, loading"
}
```

## This Sprint Does Not Close

- Win/Linux platform chrome — c.3
- NFR pass — c.4
- Release — c.5
- AI-generated icons — post-MVP

## Acceptance Criteria

- `"warning"` → first variant; `"warning:2"` → second variant (visually distinct per c.1)
- Path and base64 forms unchanged from b.2
- Unknown named icon → validation stderr, exit ≠ 0, no window
- Out-of-range variant → validation stderr with valid range
- `icon` + `level` together: icon wins level-icon slot
- Input dialog `icon` field follows same rules as message
- Message `image` field: unknown named icon (e.g. `"nonexistent"`) → validation stderr with `"field": "image"`, exit ≠ 0, no window
- Message `image` field: out-of-range variant (e.g. `"success:99"`) → validation stderr with valid range for that role
- No code path renders info placeholder for unknown named icons

## Required Validation

- `cargo test --workspace -- --test-threads=1`
- `cargo test -p wyvern-schema` — `validation_message`, `validation_input` icon cases
- `cargo test -p wyvern-window` — variant selection, remove/update unknown-name fallback tests
- `sc-lint check native --config .sc-lint.toml`
- Grep gate: `placeholder_svg_for_level` not called for unknown named specs in production paths
