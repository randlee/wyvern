//! Integration coverage for wizard validation rules (d.1).

use serde_json::json;
use wyvern_schema::{validate, Command, ValidationError, WizardCommand, WizardPageLayout};

#[test]
fn validation_wizard_minimal_passes() {
    let cmd = validate(&json!({
        "type": "wizard",
        "page": {
            "id": "start",
            "title": "Start",
            "html": "pages/start.html"
        }
    }))
    .expect("valid");
    match cmd {
        Command::Wizard(WizardCommand {
            page,
            config,
            width,
            height,
        }) => {
            assert_eq!(page.id, "start");
            assert_eq!(page.title, "Start");
            assert_eq!(page.html, "pages/start.html");
            assert!(page.layout.is_none());
            assert_eq!(config, json!({}));
            assert!(width.is_none());
            assert!(height.is_none());
        }
        other => panic!("expected Wizard, got {other:?}"),
    }
}

#[test]
fn validation_wizard_layout_dialog_and_workspace() {
    for (wire, expected) in [
        ("dialog", WizardPageLayout::Dialog),
        ("workspace", WizardPageLayout::Workspace),
    ] {
        let cmd = validate(&json!({
            "type": "wizard",
            "page": {
                "id": "p",
                "title": "P",
                "html": "p.html",
                "layout": wire
            },
            "config": { "theme": "dark" },
            "width": 640,
            "height": 480
        }))
        .expect("valid");
        let Command::Wizard(w) = cmd else {
            panic!("expected Wizard");
        };
        assert_eq!(w.page.layout, Some(expected));
        assert_eq!(w.config["theme"], "dark");
        assert_eq!(w.width, Some(640));
        assert_eq!(w.height, Some(480));
    }
}

#[test]
fn validation_wizard_missing_page_fails() {
    let err = validate(&json!({"type": "wizard"})).unwrap_err();
    assert!(matches!(
        err,
        ValidationError::Validation { ref field, .. } if field == "page"
    ));
}

#[test]
fn validation_wizard_missing_page_id_fails() {
    let err = validate(&json!({
        "type": "wizard",
        "page": { "title": "T", "html": "a.html" }
    }))
    .unwrap_err();
    assert!(matches!(
        err,
        ValidationError::Validation { ref field, .. } if field == "page.id"
    ));
}

#[test]
fn validation_wizard_empty_html_fails() {
    let err = validate(&json!({
        "type": "wizard",
        "page": { "id": "a", "title": "T", "html": "" }
    }))
    .unwrap_err();
    assert!(matches!(
        err,
        ValidationError::Validation { ref field, .. } if field == "page.html"
    ));
}

#[test]
fn validation_wizard_invalid_layout_fails() {
    let err = validate(&json!({
        "type": "wizard",
        "page": {
            "id": "a",
            "title": "T",
            "html": "a.html",
            "layout": "fullscreen"
        }
    }))
    .unwrap_err();
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "page.layout");
            assert!(message.contains("dialog") && message.contains("workspace"));
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn validation_wizard_unknown_field_fails() {
    let err = validate(&json!({
        "type": "wizard",
        "page": { "id": "a", "title": "T", "html": "a.html" },
        "page_html": "nope"
    }))
    .unwrap_err();
    assert!(matches!(
        err,
        ValidationError::Validation { ref field, .. } if field == "page_html"
    ));
}
