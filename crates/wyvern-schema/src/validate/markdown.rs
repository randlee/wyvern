//! Validate `markdown` commands.

use std::path::Path;

use serde_json::{Map, Value};

use crate::chrome::{ChromeStatus, ChromeTitle};
use crate::command::{ButtonsPreset, Command};
use crate::error::ValidationError;
use crate::field_name::FieldName;

use super::helpers::{
    closest_match, json_type_name, optional_string_field, optional_window_size_fields,
    MARKDOWN_CONTENT_MAX_BYTES, MARKDOWN_FIELDS,
};

pub(super) fn validate_markdown(obj: &Map<String, Value>) -> Result<Command, ValidationError> {
    for key in obj.keys() {
        let key_str = key.as_str();
        if !MARKDOWN_FIELDS.contains(&key_str) {
            return Err(ValidationError::validation(
                FieldName::new(key_str),
                format!("unknown field '{key_str}'"),
            ));
        }
    }

    let file = optional_string_field(obj, "file")?;
    let content = optional_string_field(obj, "content")?;

    // REQ-0058 — exactly one of file or content.
    match (file.as_ref(), content.as_ref()) {
        (None, None) | (Some(_), Some(_)) => {
            return Err(ValidationError::validation(
                "file",
                "markdown requires exactly one of 'file' or 'content'",
            ));
        }
        (Some(_), None) | (None, Some(_)) => {}
    }

    if let Some(body) = content.as_ref() {
        if body.len() > MARKDOWN_CONTENT_MAX_BYTES {
            return Err(ValidationError::validation(
                "content",
                format!(
                    "markdown content exceeds maximum of {MARKDOWN_CONTENT_MAX_BYTES} bytes (got {} bytes)",
                    body.len()
                ),
            ));
        }
    }

    let title = match optional_string_field(obj, "title")? {
        Some(t) => Some(ChromeTitle::new(t)),
        None => Some(match file.as_ref() {
            Some(path) => {
                let name = Path::new(path)
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| path.clone());
                ChromeTitle::new(name)
            }
            None => ChromeTitle::new("Markdown"),
        }),
    };
    let status = optional_string_field(obj, "status")?;

    let buttons = match obj.get("buttons") {
        None => ButtonsPreset::Ok,
        Some(Value::String(s)) => match ButtonsPreset::parse(s) {
            Some(preset) => preset,
            None => {
                let options = ButtonsPreset::all_names().join(", ");
                let mut msg = format!("got '{s}', expected one of: {options}");
                if let Some(suggestion) = closest_match(s, ButtonsPreset::all_names()) {
                    msg.push_str(&format!("; did you mean '{suggestion}'?"));
                }
                return Err(ValidationError::validation("buttons", msg));
            }
        },
        Some(other) => {
            return Err(ValidationError::validation(
                "buttons",
                format!(
                    "field 'buttons' expected string, got {}",
                    json_type_name(other)
                ),
            ));
        }
    };

    if buttons == ButtonsPreset::Custom {
        return Err(ValidationError::validation(
            "buttons",
            "buttons: custom is not supported for markdown",
        ));
    }

    let (width, height) = optional_window_size_fields(obj)?;

    Ok(Command::Markdown {
        title,
        file,
        content,
        status: status.map(ChromeStatus::new),
        buttons,
        width,
        height,
    })
}
