//! Markdown command validation (sprint b.5 — file only).

use serde_json::json;
use wyvern_schema::{ButtonsPreset, Command, ValidationError};

#[test]
fn validation_markdown_file_passes() {
    let cmd = wyvern_schema::validate(&json!({
        "type": "markdown",
        "file": "doc.md"
    }))
    .expect("valid markdown file");
    match cmd {
        Command::Markdown {
            title,
            file,
            content,
            status,
            buttons,
        } => {
            assert_eq!(title.as_ref().map(|t| t.as_str()), Some("doc.md"));
            assert_eq!(file.as_deref(), Some("doc.md"));
            assert!(content.is_none());
            assert!(status.is_none());
            assert_eq!(buttons, ButtonsPreset::Ok);
        }
        other => panic!("expected Markdown, got {other:?}"),
    }
}

#[test]
fn validation_markdown_explicit_title_and_status() {
    let cmd = wyvern_schema::validate(&json!({
        "type": "markdown",
        "file": "path/to/readme.md",
        "title": "Readme",
        "status": "Draft",
        "buttons": "ok_cancel"
    }))
    .expect("valid");
    match cmd {
        Command::Markdown {
            title,
            file,
            status,
            buttons,
            ..
        } => {
            assert_eq!(title.as_ref().map(|t| t.as_str()), Some("Readme"));
            assert_eq!(file.as_deref(), Some("path/to/readme.md"));
            assert_eq!(status.as_ref().map(|s| s.as_str()), Some("Draft"));
            assert_eq!(buttons, ButtonsPreset::OkCancel);
        }
        other => panic!("expected Markdown, got {other:?}"),
    }
}

#[test]
fn validation_markdown_title_defaults_to_filename() {
    let cmd = wyvern_schema::validate(&json!({
        "type": "markdown",
        "file": "/tmp/notes/guide.md"
    }))
    .expect("valid");
    match cmd {
        Command::Markdown { title, .. } => {
            assert_eq!(title.as_ref().map(|t| t.as_str()), Some("guide.md"));
        }
        other => panic!("expected Markdown, got {other:?}"),
    }
}

#[test]
fn validation_markdown_neither_file_nor_content_fails() {
    let err = wyvern_schema::validate(&json!({"type": "markdown"})).expect_err("neither");
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "file");
            assert!(message.contains("exactly one of"));
        }
        other => panic!("expected Validation, got {other:?}"),
    }
}

#[test]
fn validation_markdown_both_file_and_content_fails_req0058() {
    let err = wyvern_schema::validate(&json!({
        "type": "markdown",
        "file": "doc.md",
        "content": "# Hi"
    }))
    .expect_err("both");
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "file");
            assert!(message.contains("exactly one of"));
        }
        other => panic!("expected Validation, got {other:?}"),
    }
}

#[test]
fn validation_markdown_content_alone_rejected_until_b6() {
    let err = wyvern_schema::validate(&json!({
        "type": "markdown",
        "content": "# Hello"
    }))
    .expect_err("content alone");
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "content");
            assert!(message.contains("not supported until inline markdown ships (b.6)"));
        }
        other => panic!("expected Validation, got {other:?}"),
    }
}

#[test]
fn validation_markdown_unknown_field_fails() {
    let err = wyvern_schema::validate(&json!({
        "type": "markdown",
        "file": "doc.md",
        "extra": true
    }))
    .expect_err("unknown");
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "extra");
            assert!(message.contains("unknown field"));
        }
        other => panic!("expected Validation, got {other:?}"),
    }
}

#[test]
fn validation_markdown_buttons_typo_suggests() {
    let err = wyvern_schema::validate(&json!({
        "type": "markdown",
        "file": "doc.md",
        "buttons": "ok_cance"
    }))
    .expect_err("typo");
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "buttons");
            assert!(message.contains("expected one of"));
            assert!(message.contains("did you mean"));
        }
        other => panic!("expected Validation, got {other:?}"),
    }
}

#[test]
fn validation_markdown_custom_buttons_rejected_in_b5() {
    let err = wyvern_schema::validate(&json!({
        "type": "markdown",
        "file": "doc.md",
        "buttons": "custom"
    }))
    .expect_err("custom");
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "buttons");
            assert!(message.contains("not supported"));
        }
        other => panic!("expected Validation, got {other:?}"),
    }
}
