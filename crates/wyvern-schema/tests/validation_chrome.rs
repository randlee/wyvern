//! Integration coverage for Phase A chrome validation rules.

use serde_json::json;
use wyvern_schema::{validate, Command, ValidationError};

#[test]
fn rule_chrome_with_title_passes() {
    let cmd = validate(&json!({"type":"chrome","title":"T"})).unwrap();
    assert!(matches!(cmd, Command::Chrome { title, status: None, .. } if title == "T"));
}

#[test]
fn rule_chrome_missing_title_fails() {
    let err = validate(&json!({"type":"chrome"})).unwrap_err();
    assert!(matches!(
        err,
        ValidationError::Validation { ref field, .. } if field == "title"
    ));
}

#[test]
fn rule_empty_object_missing_type_fails() {
    let err = validate(&json!({})).unwrap_err();
    assert!(matches!(
        err,
        ValidationError::Validation { ref field, .. } if field == "type"
    ));
}

#[test]
fn rule_title_only_missing_type_fails() {
    let err = validate(&json!({"title":"T"})).unwrap_err();
    assert!(matches!(
        err,
        ValidationError::Validation { ref field, .. } if field == "type"
    ));
}

#[test]
fn rule_type_null_fails() {
    let err = validate(&json!({"type":null,"title":"T"})).unwrap_err();
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "type");
            assert!(message.contains("expected string") && message.contains("null"));
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn rule_type_number_fails() {
    let err = validate(&json!({"type":1,"title":"T"})).unwrap_err();
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "type");
            assert!(message.contains("expected string") && message.contains("number"));
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn rule_chrome_unknown_field_fails() {
    let err = validate(&json!({"type":"chrome","title":"T","foo":1})).unwrap_err();
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "foo");
            assert!(message.contains("unknown field"));
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn rule_wrong_field_types_explicit_expected_got() {
    let err = validate(&json!({"type":"chrome","title":[]})).unwrap_err();
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "title");
            assert!(message.contains("expected string"));
            assert!(message.contains("array"));
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn rule_type_message_missing_required_fields() {
    // Message is unlocked in b.1; incomplete payloads still fail validation.
    let err = validate(&json!({"type":"message","message":"Hi"})).unwrap_err();
    assert!(matches!(
        err,
        ValidationError::Validation { ref field, .. } if field == "title" || field == "buttons"
    ));
}

#[test]
fn rule_type_unknown_fails() {
    let err = validate(&json!({"type":"unknown"})).unwrap_err();
    assert!(matches!(
        err,
        ValidationError::Validation { ref field, .. } if field == "type"
    ));
}

#[test]
fn rule_action_show_state_error() {
    let err = validate(&json!({"action":"show"})).unwrap_err();
    match err {
        ValidationError::State { field, message } => {
            assert_eq!(field, "action");
            assert!(message.contains("--interactive"));
        }
        other => panic!("unexpected {other:?}"),
    }
}
