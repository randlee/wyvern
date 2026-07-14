//! Shared validation helpers and field constants.

use serde_json::{Map, Value};

use crate::error::ValidationError;

/// Known lifecycle actions that require `--interactive` (REQ-0060).
pub(super) const LIFECYCLE_ACTIONS: &[&str] = &["show", "hide", "exit"];

/// Allowed fields on a `chrome` command object.
pub(super) const CHROME_FIELDS: &[&str] = &["type", "title", "status", "width", "height"];

/// Allowed fields on a `message` command object (b.2 full surface).
pub(super) const MESSAGE_FIELDS: &[&str] = &[
    "type",
    "title",
    "message",
    "status",
    "buttons",
    "custom_buttons",
    "default_button",
    "level",
    "icon",
    "image",
    "markdown",
    "width",
    "height",
];

/// Allowed fields on an `input` command object (b.4 full surface + c.11 password).
pub(super) const INPUT_FIELDS: &[&str] = &[
    "type",
    "title",
    "message",
    "status",
    "icon",
    "markdown",
    "multiline",
    "placeholder",
    "default",
    "password",
    "mode",
    "filter",
    "multiple",
    "start_path",
    "buttons",
    "width",
    "height",
];

/// Allowed fields on a `markdown` command object (b.5 file subset).
pub(super) const MARKDOWN_FIELDS: &[&str] = &[
    "type", "title", "file", "content", "status", "buttons", "width", "height",
];

/// Allowed fields on a `question` command object (b.7).
pub(super) const QUESTION_FIELDS: &[&str] = &["type", "questions", "width", "height"];

/// Allowed fields on each question card.
pub(super) const QUESTION_CARD_FIELDS: &[&str] = &["question", "header", "options", "multiSelect"];

/// Allowed fields on each question option.
pub(super) const QUESTION_OPTION_FIELDS: &[&str] = &["label", "description", "preview"];

/// Max characters for `questions[].header` (REQ-0062).
pub(super) const QUESTION_HEADER_MAX_CHARS: usize = 12;

/// Maximum UTF-8 byte length for markdown `content`.
///
/// Aligned with the host `/api/*` request body limit (256 KiB) so inline
/// markdown cannot exceed what the HTTP API accepts. Enforced at schema
/// validation, CLI file load, and host render (defense in depth).
pub const MARKDOWN_CONTENT_MAX_BYTES: usize = 256 * 1024;

/// Phase B executable `type` values (through b.7).
pub(super) const VALID_TYPES: &[&str] = &["chrome", "message", "input", "markdown", "question"];

pub(super) fn require_string_field(
    obj: &Map<String, Value>,
    field: &str,
) -> Result<String, ValidationError> {
    match obj.get(field) {
        None => Err(ValidationError::validation(
            field,
            format!("missing required field '{field}'"),
        )),
        Some(Value::String(s)) => Ok(s.clone()),
        Some(other) => Err(ValidationError::validation(
            field,
            format!(
                "field '{field}' expected string, got {}",
                json_type_name(other)
            ),
        )),
    }
}

pub(super) fn optional_string_field(
    obj: &Map<String, Value>,
    field: &str,
) -> Result<Option<String>, ValidationError> {
    match obj.get(field) {
        None => Ok(None),
        Some(Value::String(s)) => Ok(Some(s.clone())),
        Some(other) => Err(ValidationError::validation(
            field,
            format!(
                "field '{field}' expected string, got {}",
                json_type_name(other)
            ),
        )),
    }
}

/// Optional `icon` / `image` string with structural checks (no catalog).
///
/// Accepts paths, URLs, `data:` URIs, bare role names, and `role:index`.
/// Rejects empty strings and malformed named-spec shapes with field-scoped errors.
pub(super) fn optional_media_ref_field(
    obj: &Map<String, Value>,
    field: &str,
) -> Result<Option<crate::MediaRef>, ValidationError> {
    match optional_string_field(obj, field)? {
        None => Ok(None),
        Some(value) => {
            validate_media_ref(field, &value)?;
            Ok(Some(crate::MediaRef::new(value)))
        }
    }
}

