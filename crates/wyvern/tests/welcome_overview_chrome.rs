//! Welcome Overview is a terminal topic with Back/Finish chrome (no hop).

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn workspace_overview() -> PathBuf {
    workspace_root().join("share/wyvern/welcome/pages/overview.html")
}

fn embedded_overview() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("embedded/share/wyvern/welcome/pages/overview.html")
}

fn assert_overview_chrome(html: &str) {
    assert!(
        html.contains(r#"data-wizard-terminal="true""#),
        "overview must stay a terminal topic page"
    );
    assert!(
        html.contains(r#"<nav class="wizard-chrome""#),
        "overview must ship wizard chrome (Back/Finish)"
    );
    assert!(html.contains(r#"data-wizard-back"#), "missing Back control");
    assert!(
        html.contains(r#"data-wizard-next"#),
        "missing Finish control"
    );
    assert!(
        html.contains("Finish"),
        "terminal next button must be labeled Finish"
    );
    assert!(
        html.contains("return {}"),
        "collectCurrentPageData must return {{}}"
    );
    assert!(
        !html.contains("wizardNextWizard"),
        "overview Finish must not hop to another wizard"
    );
    assert!(
        !html.contains("Stub topic page"),
        "overview must not keep stub copy"
    );
    assert!(html.contains("AskUserQuestion"), "missing questions topic");
    assert!(html.contains("Template wizard"), "missing templates topic");
    assert!(html.contains("Agent DAG"), "missing agent-dag topic");
}

#[test]
fn overview_page_ships_back_and_finish_chrome() {
    let html = std::fs::read_to_string(workspace_overview()).expect("overview.html");
    assert_overview_chrome(&html);
}

#[test]
fn embedded_overview_tracks_workspace_share() {
    let workspace = std::fs::read_to_string(workspace_overview()).expect("workspace overview");
    let embedded = std::fs::read_to_string(embedded_overview()).expect("embedded overview");
    assert_eq!(
        workspace, embedded,
        "crates/wyvern/embedded/share must track workspace share/wyvern/welcome/pages/overview.html"
    );
    assert_overview_chrome(&embedded);
}
