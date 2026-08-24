//! Integration coverage for report validation rules (h.1 / REQ-0140).

use serde_json::json;
use wyvern_schema::{validate, Command, PanelRole, ReportCommand, ReportMode, ValidationError};

#[test]
fn report_view_minimal_passes() {
    let cmd = validate(&json!({
        "type": "report",
        "title": "XHTML review",
        "page": "pages/view.xhtml"
    }))
    .expect("valid");
    match cmd {
        Command::Report(ReportCommand {
            title,
            page,
            mode,
            panels,
            width,
            height,
        }) => {
            assert_eq!(title.as_str(), "XHTML review");
            assert_eq!(page.as_str(), "pages/view.xhtml");
            assert_eq!(mode, ReportMode::View);
            assert!(panels.is_none());
            assert!(width.is_none());
            assert!(height.is_none());
        }
        other => panic!("expected Report, got {other:?}"),
    }
}

#[test]
fn report_mode_defaults_to_view() {
    let cmd = validate(&json!({
        "type": "report",
        "title": "T",
        "page": "pages/view.html"
    }))
    .expect("valid");
    let Command::Report(r) = cmd else {
        panic!("expected Report");
    };
    assert_eq!(r.mode, ReportMode::View);
}

#[test]
fn report_review_requires_panels() {
    let err = validate(&json!({
        "type": "report",
        "title": "T",
        "page": "pages/view.xhtml",
        "mode": "review"
    }))
    .unwrap_err();
    assert!(matches!(
        err,
        ValidationError::Validation { ref field, .. } if field == "panels"
    ));
}

#[test]
fn report_review_empty_panels_fails() {
    let err = validate(&json!({
        "type": "report",
        "title": "T",
        "page": "pages/view.xhtml",
        "mode": "review",
        "panels": []
    }))
    .unwrap_err();
    assert!(matches!(
        err,
        ValidationError::Validation { ref field, .. } if field == "panels"
    ));
}

#[test]
fn report_review_with_panels_passes() {
    let cmd = validate(&json!({
        "type": "report",
        "title": "Failed benchmark panels",
        "page": "pages/view.xhtml",
        "mode": "review",
        "panels": [
            { "path": "panels/fail-1.xhtml", "label": "Fail 1", "role": "failure" }
        ],
        "width": 800,
        "height": 600
    }))
    .expect("valid");
    let Command::Report(r) = cmd else {
        panic!("expected Report");
    };
    assert_eq!(r.mode, ReportMode::Review);
    let panels = r.panels.expect("panels");
    assert_eq!(panels.len(), 1);
    assert_eq!(panels[0].path.as_str(), "panels/fail-1.xhtml");
    assert_eq!(panels[0].label.as_deref(), Some("Fail 1"));
    assert_eq!(panels[0].role, Some(PanelRole::Failure));
    assert_eq!(r.width, Some(800));
    assert_eq!(r.height, Some(600));
}

#[test]
fn report_unknown_field_fails() {
    let err = validate(&json!({
        "type": "report",
        "title": "T",
        "page": "pages/view.xhtml",
        "config": {}
    }))
    .unwrap_err();
    assert!(matches!(
        err,
        ValidationError::Validation { ref field, .. } if field == "config"
    ));
}

#[test]
fn report_workflow_field_fails() {
    let err = validate(&json!({
        "type": "report",
        "title": "T",
        "page": "pages/view.xhtml",
        "workflow": { "pre": "x.py" }
    }))
    .unwrap_err();
    assert!(matches!(
        err,
        ValidationError::Validation { ref field, .. } if field == "workflow"
    ));
}

#[test]
fn report_missing_title_fails() {
    let err = validate(&json!({
        "type": "report",
        "page": "pages/view.xhtml"
    }))
    .unwrap_err();
    assert!(matches!(
        err,
        ValidationError::Validation { ref field, .. } if field == "title"
    ));
}

#[test]
fn report_missing_page_fails() {
    let err = validate(&json!({
        "type": "report",
        "title": "T"
    }))
    .unwrap_err();
    assert!(matches!(
        err,
        ValidationError::Validation { ref field, .. } if field == "page"
    ));
}

#[test]
fn report_empty_title_fails() {
    let err = validate(&json!({
        "type": "report",
        "title": "",
        "page": "pages/view.xhtml"
    }))
    .unwrap_err();
    assert!(matches!(
        err,
        ValidationError::Validation { ref field, .. } if field == "title"
    ));
}

#[test]
fn report_page_must_be_html_or_xhtml() {
    let err = validate(&json!({
        "type": "report",
        "title": "T",
        "page": "pages/view.txt"
    }))
    .unwrap_err();
    assert!(matches!(
        err,
        ValidationError::Validation { ref field, .. } if field == "page"
    ));
}

#[test]
fn report_invalid_mode_fails() {
    let err = validate(&json!({
        "type": "report",
        "title": "T",
        "page": "pages/view.xhtml",
        "mode": "wizard"
    }))
    .unwrap_err();
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "mode");
            assert!(message.contains("view"));
            assert!(message.contains("review"));
        }
        other => panic!("expected Validation, got {other:?}"),
    }
}
