
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
    let err =
        validate(&json!({"type":"chrome","title":"T","extra":true})).expect_err("unknown field");
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
    let err = validate(&json!({"type":"chrome","title":"T","status":false})).expect_err("status");
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
