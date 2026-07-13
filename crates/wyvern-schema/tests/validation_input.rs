//! Integration coverage for Phase B `input` validation (sprint b.4).

use serde_json::json;
use wyvern_schema::{validate, ButtonsPreset, Command, InputMode, ValidationError};

#[test]
fn validation_input_minimal_defaults_text_and_ok_cancel() {
    let cmd = validate(&json!({
        "type": "input",
        "title": "Name",
        "message": "Enter name"
    }))
    .expect("valid");
    match cmd {
        Command::Input {
            title,
            message,
            status,
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
        } => {
            assert_eq!(title.as_str(), "Name");
            assert_eq!(message, "Enter name");
            assert!(status.is_none());
            assert!(icon.is_none());
            assert!(!markdown);
            assert!(!multiline);
            assert!(placeholder.is_none());
            assert!(default.is_none());
            assert_eq!(mode, InputMode::Text);
            assert!(filter.is_none());
            assert!(!multiple);
            assert!(start_path.is_none());
            assert_eq!(buttons, ButtonsPreset::OkCancel);
        }
        other => panic!("expected Input, got {other:?}"),
    }
}

#[test]
fn validation_input_mode_text_explicit_passes() {
    let cmd = validate(&json!({
        "type": "input",
        "title": "T",
        "message": "M",
        "mode": "text",
        "buttons": "ok"
    }))
    .expect("valid");
    match cmd {
        Command::Input { mode, buttons, .. } => {
            assert_eq!(mode, InputMode::Text);
            assert_eq!(buttons, ButtonsPreset::Ok);
        }
        other => panic!("expected Input, got {other:?}"),
    }
}

#[test]
fn validation_input_placeholder_and_default_allowed() {
    let cmd = validate(&json!({
        "type": "input",
        "title": "T",
        "message": "M",
        "placeholder": "hint",
        "default": "prefill",
        "multiline": true,
        "markdown": true,
        "status": "Ready",
        "icon": "info"
    }))
    .expect("valid");
    match cmd {
        Command::Input {
            placeholder,
            default,
            multiline,
            markdown,
            status,
            icon,
            ..
        } => {
            assert_eq!(placeholder.as_deref(), Some("hint"));
            assert_eq!(default.as_deref(), Some("prefill"));
            assert!(multiline);
            assert!(markdown);
            assert_eq!(status.as_ref().map(|s| s.as_str()), Some("Ready"));
            assert_eq!(icon.as_deref(), Some("info"));
        }
        other => panic!("expected Input, got {other:?}"),
    }
}

#[test]
fn validation_input_mode_file_passes() {
    let cmd = validate(&json!({
        "type": "input",
        "title": "T",
        "message": "Pick a file",
        "mode": "file",
        "filter": ["*.json", "*.txt"],
        "multiple": true,
        "start_path": "/tmp"
    }))
    .expect("valid file mode");
    match cmd {
        Command::Input {
            mode,
            filter,
            multiple,
            start_path,
            ..
        } => {
            assert_eq!(mode, InputMode::File);
            assert_eq!(
                filter.as_deref(),
                Some(["*.json".to_string(), "*.txt".to_string()].as_slice())
            );
            assert!(multiple);
            assert_eq!(start_path.as_deref(), Some("/tmp"));
        }
        other => panic!("expected Input, got {other:?}"),
    }
}

#[test]
fn validation_input_mode_folder_passes() {
    let cmd = validate(&json!({
        "type": "input",
        "title": "T",
        "message": "Pick a folder",
        "mode": "folder",
        "start_path": "/tmp"
    }))
    .expect("valid folder mode");
    match cmd {
        Command::Input {
            mode,
            filter,
            multiple,
            start_path,
            ..
        } => {
            assert_eq!(mode, InputMode::Folder);
            assert!(filter.is_none());
            assert!(!multiple);
            assert_eq!(start_path.as_deref(), Some("/tmp"));
        }
        other => panic!("expected Input, got {other:?}"),
    }
}

#[test]
fn validation_input_multiline_with_file_fails_req0057() {
    let err = validate(&json!({
        "type": "input",
        "title": "T",
        "message": "M",
        "mode": "file",
        "multiline": true
    }))
    .expect_err("REQ-0057");
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "multiline");
            assert!(message.contains("only valid when mode is 'text'"));
        }
        other => panic!("expected Validation, got {other:?}"),
    }
}

#[test]
fn validation_input_multiline_with_folder_fails_req0057() {
    let err = validate(&json!({
        "type": "input",
        "title": "T",
        "message": "M",
        "mode": "folder",
        "multiline": true
    }))
    .expect_err("REQ-0057");
    match err {
        ValidationError::Validation { field, .. } => assert_eq!(field, "multiline"),
        other => panic!("expected Validation, got {other:?}"),
    }
}

