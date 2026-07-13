//! Validate JSON input against the Phase A executable surface (`chrome` only).

use serde_json::{Map, Value};

use crate::chrome::{ChromeStatus, ChromeTitle};
use crate::command::Command;
use crate::error::ValidationError;
use crate::field_name::FieldName;

/// Known lifecycle actions that require `--interactive` (REQ-0060).
const LIFECYCLE_ACTIONS: &[&str] = &["show", "hide", "exit"];

/// Allowed fields on a `chrome` command object.
const CHROME_FIELDS: &[&str] = &["type", "title", "status"];

/// Phase A executable `type` values.
const VALID_TYPES: &[&str] = &["chrome"];

/// Validate `value` as a Phase A command.
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

    let title = match obj.get("title") {
        None => {
            return Err(ValidationError::validation(
                "title",
                "missing required field 'title'",
            ));
        }
        Some(Value::String(s)) => ChromeTitle::new(s.clone()),
        Some(other) => {
            return Err(ValidationError::validation(
                "title",
                format!(
                    "field 'title' expected string, got {}",
                    json_type_name(other)
                ),
            ));
        }
    };

    let status = match obj.get("status") {
        None => None,
        Some(Value::String(s)) => Some(ChromeStatus::new(s.clone())),
        Some(other) => {
            return Err(ValidationError::validation(
                "status",
                format!(
                    "field 'status' expected string, got {}",
                    json_type_name(other)
                ),
            ));
        }
    };

    Ok(Command::Chrome { title, status })
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
    use crate::{ChromeStatus, ChromeTitle};
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
    fn type_message_not_implemented_fails() {
        let err = validate(&json!({"type":"message","title":"T","message":"Hi"}))
            .expect_err("message not in Phase A");
        match err {
            ValidationError::Validation { field, message } => {
                assert_eq!(field, "type");
                assert!(message.contains("message"));
                assert!(message.contains("chrome"));
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
}
