//! Integration coverage for Phase B `message` validation rules (sprint b.2).

use serde_json::json;
use wyvern_schema::{validate, ButtonsPreset, Command, MessageLevel, ValidationError};

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
            level,
            icon,
            image,
            markdown,
        } => {
            assert_eq!(title.as_str(), "T");
            assert_eq!(message, "Hi");
            assert!(status.is_none());
            assert_eq!(buttons, ButtonsPreset::Ok);
            assert!(custom_buttons.is_none());
            assert!(default_button.is_none());
            assert!(level.is_none());
            assert!(icon.is_none());
            assert!(image.is_none());
            assert!(!markdown);
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
fn validation_message_level_info_passes() {
    let cmd = validate(&json!({
        "type": "message",
        "title": "T",
        "message": "Hi",
        "buttons": "ok",
        "level": "info"
    }))
    .expect("level unlocked");
    match cmd {
        Command::Message { level, .. } => assert_eq!(level, Some(MessageLevel::Info)),
        other => panic!("expected Message, got {other:?}"),
    }
}

#[test]
fn validation_message_all_levels_accepted() {
    for (name, expected) in [
        ("info", MessageLevel::Info),
        ("warning", MessageLevel::Warning),
        ("error", MessageLevel::Error),
        ("question", MessageLevel::Question),
    ] {
        let cmd = validate(&json!({
            "type": "message",
            "title": "T",
            "message": "Hi",
            "buttons": "ok",
            "level": name
        }))
        .unwrap_or_else(|e| panic!("level {name} should pass: {e}"));
        match cmd {
            Command::Message { level, .. } => assert_eq!(level, Some(expected)),
            other => panic!("expected Message, got {other:?}"),
        }
    }
}

#[test]
fn validation_message_level_invalid_enum_fails() {
    let err = validate(&json!({
        "type": "message",
        "title": "T",
        "message": "Hi",
        "buttons": "ok",
        "level": "critical"
    }))
    .unwrap_err();
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "level");
            assert!(message.contains("expected one of"));
            assert!(message.contains("info"));
            assert!(message.contains("warning"));
            assert!(message.contains("error"));
            assert!(message.contains("question"));
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn validation_message_level_near_miss_suggests() {
    let err = validate(&json!({
        "type": "message",
        "title": "T",
        "message": "Hi",
        "buttons": "ok",
        "level": "warnin"
    }))
    .unwrap_err();
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "level");
            assert!(message.contains("did you mean 'warning'"));
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn validation_message_icon_field_passes() {
    let cmd = validate(&json!({
        "type": "message",
        "title": "T",
        "message": "Hi",
        "buttons": "ok",
        "icon": "warning:2"
    }))
    .expect("icon unlocked");
    match cmd {
        Command::Message { icon, .. } => assert_eq!(icon.as_deref(), Some("warning:2")),
        other => panic!("expected Message, got {other:?}"),
    }
}

#[test]
fn validation_message_icon_default_variant_passes() {
    let cmd = validate(&json!({
        "type": "message",
        "title": "T",
        "message": "Hi",
        "buttons": "ok",
        "icon": "warning"
    }))
    .expect("default variant");
    match cmd {
        Command::Message { icon, .. } => assert_eq!(icon.as_deref(), Some("warning")),
        other => panic!("expected Message, got {other:?}"),
    }
}

#[test]
fn validation_message_icon_path_and_data_uri_pass() {
    let path = validate(&json!({
        "type": "message",
        "title": "T",
        "message": "Hi",
        "buttons": "ok",
        "icon": "/path/to/icon.svg"
    }))
    .expect("path icon");
    match path {
        Command::Message { icon, .. } => assert_eq!(icon.as_deref(), Some("/path/to/icon.svg")),
        other => panic!("expected Message, got {other:?}"),
    }

    let data = validate(&json!({
        "type": "message",
        "title": "T",
        "message": "Hi",
        "buttons": "ok",
        "icon": "data:image/png;base64,AA=="
    }))
    .expect("data uri icon");
    match data {
        Command::Message { icon, .. } => {
            assert_eq!(icon.as_deref(), Some("data:image/png;base64,AA=="));
        }
        other => panic!("expected Message, got {other:?}"),
    }
}

#[test]
fn validation_message_image_field_passes() {
    let cmd = validate(&json!({
        "type": "message",
        "title": "T",
        "message": "Hi",
        "buttons": "ok",
        "image": "/tmp/deco.png"
    }))
    .expect("image unlocked");
    match cmd {
        Command::Message { image, .. } => assert_eq!(image.as_deref(), Some("/tmp/deco.png")),
        other => panic!("expected Message, got {other:?}"),
    }
}

#[test]
fn validation_message_image_named_icon_passes() {
    let cmd = validate(&json!({
        "type": "message",
        "title": "T",
        "message": "Hi",
        "buttons": "ok",
        "image": "success:2"
    }))
    .expect("named image");
    match cmd {
        Command::Message { image, .. } => assert_eq!(image.as_deref(), Some("success:2")),
        other => panic!("expected Message, got {other:?}"),
    }
}

#[test]
fn validation_message_markdown_true_passes() {
    let cmd = validate(&json!({
        "type": "message",
        "title": "T",
        "message": "**Hi**",
        "buttons": "ok",
        "markdown": true
    }))
    .expect("markdown unlocked");
    match cmd {
        Command::Message { markdown, .. } => assert!(markdown),
        other => panic!("expected Message, got {other:?}"),
    }
}

#[test]
fn validation_message_markdown_wrong_type_fails() {
    let err = validate(&json!({
        "type": "message",
        "title": "T",
        "message": "Hi",
        "buttons": "ok",
        "markdown": "yes"
    }))
    .unwrap_err();
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "markdown");
            assert!(message.contains("expected boolean"));
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
