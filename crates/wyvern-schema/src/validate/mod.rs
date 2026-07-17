//! Validate JSON input against the executable command surface
//! (`chrome`, `message`, `input`, `markdown`, `question`, `wizard`).

mod chrome;
mod helpers;
mod input;
mod markdown;
mod message;
mod question;
mod wizard;

#[cfg(test)]
mod tests;

use serde_json::{Map, Value};

use crate::command::Command;
use crate::error::ValidationError;

use chrome::validate_chrome;
use helpers::{json_type_name, unknown_type_error, LIFECYCLE_ACTIONS};
use input::validate_input;
use markdown::validate_markdown;
use message::validate_message;
use question::validate_question;
use wizard::validate_wizard;

#[doc(inline)]
pub use helpers::MARKDOWN_CONTENT_MAX_BYTES;

/// Validate `value` as an executable command.
///
/// # Errors
///
/// Returns [`ValidationError::Validation`] for schema/field failures and
/// [`ValidationError::State`] when a lifecycle `action` is used outside
/// `--interactive`.
pub fn validate(value: &Value) -> Result<Command, ValidationError> {
    let obj = match value {
        Value::Object(map) => map,
        _ => {
            return Err(ValidationError::validation("type", "expected JSON object"));
        }
    };

    if let Some(err) = check_lifecycle_action(obj) {
        return Err(err);
    }

    let type_value = match obj.get("type") {
        None => {
            return Err(ValidationError::validation(
                "type",
                "missing required field 'type'",
            ));
        }
        Some(v) => v,
    };

    let type_str = match type_value {
        Value::String(s) => s.as_str(),
        other => {
            return Err(ValidationError::validation(
                "type",
                format!(
                    "field 'type' expected string, got {}",
                    json_type_name(other)
                ),
            ));
        }
    };

    match type_str {
        "chrome" => validate_chrome(obj),
        "message" => validate_message(obj),
        "input" => validate_input(obj),
        "markdown" => validate_markdown(obj),
        "question" => validate_question(obj),
        "wizard" => validate_wizard(obj),
        other => Err(unknown_type_error(other)),
    }
}

fn check_lifecycle_action(obj: &Map<String, Value>) -> Option<ValidationError> {
    let action = obj.get("action")?;
    let action_str = action.as_str()?;
    if LIFECYCLE_ACTIONS.contains(&action_str) {
        Some(ValidationError::state(
            "action",
            format!("{action_str} is only valid in --interactive mode"),
        ))
    } else {
        None
    }
}
