//! Markdown command validation (sprint b.6 — REQ-0058 file|content matrix).

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
            ..
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
fn validation_markdown_content_only_passes() {
    let cmd = wyvern_schema::validate(&json!({
        "type": "markdown",
        "content": "# Hello\n\nBody"
    }))
    .expect("valid inline markdown");
    match cmd {
        Command::Markdown {
            title,
            file,
            content,
            status,
            buttons,
            ..
        } => {
            assert_eq!(title.as_ref().map(|t| t.as_str()), Some("Markdown"));
            assert!(file.is_none());
            assert_eq!(content.as_deref(), Some("# Hello\n\nBody"));
            assert!(status.is_none());
            assert_eq!(buttons, ButtonsPreset::Ok);
        }
        other => panic!("expected Markdown, got {other:?}"),
    }
}

#[test]
fn validation_markdown_empty_content_passes() {
    let cmd = wyvern_schema::validate(&json!({
        "type": "markdown",
        "content": ""
    }))
    .expect("empty content allowed");
    match cmd {
        Command::Markdown {
            title,
            file,
            content,
            ..
        } => {
            assert_eq!(title.as_ref().map(|t| t.as_str()), Some("Markdown"));
            assert!(file.is_none());
            assert_eq!(content.as_deref(), Some(""));
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
fn validation_markdown_inline_title_and_status() {
    let cmd = wyvern_schema::validate(&json!({
        "type": "markdown",
        "content": "## Notes\n\n- item one",
        "title": "Inline doc",
        "status": "Read-only",
        "buttons": "ok"
    }))
    .expect("valid");
    match cmd {
        Command::Markdown {
            title,
            file,
            content,
            status,
            buttons,
            ..
        } => {
            assert_eq!(title.as_ref().map(|t| t.as_str()), Some("Inline doc"));
            assert!(file.is_none());
            assert_eq!(content.as_deref(), Some("## Notes\n\n- item one"));
            assert_eq!(status.as_ref().map(|s| s.as_str()), Some("Read-only"));
            assert_eq!(buttons, ButtonsPreset::Ok);
        }
        other => panic!("expected Markdown, got {other:?}"),
    }
}

#[test]
fn validation_markdown_title_defaults_to_filename() {
    let cmd = wyvern_schema::validate(&json!({
        "type": "markdown",
        "file": "notes/guide.md"
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
fn validation_markdown_custom_buttons_rejected() {
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

#[test]
fn validation_markdown_content_over_max_bytes_rejected() {
    let oversized = "x".repeat(wyvern_schema::MARKDOWN_CONTENT_MAX_BYTES + 1);
    let err = wyvern_schema::validate(&json!({
        "type": "markdown",
        "content": oversized
    }))
    .expect_err("oversized");
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "content");
            assert!(message.contains("exceeds maximum"));
            assert!(message.contains(&wyvern_schema::MARKDOWN_CONTENT_MAX_BYTES.to_string()));
        }
        other => panic!("expected Validation, got {other:?}"),
    }
}
