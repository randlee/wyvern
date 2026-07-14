//! Validate `input` commands.

use serde_json::{Map, Value};

use crate::chrome::{ChromeStatus, ChromeTitle};
use crate::command::{ButtonsPreset, Command, InputMode};
use crate::error::ValidationError;
use crate::field_name::FieldName;

use super::helpers::{
    closest_match, json_type_name, optional_bool_field, optional_string_field, require_string_field,
    INPUT_FIELDS,
};

pub(super) fn validate_input(obj: &Map<String, Value>) -> Result<Command, ValidationError> {
    for key in obj.keys() {
        let key_str = key.as_str();
        if !INPUT_FIELDS.contains(&key_str) {
            return Err(ValidationError::validation(
                FieldName::new(key_str),
                format!("unknown field '{key_str}'"),
            ));
        }
    }

    let title = require_string_field(obj, "title")?;
    let message = require_string_field(obj, "message")?;
    let status = optional_string_field(obj, "status")?;
    // icon is an opaque string (path, URL, or UI template hint) — no catalog check.
    let icon = optional_string_field(obj, "icon")?;
    let markdown = optional_bool_field(obj, "markdown")?.unwrap_or(false);
    let multiline = optional_bool_field(obj, "multiline")?.unwrap_or(false);
    let placeholder = optional_string_field(obj, "placeholder")?;
    let default = optional_string_field(obj, "default")?;

    let mode = match obj.get("mode") {
        None => InputMode::Text,
        Some(Value::String(s)) => match InputMode::parse(s) {
            Some(mode) => mode,
            None => {
                let options = InputMode::all_names().join(", ");
                let mut msg = format!("got '{s}', expected one of: {options}");
                if let Some(suggestion) = closest_match(s, InputMode::all_names()) {
                    msg.push_str(&format!("; did you mean '{suggestion}'?"));
                }
                return Err(ValidationError::validation("mode", msg));
            }
        },
        Some(other) => {
            return Err(ValidationError::validation(
                "mode",
                format!(
                    "field 'mode' expected string, got {}",
                    json_type_name(other)
                ),
            ));
        }
    };

    // REQ-0057 — multiline is text-mode only.
    if multiline && matches!(mode, InputMode::File | InputMode::Folder) {
        return Err(ValidationError::validation(
            "multiline",
            "multiline is only valid when mode is 'text' (or omitted)",
        ));
    }

    // REQ-0059 — placeholder / default only for text mode.
    if placeholder.is_some() && matches!(mode, InputMode::File | InputMode::Folder) {
        return Err(ValidationError::validation(
            "placeholder",
            "placeholder is only valid when mode is 'text' (or omitted)",
        ));
    }
    if default.is_some() && matches!(mode, InputMode::File | InputMode::Folder) {
        return Err(ValidationError::validation(
            "default",
            "default is only valid when mode is 'text' (or omitted)",
        ));
    }

    // REQ-0059 — filter / multiple only for file mode.
    let filter = match obj.get("filter") {
        None => None,
        Some(_) if mode != InputMode::File => {
            return Err(ValidationError::validation(
                "filter",
                "filter is only valid when mode is 'file'",
            ));
        }
        Some(Value::Array(items)) => {
            let mut patterns = Vec::with_capacity(items.len());
            for (i, item) in items.iter().enumerate() {
                match item {
                    Value::String(s) => patterns.push(s.clone()),
                    other => {
                        return Err(ValidationError::validation(
                            "filter",
                            format!("filter[{i}] expected string, got {}", json_type_name(other)),
                        ));
                    }
                }
            }
            Some(patterns)
        }
        Some(other) => {
            return Err(ValidationError::validation(
                "filter",
                format!(
                    "field 'filter' expected array, got {}",
                    json_type_name(other)
                ),
            ));
        }
    };

    let multiple = match obj.get("multiple") {
        None => false,
        Some(_) if mode != InputMode::File => {
            return Err(ValidationError::validation(
                "multiple",
                "multiple is only valid when mode is 'file'",
            ));
        }
        Some(Value::Bool(b)) => *b,
        Some(other) => {
            return Err(ValidationError::validation(
                "multiple",
                format!(
                    "field 'multiple' expected boolean, got {}",
                    json_type_name(other)
                ),
            ));
        }
    };

    // REQ-0059 — start_path only for file or folder mode.
    let start_path = match obj.get("start_path") {
        None => None,
        Some(_) if matches!(mode, InputMode::Text) => {
            return Err(ValidationError::validation(
                "start_path",
                "start_path is only valid when mode is 'file' or 'folder'",
            ));
        }
        Some(Value::String(s)) => Some(s.clone()),
        Some(other) => {
            return Err(ValidationError::validation(
                "start_path",
                format!(
                    "field 'start_path' expected string, got {}",
                    json_type_name(other)
                ),
            ));
        }
    };

    let buttons = match obj.get("buttons") {
        None => ButtonsPreset::OkCancel,
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
            "buttons: custom is not supported for input in sprint b.4",
        ));
    }

    Ok(Command::Input {
        title: ChromeTitle::new(title),
        message,
        status: status.map(ChromeStatus::new),
        icon,
        markdown,
        multiline,
        placeholder,
        default,
        mode,
        filter,
        multiple,
        start_path,
        buttons,
    })
}
