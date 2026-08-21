//! Integration tests for dataflow lint (WIZARD-LINT-005–008).

use std::path::PathBuf;

use wyvern::{run_wizard_command, WizardCmdResult};

fn fixture_dir(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/")
        .parent()
        .expect("workspace root")
        .join("fixtures")
        .join(name)
}

#[test]
fn dataflow_lint_clean_fixture_passes() {
    let dir = fixture_dir("wizard-dataflow-lint");
    let path_str = dir.to_str().expect("utf-8").to_string();
    let result = run_wizard_command(&["lint".into(), path_str]).expect("lint ok");
    match result {
        WizardCmdResult::Clean(report) => {
            assert!(
                !report.contains("WIZARD-LINT-005"),
                "unexpected 005:\n{report}"
            );
        }
        WizardCmdResult::Findings(report) => {
            panic!("expected clean dataflow fixture:\n{report}");
        }
    }
}

#[test]
fn dataflow_lint_unsatisfied_require_reports_005() {
    let dir = fixture_dir("wizard-dataflow-lint-bad");
    let path_str = dir.to_str().expect("utf-8").to_string();
    let result = run_wizard_command(&["lint".into(), path_str]).expect("lint ok");
    match result {
        WizardCmdResult::Findings(report) => {
            assert!(
                report.contains("WIZARD-LINT-005"),
                "expected 005 in:\n{report}"
            );
        }
        WizardCmdResult::Clean(report) => {
            panic!("expected 005 finding, got clean:\n{report}");
        }
    }
}

#[test]
fn wizard_lint_help_lists_dataflow_codes() {
    let result = run_wizard_command(&["lint".into(), "--help".into()]).expect("help ok");
    match result {
        WizardCmdResult::Clean(text) => {
            assert!(text.contains("WIZARD-LINT-005"), "{text}");
            assert!(text.contains("WIZARD-LINT-008"), "{text}");
        }
        WizardCmdResult::Findings(_) => panic!("help should be clean"),
    }
}