fn validate_media_ref(field: &str, value: &str) -> Result<(), ValidationError> {
    if value.is_empty() || value.trim().is_empty() {
        return Err(ValidationError::validation(
            field,
            format!("field '{field}' must not be empty"),
        ));
    }
    if looks_like_opaque_media(value) {
        return Ok(());
    }
    // Named-style template hint: `role` or `role:index` (catalog deleted in c.9).
    if let Some((role, variant)) = value.split_once(':') {
        let role_ok = !role.is_empty()
            && role
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
        let variant_ok = !variant.is_empty() && variant.chars().all(|c| c.is_ascii_digit());
        if !role_ok || !variant_ok {
            return Err(ValidationError::validation(
                field,
                format!(
                    "invalid {field} spec '{value}': expected 'name' or 'name:index' with numeric index"
                ),
            ));
        }
    }
    Ok(())
}

fn looks_like_opaque_media(value: &str) -> bool {
    value.contains("://")
        || value.starts_with("data:")
        || value.starts_with('/')
        || value.starts_with("./")
        || value.starts_with("../")
        || value.contains('/')
        || value.contains('\\')
}

pub(super) fn optional_bool_field(
    obj: &Map<String, Value>,
    field: &str,
) -> Result<Option<bool>, ValidationError> {
    match obj.get(field) {
        None => Ok(None),
        Some(Value::Bool(b)) => Ok(Some(*b)),
        Some(other) => Err(ValidationError::validation(
            field,
            format!(
                "field '{field}' expected boolean, got {}",
                json_type_name(other)
            ),
        )),
    }
}

/// Minimum viewer width (CSS px) — matches `wyvern-viewer` resize clamp.
pub const VIEWER_WIDTH_MIN: u32 = 200;
/// Maximum viewer width (CSS px).
pub const VIEWER_WIDTH_MAX: u32 = 800;
/// Minimum viewer height (CSS px).
pub const VIEWER_HEIGHT_MIN: u32 = 96;
/// Maximum viewer height (CSS px).
pub const VIEWER_HEIGHT_MAX: u32 = 600;

pub(super) fn optional_window_size_fields(
    obj: &Map<String, Value>,
) -> Result<(Option<u32>, Option<u32>), ValidationError> {
    let width = optional_u32_field_bounded(obj, "width", VIEWER_WIDTH_MIN, VIEWER_WIDTH_MAX)?;
    let height = optional_u32_field_bounded(obj, "height", VIEWER_HEIGHT_MIN, VIEWER_HEIGHT_MAX)?;
    Ok((width, height))
}

pub(super) fn optional_u32_field_bounded(
    obj: &Map<String, Value>,
    field: &str,
    min: u32,
    max: u32,
) -> Result<Option<u32>, ValidationError> {
    match obj.get(field) {
        None => Ok(None),
        Some(Value::Number(n)) => {
            let v = n
                .as_u64()
                .and_then(|u| u32::try_from(u).ok())
                .ok_or_else(|| {
                    ValidationError::validation(
                        field,
                        format!("field '{field}' expected positive integer"),
                    )
                })?;
            if v < min || v > max {
                return Err(ValidationError::validation(
                    field,
                    format!("field '{field}' must be between {min} and {max} (got {v})"),
                ));
            }
            Ok(Some(v))
        }
        Some(other) => Err(ValidationError::validation(
            field,
            format!(
                "field '{field}' expected positive integer, got {}",
                json_type_name(other)
            ),
        )),
    }
}

pub(super) fn unknown_type_error(got: &str) -> ValidationError {
    let options = VALID_TYPES.join(", ");
    let mut message = format!("got '{got}', expected one of: {options}");
    if let Some(suggestion) = closest_match(got, VALID_TYPES) {
        message.push_str(&format!("; did you mean '{suggestion}'?"));
    }
    ValidationError::validation("type", message)
}

pub(super) fn closest_match<'a>(got: &str, options: &[&'a str]) -> Option<&'a str> {
    options
        .iter()
        .copied()
        .filter(|opt| strsim::levenshtein(got, opt) <= 2)
        .min_by_key(|opt| strsim::levenshtein(got, opt))
}

pub(super) fn json_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}