#[test]
fn validation_input_placeholder_with_file_fails_req0059() {
    let err = validate(&json!({
        "type": "input",
        "title": "T",
        "message": "M",
        "mode": "file",
        "placeholder": "hint"
    }))
    .expect_err("REQ-0059 placeholder");
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "placeholder");
            assert!(message.contains("only valid when mode is 'text'"));
        }
        other => panic!("expected Validation, got {other:?}"),
    }
}

#[test]
fn validation_input_default_with_folder_fails_req0059() {
    let err = validate(&json!({
        "type": "input",
        "title": "T",
        "message": "M",
        "mode": "folder",
        "default": "x"
    }))
    .expect_err("REQ-0059 default");
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "default");
            assert!(message.contains("only valid when mode is 'text'"));
        }
        other => panic!("expected Validation, got {other:?}"),
    }
}

#[test]
fn validation_input_filter_with_text_mode_fails_req0059() {
    let err = validate(&json!({
        "type": "input",
        "title": "T",
        "message": "M",
        "filter": ["*.rs"]
    }))
    .expect_err("REQ-0059 filter");
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "filter");
            assert!(message.contains("only valid when mode is 'file'"));
        }
        other => panic!("expected Validation, got {other:?}"),
    }
}

#[test]
fn validation_input_filter_with_folder_fails_req0059() {
    let err = validate(&json!({
        "type": "input",
        "title": "T",
        "message": "M",
        "mode": "folder",
        "filter": ["*.rs"]
    }))
    .expect_err("REQ-0059 filter+folder");
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "filter");
            assert!(message.contains("only valid when mode is 'file'"));
        }
        other => panic!("expected Validation, got {other:?}"),
    }
}

#[test]
fn validation_input_multiple_with_text_mode_fails_req0059() {
    let err = validate(&json!({
        "type": "input",
        "title": "T",
        "message": "M",
        "mode": "text",
        "multiple": true
    }))
    .expect_err("REQ-0059 multiple");
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "multiple");
            assert!(message.contains("only valid when mode is 'file'"));
        }
        other => panic!("expected Validation, got {other:?}"),
    }
}

#[test]
fn validation_input_multiple_with_folder_fails_req0059() {
    let err = validate(&json!({
        "type": "input",
        "title": "T",
        "message": "M",
        "mode": "folder",
        "multiple": true
    }))
    .expect_err("REQ-0059 multiple+folder");
    match err {
        ValidationError::Validation { field, .. } => assert_eq!(field, "multiple"),
        other => panic!("expected Validation, got {other:?}"),
    }
}

#[test]
fn validation_input_start_path_with_text_mode_fails_req0059() {
    let err = validate(&json!({
        "type": "input",
        "title": "T",
        "message": "M",
        "start_path": "/tmp"
    }))
    .expect_err("REQ-0059 start_path");
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "start_path");
            assert!(message.contains("only valid when mode is 'file' or 'folder'"));
        }
        other => panic!("expected Validation, got {other:?}"),
    }
}

#[test]
fn validation_input_missing_title_fails() {
    let err = validate(&json!({
        "type": "input",
        "message": "M"
    }))
    .expect_err("missing title");
    match err {
        ValidationError::Validation { field, .. } => assert_eq!(field, "title"),
        other => panic!("expected Validation, got {other:?}"),
    }
}

#[test]
fn validation_input_missing_message_fails() {
    let err = validate(&json!({
        "type": "input",
        "title": "T"
    }))
    .expect_err("missing message");
    match err {
        ValidationError::Validation { field, .. } => assert_eq!(field, "message"),
        other => panic!("expected Validation, got {other:?}"),
    }
}

#[test]
fn validation_input_mode_typo_suggests() {
    let err = validate(&json!({
        "type": "input",
        "title": "T",
        "message": "M",
        "mode": "txt"
    }))
    .expect_err("bad mode");
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "mode");
            assert!(message.contains("expected one of"));
            assert!(message.contains("did you mean 'text'"));
        }
        other => panic!("expected Validation, got {other:?}"),
    }
}

#[test]
fn validation_input_unknown_field_fails() {
    let err = validate(&json!({
        "type": "input",
        "title": "T",
        "message": "M",
        "level": "info"
    }))
    .expect_err("unknown field");
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "level");
            assert!(message.contains("unknown field"));
        }
        other => panic!("expected Validation, got {other:?}"),
    }
}
