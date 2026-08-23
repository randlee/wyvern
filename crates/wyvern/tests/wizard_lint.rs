//! Integration tests for `wyvern wizard lint`.
//!
//! Validates lint on shipped examples and CLI error paths.

use std::path::PathBuf;

use wyvern::{run_wizard_command, WizardCmdResult};

fn template_picker_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/")
        .parent()
        .expect("workspace root")
        .join("share/wyvern/examples/template-picker")
}

/// Running `wyvern wizard lint` on template-picker is clean after g.14 Cancel fix.
#[test]
fn template_picker_lint_is_clean() {
    let dir = template_picker_dir();
    assert!(
        dir.exists(),
        "template-picker dir not found at {}",
        dir.display()
    );

    let path_str = dir.to_str().expect("utf-8 path").to_string();
    let result = run_wizard_command(&["lint".into(), path_str]).expect("lint must not I/O-error");

    match result {
        WizardCmdResult::Clean(report) => {
            assert!(
                !report.contains("WIZARD-LINT-002"),
                "unexpected 002 after Cancel fix:\n{report}"
            );
        }
        WizardCmdResult::Findings(report) => {
            panic!("expected clean template-picker lint after g.14:\n{report}");
        }
    }
}

/// The lint report covers all three pages (pick, form, review).
#[test]
fn template_picker_lint_reaches_all_three_pages() {
    let dir = template_picker_dir();
    let path_str = dir.to_str().expect("utf-8 path").to_string();
    let result = run_wizard_command(&["lint".into(), path_str]).expect("no I/O error");

    // The summary line reports how many pages were checked.
    let text = match result {
        WizardCmdResult::Clean(t) | WizardCmdResult::Findings(t) => t,
    };
    // Expect "3 pages" in the summary.
    assert!(
        text.contains("3 page"),
        "expected 3 pages in summary line:\n{text}"
    );
}

/// `wyvern wizard lint --help` returns usage text, exits clean.
#[test]
fn wizard_lint_help_returns_usage_clean() {
    let result =
        run_wizard_command(&["lint".into(), "--help".into()]).expect("help should not error");
    match result {
        WizardCmdResult::Clean(text) => {
            assert!(text.contains("wyvern wizard lint"), "{text}");
            assert!(text.contains("WIZARD-LINT-001"), "{text}");
        }
        WizardCmdResult::Findings(_) => panic!("help should be Clean"),
    }
}

/// `wyvern wizard lint` with no paths is a usage error (exit 2 territory).
#[test]
fn wizard_lint_no_paths_is_usage_error() {
    use wyvern::WizardCmdError;
    let err = run_wizard_command(&["lint".into()]).expect_err("should error");
    match err {
        WizardCmdError::Usage { message, .. } => {
            assert!(
                message.contains("requires at least one"),
                "expected 'requires at least one <path>': {message}"
            );
        }
        other => panic!("expected Usage error, got {other:?}"),
    }
}

/// `wyvern wizard lint` on a nonexistent path returns a Stage error (exit 1).
#[test]
fn wizard_lint_nonexistent_path_is_stage_error() {
    use wyvern::WizardCmdError;
    let err = run_wizard_command(&["lint".into(), "/nonexistent/wizard-pkg".into()])
        .expect_err("should error for missing path");
    match err {
        WizardCmdError::Stage { message, exit_code } => {
            assert!(
                message.contains("not found") || message.contains("error"),
                "expected error message: {message}"
            );
            assert_eq!(exit_code, 1);
        }
        other => panic!("expected Stage error, got {other:?}"),
    }
}
