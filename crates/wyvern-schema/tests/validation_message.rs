//! Integration coverage for Phase B `message` validation rules (sprint b.1).

use serde_json::json;
use wyvern_schema::{validate, ButtonsPreset, Command, ValidationError};

#[test]
fn validation_message_ok_preset_passes() {
    let cmd = validate(&json!({
        "type": "message",
        "title": "T",
        "message": "Hi",
        "buttons": "ok"
    }))
    .expect("valid");
    match cmd {
        Command::Message {
            title,
            message,
            status,
            buttons,
            custom_buttons,
            default_button,
        } => {
            assert_eq!(title.as_str(), "T");
            assert_eq!(message, "Hi");
            assert!(status.is_none());
            assert_eq!(buttons, ButtonsPreset::Ok);
            assert!(custom_buttons.is_none());
            assert!(default_button.is_none());
        }
        other => panic!("expected Message, got {other:?}"),
    }
}

#[test]
fn validation_message_all_presets_accepted() {
    for preset in ["ok", "ok_cancel", "yes_no", "yes_no_cancel", "retry_cancel"] {
        let cmd = validate(&json!({
            "type": "message",
            "title": "T",
            "message": "Hi",
            "buttons": preset
        }))
        .unwrap_or_else(|e| panic!("preset {preset} should pass: {e}"));
        assert!(matches!(cmd, Command::Message { .. }));
    }
}

#[test]
fn validation_message_custom_buttons_pass() {
    let cmd = validate(&json!({
        "type": "message",
        "title": "T",
        "message": "Hi",
        "buttons": "custom",
        "custom_buttons": ["Save", "Discard"],
        "default_button": 1
    }))
    .expect("valid custom");
    match cmd {
        Command::Message {
            buttons,
            custom_buttons,
            default_button,
            ..
        } => {
            assert_eq!(buttons, ButtonsPreset::Custom);
            assert_eq!(
                custom_buttons.as_deref(),
                Some(["Save".to_string(), "Discard".to_string()].as_slice())
            );
            assert_eq!(default_button, Some(1));
        }
        other => panic!("expected Message, got {other:?}"),
    }
}

#[test]
fn validation_message_missing_title_fails() {
    let err = validate(&json!({
        "type": "message",
        "message": "Hi",
        "buttons": "ok"
    }))
    .unwrap_err();
    assert!(matches!(
        err,
        ValidationError::Validation { ref field, .. } if field == "title"
    ));
}

#[test]
fn validation_message_missing_message_fails() {
    let err = validate(&json!({
        "type": "message",
        "title": "T",
        "buttons": "ok"
    }))
    .unwrap_err();
    assert!(matches!(
        err,
        ValidationError::Validation { ref field, .. } if field == "message"
    ));
}

#[test]
fn validation_message_custom_without_custom_buttons_fails() {
    let err = validate(&json!({
        "type": "message",
        "title": "T",
        "message": "Hi",
        "buttons": "custom"
    }))
    .unwrap_err();
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "custom_buttons");
            assert!(message.contains("custom"));
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn validation_message_custom_buttons_with_non_custom_fails() {
    let err = validate(&json!({
        "type": "message",
        "title": "T",
        "message": "Hi",
        "buttons": "ok_cancel",
        "custom_buttons": ["Nope"]
    }))
    .unwrap_err();
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "custom_buttons");
            assert!(message.contains("only valid"));
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn validation_message_default_button_out_of_range_fails() {
    let err = validate(&json!({
        "type": "message",
        "title": "T",
        "message": "Hi",
        "buttons": "yes_no",
        "default_button": 2
    }))
    .unwrap_err();
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "default_button");
            assert!(message.contains("out of range"));
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn validation_message_unknown_field_fails() {
    let err = validate(&json!({
        "type": "message",
        "title": "T",
        "message": "Hi",
        "buttons": "ok",
        "extra": true
    }))
    .unwrap_err();
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "extra");
            assert!(message.contains("unknown field"));
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn validation_message_deferred_level_fails() {
    let err = validate(&json!({
        "type": "message",
        "title": "T",
        "message": "Hi",
        "buttons": "ok",
        "level": "info"
    }))
    .unwrap_err();
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "level");
            assert!(message.contains("b.2"));
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn validation_message_deferred_icon_fails() {
    let err = validate(&json!({
        "type": "message",
        "title": "T",
        "message": "Hi",
        "buttons": "ok",
        "icon": "warning"
    }))
    .unwrap_err();
    assert!(matches!(
        err,
        ValidationError::Validation { ref field, .. } if field == "icon"
    ));
}

#[test]
fn validation_message_deferred_image_fails() {
    let err = validate(&json!({
        "type": "message",
        "title": "T",
        "message": "Hi",
        "buttons": "ok",
        "image": "x.png"
    }))
    .unwrap_err();
    assert!(matches!(
        err,
        ValidationError::Validation { ref field, .. } if field == "image"
    ));
}

#[test]
fn validation_message_deferred_markdown_fails() {
    let err = validate(&json!({
        "type": "message",
        "title": "T",
        "message": "Hi",
        "buttons": "ok",
        "markdown": true
    }))
    .unwrap_err();
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "markdown");
            assert!(message.contains("b.2"));
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn validation_message_buttons_near_miss_suggests() {
    let err = validate(&json!({
        "type": "message",
        "title": "T",
        "message": "Hi",
        "buttons": "ok-cancel"
    }))
    .unwrap_err();
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "buttons");
            assert!(message.contains("expected one of"));
            // "ok_cancel" is within Levenshtein ≤ 2 of "ok-cancel" (1 substitution)
            assert!(message.contains("did you mean"));
        }
        other => panic!("unexpected {other:?}"),
    }
}
