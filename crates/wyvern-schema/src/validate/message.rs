//! Validate `message` commands.

use serde_json::{Map, Value};

use crate::chrome::{ChromeStatus, ChromeTitle};
use crate::command::{ButtonsPreset, Command, MessageLevel};
use crate::error::ValidationError;
use crate::field_name::FieldName;

use super::helpers::{
    closest_match, json_type_name, optional_bool_field, optional_string_field, require_string_field,
    MESSAGE_FIELDS,
};

pub(super) fn validate_message(obj: &Map<String, Value>) -> Result<Command, ValidationError> {
    for key in obj.keys() {
        let key_str = key.as_str();
        if !MESSAGE_FIELDS.contains(&key_str) {
            return Err(ValidationError::validation(
                FieldName::new(key_str),
                format!("unknown field '{key_str}'"),
            ));
        }
    }

    let title = require_string_field(obj, "title")?;
    let message = require_string_field(obj, "message")?;
    let status = optional_string_field(obj, "status")?;

    let buttons = match obj.get("buttons") {
        None => {
            return Err(ValidationError::validation(
                "buttons",
                "missing required field 'buttons'",
            ));
        }
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

    let custom_buttons = match obj.get("custom_buttons") {
        None => None,
        Some(Value::Array(items)) => {
            let mut labels = Vec::with_capacity(items.len());
            for (i, item) in items.iter().enumerate() {
                match item {
                    Value::String(s) => labels.push(s.clone()),
                    other => {
                        return Err(ValidationError::validation(
                            "custom_buttons",
                            format!(
                                "custom_buttons[{i}] expected string, got {}",
                                json_type_name(other)
                            ),
                        ));
                    }
                }
            }
            Some(labels)
        }
        Some(other) => {
            return Err(ValidationError::validation(
                "custom_buttons",
                format!(
                    "field 'custom_buttons' expected array, got {}",
                    json_type_name(other)
                ),
            ));
        }
    };

    // REQ-0055: buttons: custom without custom_buttons → error
    if buttons == ButtonsPreset::Custom && custom_buttons.is_none() {
        return Err(ValidationError::validation(
            "custom_buttons",
            "buttons: custom requires a custom_buttons array",
        ));
    }

    // REQ-0056: custom_buttons with non-custom buttons → error
    if custom_buttons.is_some() && buttons != ButtonsPreset::Custom {
        return Err(ValidationError::validation(
            "custom_buttons",
            "custom_buttons is only valid when buttons is 'custom'",
        ));
    }

    let default_button = match obj.get("default_button") {
        None => None,
        Some(Value::Number(n)) => {
            let Some(idx) = n.as_u64() else {
                return Err(ValidationError::validation(
                    "default_button",
                    "field 'default_button' expected non-negative integer",
                ));
            };
            if idx > u64::from(u32::MAX) {
                return Err(ValidationError::validation(
                    "default_button",
                    "field 'default_button' exceeds u32 range",
                ));
            }
            Some(idx as u32)
        }
        Some(other) => {
            return Err(ValidationError::validation(
                "default_button",
                format!(
                    "field 'default_button' expected number, got {}",
                    json_type_name(other)
                ),
            ));
        }
    };

    if let Some(idx) = default_button {
        let count = buttons.button_count(custom_buttons.as_deref());
        if count == 0 || (idx as usize) >= count {
            return Err(ValidationError::validation(
                "default_button",
                format!("default_button index {idx} is out of range for {count} button(s)"),
            ));
        }
    }

    let level = match obj.get("level") {
        None => None,
        Some(Value::String(s)) => match MessageLevel::parse(s) {
            Some(level) => Some(level),
            None => {
                let options = MessageLevel::all_names().join(", ");
                let mut msg = format!("got '{s}', expected one of: {options}");
                if let Some(suggestion) = closest_match(s, MessageLevel::all_names()) {
                    msg.push_str(&format!("; did you mean '{suggestion}'?"));
                }
                return Err(ValidationError::validation("level", msg));
            }
        },
        Some(other) => {
            return Err(ValidationError::validation(
                "level",
                format!(
                    "field 'level' expected string, got {}",
                    json_type_name(other)
                ),
            ));
        }
    };

    // icon / image are opaque strings (path, URL, or UI template hint) — no catalog check.
    let icon = optional_string_field(obj, "icon")?;
    let image = optional_string_field(obj, "image")?;
    let markdown = optional_bool_field(obj, "markdown")?.unwrap_or(false);

    Ok(Command::Message {
        title: ChromeTitle::new(title),
        message,
        status: status.map(ChromeStatus::new),
        buttons,
        custom_buttons,
        default_button,
        level,
        icon,
        image,
        markdown,
    })
}
