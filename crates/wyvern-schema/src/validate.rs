//! Validate JSON input against the Phase B executable surface (`chrome`, `message`).

use serde_json::{Map, Value};

use crate::chrome::{ChromeStatus, ChromeTitle};
use crate::command::{ButtonsPreset, Command, MessageLevel};
use crate::error::ValidationError;
use crate::field_name::FieldName;

/// Known lifecycle actions that require `--interactive` (REQ-0060).
const LIFECYCLE_ACTIONS: &[&str] = &["show", "hide", "exit"];

/// Allowed fields on a `chrome` command object.
const CHROME_FIELDS: &[&str] = &["type", "title", "status"];

/// Allowed fields on a `message` command object (b.2 full surface).
const MESSAGE_FIELDS: &[&str] = &[
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

/// Phase B executable `type` values (through b.2).
const VALID_TYPES: &[&str] = &["chrome", "message"];

/// Validate `value` as a Phase B command.
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

fn validate_chrome(obj: &Map<String, Value>) -> Result<Command, ValidationError> {
    for key in obj.keys() {
        if !CHROME_FIELDS.contains(&key.as_str()) {
            return Err(ValidationError::validation(
                FieldName::new(key.as_str()),
                format!("unknown field '{key}'"),
            ));
        }
    }

    let title = require_string_field(obj, "title")?;
    let status = optional_string_field(obj, "status")?;

    Ok(Command::Chrome {
        title: ChromeTitle::new(title),
        status: status.map(ChromeStatus::new),
    })
}

fn validate_message(obj: &Map<String, Value>) -> Result<Command, ValidationError> {
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

fn require_string_field(obj: &Map<String, Value>, field: &str) -> Result<String, ValidationError> {
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

fn optional_string_field(
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

fn optional_bool_field(
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

fn unknown_type_error(got: &str) -> ValidationError {
    let options = VALID_TYPES.join(", ");
    let mut message = format!("got '{got}', expected one of: {options}");
    if let Some(suggestion) = closest_match(got, VALID_TYPES) {
        message.push_str(&format!("; did you mean '{suggestion}'?"));
    }
    ValidationError::validation("type", message)
}

fn closest_match<'a>(got: &str, options: &[&'a str]) -> Option<&'a str> {
    options
        .iter()
        .copied()
        .filter(|opt| strsim::levenshtein(got, opt) <= 2)
        .min_by_key(|opt| strsim::levenshtein(got, opt))
}

fn json_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ButtonsPreset, ChromeStatus, ChromeTitle, MessageLevel};
    use serde_json::json;

    #[test]
    fn chrome_with_title_passes() {
        let cmd = validate(&json!({"type":"chrome","title":"T"})).expect("valid");
        assert_eq!(
            cmd,
            Command::Chrome {
                title: ChromeTitle::new("T"),
                status: None,
            }
        );
    }

    #[test]
    fn chrome_with_title_and_status_passes() {
        let cmd = validate(&json!({"type":"chrome","title":"T","status":"Ready"})).expect("valid");
        assert_eq!(
            cmd,
            Command::Chrome {
                title: ChromeTitle::new("T"),
                status: Some(ChromeStatus::new("Ready")),
            }
        );
    }

    #[test]
    fn chrome_missing_title_fails() {
        let err = validate(&json!({"type":"chrome"})).expect_err("missing title");
        match err {
            ValidationError::Validation { field, message } => {
                assert_eq!(field, "title");
                assert!(message.contains("missing required field 'title'"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn empty_object_missing_type_fails() {
        let err = validate(&json!({})).expect_err("missing type");
        match err {
            ValidationError::Validation { field, message } => {
                assert_eq!(field, "type");
                assert!(message.contains("missing required field 'type'"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn title_without_type_fails() {
        let err = validate(&json!({"title":"T"})).expect_err("missing type");
        match err {
            ValidationError::Validation { field, .. } => assert_eq!(field, "type"),
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn type_null_fails_non_string() {
        let err = validate(&json!({"type":null,"title":"T"})).expect_err("null type");
        match err {
            ValidationError::Validation { field, message } => {
                assert_eq!(field, "type");
                assert!(message.contains("expected string"));
                assert!(message.contains("null"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn type_number_fails_wrong_type() {
        let err = validate(&json!({"type":1,"title":"T"})).expect_err("number type");
        match err {
            ValidationError::Validation { field, message } => {
                assert_eq!(field, "type");
                assert!(message.contains("expected string"));
                assert!(message.contains("number"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn chrome_unknown_field_fails() {
        let err = validate(&json!({"type":"chrome","title":"T","extra":true}))
            .expect_err("unknown field");
        match err {
            ValidationError::Validation { field, message } => {
                assert_eq!(field, "extra");
                assert!(message.contains("unknown field 'extra'"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn chrome_title_wrong_type_fails() {
        let err = validate(&json!({"type":"chrome","title":123})).expect_err("wrong title type");
        match err {
            ValidationError::Validation { field, message } => {
                assert_eq!(field, "title");
                assert!(message.contains("expected string"));
                assert!(message.contains("number"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn chrome_status_wrong_type_fails() {
        let err =
            validate(&json!({"type":"chrome","title":"T","status":false})).expect_err("status");
        match err {
            ValidationError::Validation { field, message } => {
                assert_eq!(field, "status");
                assert!(message.contains("expected string"));
                assert!(message.contains("boolean"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn type_message_ok_passes() {
        let cmd = validate(&json!({
            "type":"message",
            "title":"T",
            "message":"Hi",
            "buttons":"ok"
        }))
        .expect("valid message");
        assert!(matches!(
            cmd,
            Command::Message {
                buttons: ButtonsPreset::Ok,
                default_button: None,
                level: None,
                icon: None,
                image: None,
                markdown: false,
                ..
            }
        ));
    }

    #[test]
    fn type_message_level_and_markdown_pass() {
        let cmd = validate(&json!({
            "type":"message",
            "title":"T",
            "message":"**Hi**",
            "buttons":"ok",
            "level":"warning",
            "markdown": true,
            "icon":"error",
            "image":"data:image/png;base64,AA=="
        }))
        .expect("valid message extras");
        match cmd {
            Command::Message {
                level,
                markdown,
                icon,
                image,
                ..
            } => {
                assert_eq!(level, Some(MessageLevel::Warning));
                assert!(markdown);
                assert_eq!(icon.as_deref(), Some("error"));
                assert_eq!(image.as_deref(), Some("data:image/png;base64,AA=="));
            }
            other => panic!("expected Message, got {other:?}"),
        }
    }

    #[test]
    fn type_message_level_invalid_fails_req0054() {
        let err = validate(&json!({
            "type":"message",
            "title":"T",
            "message":"Hi",
            "buttons":"ok",
            "level":"warnin"
        }))
        .expect_err("bad level");
        match err {
            ValidationError::Validation { field, message } => {
                assert_eq!(field, "level");
                assert!(message.contains("expected one of"));
                assert!(message.contains("did you mean 'warning'"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn type_unknown_fails() {
        let err = validate(&json!({"type":"unknown"})).expect_err("unknown type");
        match err {
            ValidationError::Validation { field, message } => {
                assert_eq!(field, "type");
                assert!(message.contains("unknown"));
                assert!(message.contains("chrome"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn type_near_miss_suggests_chrome() {
        let err = validate(&json!({"type":"chrom","title":"T"})).expect_err("typo");
        match err {
            ValidationError::Validation { message, .. } => {
                assert!(message.contains("did you mean 'chrome'"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn action_show_is_state_error() {
        let err = validate(&json!({"action":"show"})).expect_err("state");
        match err {
            ValidationError::State { field, message } => {
                assert_eq!(field, "action");
                assert!(message.contains("show"));
                assert!(message.contains("--interactive"));
            }
            other => panic!("expected State, got {other:?}"),
        }
    }

    #[test]
    fn action_hide_is_state_error() {
        let err = validate(&json!({"action":"hide"})).expect_err("state");
        assert!(matches!(err, ValidationError::State { .. }));
    }

    #[test]
    fn action_exit_is_state_error() {
        let err = validate(&json!({"action":"exit"})).expect_err("state");
        assert!(matches!(err, ValidationError::State { .. }));
    }

    #[test]
    fn non_object_fails() {
        let err = validate(&json!("chrome")).expect_err("not object");
        match err {
            ValidationError::Validation { field, message } => {
                assert_eq!(field, "type");
                assert!(message.contains("expected JSON object"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn default_button_out_of_range_fails() {
        let err = validate(&json!({
            "type":"message",
            "title":"T",
            "message":"Hi",
            "buttons":"ok",
            "default_button": 1
        }))
        .expect_err("oob");
        match err {
            ValidationError::Validation { field, message } => {
                assert_eq!(field, "default_button");
                assert!(message.contains("out of range"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn custom_without_custom_buttons_fails() {
        let err = validate(&json!({
            "type":"message",
            "title":"T",
            "message":"Hi",
            "buttons":"custom"
        }))
        .expect_err("REQ-0055");
        match err {
            ValidationError::Validation { field, .. } => assert_eq!(field, "custom_buttons"),
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn custom_buttons_with_non_custom_preset_fails() {
        let err = validate(&json!({
            "type":"message",
            "title":"T",
            "message":"Hi",
            "buttons":"ok",
            "custom_buttons":["A"]
        }))
        .expect_err("REQ-0056");
        match err {
            ValidationError::Validation { field, .. } => assert_eq!(field, "custom_buttons"),
            other => panic!("expected Validation, got {other:?}"),
        }
    }
}
