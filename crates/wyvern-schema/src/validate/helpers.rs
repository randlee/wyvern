//! Shared validation helpers and field constants.

use serde_json::{Map, Value};

use crate::error::ValidationError;

/// Known lifecycle actions that require `--interactive` (REQ-0060).
pub(super) const LIFECYCLE_ACTIONS: &[&str] = &["show", "hide", "exit"];

/// Allowed fields on a `chrome` command object.
pub(super) const CHROME_FIELDS: &[&str] = &["type", "title", "status"];

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
];

/// Allowed fields on an `input` command object (b.4 full surface).
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
    "mode",
    "filter",
    "multiple",
    "start_path",
    "buttons",
];

/// Allowed fields on a `markdown` command object (b.5 file subset).
pub(super) const MARKDOWN_FIELDS: &[&str] =
    &["type", "title", "file", "content", "status", "buttons"];

/// Allowed fields on a `question` command object (b.7).
pub(super) const QUESTION_FIELDS: &[&str] = &["type", "questions"];

/// Allowed fields on each question card.
pub(super) const QUESTION_CARD_FIELDS: &[&str] = &["question", "header", "options", "multiSelect"];

/// Allowed fields on each question option.
pub(super) const QUESTION_OPTION_FIELDS: &[&str] = &["label", "description", "preview"];

/// Max characters for `questions[].header` (REQ-0062).
pub(super) const QUESTION_HEADER_MAX_CHARS: usize = 12;

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
