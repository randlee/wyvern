//! `report-xhtml-review` expands a manifest to review-mode finish (REQ-0144).

mod test_support;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

use test_support::{AbsentProbe, PresentProbe};
use wyvern::extensions::{
    build_match_context, expand_and_validate, ExtensionRegistry, PathRequiresProbe, RequiresProbe,
};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root")
}

fn workspace_share_dir() -> PathBuf {
    workspace_root().join("share/wyvern")
}

fn load_shipped() -> ExtensionRegistry {
    let defaults = workspace_share_dir().join("extensions.json");
    ExtensionRegistry::load(&defaults, None).expect("shipped registry")
}

fn fixture_review_manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/xhtml-review/review.json")
}

fn fixture_view_manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/xhtml-review/view.json")
}

fn wyvern() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_wyvern"));
    cmd.env_remove("WYVERN_LOG");
    cmd.env_remove("WYVERN_SHARE");
    cmd.env_remove("WYVERN_VIEWER_BIN");
    cmd.env("WYVERN_VIEWER", "none");
    cmd
}

#[test]
fn extensions_xhtml_review_registered() {
    let registry = load_shipped();
    let ids: Vec<&str> = registry
        .extensions()
        .iter()
        .map(|ext| ext.id.as_str())
        .collect();
    assert!(
        ids.contains(&"report-xhtml-review"),
        "report-xhtml-review missing: {ids:?}"
    );
    let review_pos = ids
        .iter()
        .position(|id| *id == "report-xhtml-review")
        .expect("review");
    let view_pos = ids
        .iter()
        .position(|id| *id == "report-xhtml")
        .expect("view");
    assert!(
        review_pos < view_pos,
        "longer prefix must precede report-xhtml: {ids:?}"
    );
}

#[test]
fn extensions_xhtml_review_matches_review_flag() {
    let registry = load_shipped();
    let path = fixture_view_manifest();
    let argv = vec![
        "report-xhtml".to_string(),
        "--review".to_string(),
        path.to_string_lossy().into_owned(),
    ];
    let matched = registry
        .match_argv_with(&argv, &PresentProbe)
        .expect("review flag should match");
    assert_eq!(matched.extension().id.as_str(), "report-xhtml-review");
}

#[test]
fn extensions_xhtml_review_no_match_without_python3() {
    let registry = load_shipped();
    let argv = vec![
        "report-xhtml".to_string(),
        "--review".to_string(),
        fixture_review_manifest().to_string_lossy().into_owned(),
    ];
    assert!(registry.match_argv_with(&argv, &AbsentProbe).is_none());
}

#[test]
fn extensions_xhtml_review_expand_adds_footer() {
    if !PathRequiresProbe.binary_on_path("python3") {
        return;
    }
    let registry = load_shipped();
    let path = fixture_view_manifest();
    let argv = vec![
        "report-xhtml".to_string(),
        "--review".to_string(),
        path.to_string_lossy().into_owned(),
    ];
    let matched = registry
        .match_argv_with(&argv, &PresentProbe)
        .expect("must match");
    let ctx = build_match_context(&matched, matched.extension());
    let expanded = expand_and_validate(matched.extension(), &ctx).expect("expand");
    assert_eq!(expanded.command["type"], "report");
    assert_eq!(expanded.command["mode"], "review");
    assert_eq!(expanded.command["page"], "pages/view.xhtml");
    let panels = expanded.command["panels"].as_array().expect("panels");
    assert_eq!(panels.len(), 2);

    let ui_root = expanded.host_overrides.ui_root.expect("tmpdir ui_root");
    let html = std::fs::read_to_string(ui_root.join("pages/view.xhtml")).expect("html");
    assert!(
        html.contains("data-testid=\"report-review\"")
            && html.contains("data-testid=\"review-comments\"")
            && html.contains("data-testid=\"report-cancel\"")
            && html.contains("data-testid=\"report-approve\""),
        "review footer missing: {html}"
    );
    assert!(
        html.contains("/shared/report-review.js") && !html.contains("wizard-nav"),
        "review page must load report-review.js only: {html}"
    );
    assert!(
        html.contains("id=\"manifest-data\""),
        "embedded manifest missing: {html}"
    );
    assert!(
        html.contains("report--review"),
        "review frame class missing: {html}"
    );
}

#[test]
fn extensions_xhtml_review_manifest_mode_without_flag() {
    if !PathRequiresProbe.binary_on_path("python3") {
        return;
    }
    let registry = load_shipped();
    let path = fixture_review_manifest();
    let argv = vec![
        "report-xhtml".to_string(),
        path.to_string_lossy().into_owned(),
    ];
    let matched = registry
        .match_argv_with(&argv, &PresentProbe)
        .expect("must match report-xhtml");
    assert_eq!(matched.extension().id.as_str(), "report-xhtml");
    let ctx = build_match_context(&matched, matched.extension());
    let expanded = expand_and_validate(matched.extension(), &ctx).expect("expand");
    assert_eq!(expanded.command["mode"], "review");
    let html = std::fs::read_to_string(
        expanded
            .host_overrides
            .ui_root
            .expect("ui_root")
            .join("pages/view.xhtml"),
    )
    .expect("html");
    assert!(html.contains("data-testid=\"report-approve\""), "{html}");
}

#[test]
fn extensions_xhtml_view_has_no_review_footer() {
    if !PathRequiresProbe.binary_on_path("python3") {
        return;
    }
    let registry = load_shipped();
    let path = fixture_view_manifest();
    let argv = vec![
        "report-xhtml".to_string(),
        path.to_string_lossy().into_owned(),
    ];
    let matched = registry
        .match_argv_with(&argv, &PresentProbe)
        .expect("must match");
    let ctx = build_match_context(&matched, matched.extension());
    let expanded = expand_and_validate(matched.extension(), &ctx).expect("expand");
    assert_eq!(expanded.command["mode"], "view");
    let html = std::fs::read_to_string(
        expanded
            .host_overrides
            .ui_root
            .expect("ui_root")
            .join("pages/view.xhtml"),
    )
    .expect("html");
    assert!(
        !html.contains("report-review") && !html.contains("report-review.js"),
        "view-only must omit review footer: {html}"
    );
}

#[test]
fn extensions_xhtml_review_cli_finish_exits_zero() {
    let path = fixture_view_manifest();
    let tmp = tempfile::tempdir().expect("tempdir");
    let url_file = tmp.path().join("dialog-url");
    let child = wyvern()
        .args(["report-xhtml", "--review", path.to_str().expect("utf8")])
        .env("WYVERN_DIALOG_URL_FILE", &url_file)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn wyvern");

    let dialog_url = wait_for_url_file(&url_file);
    assert!(
        dialog_url.contains("/report/pages/view.xhtml"),
        "dialog URL: {dialog_url}"
    );

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("http client");
    let page = wait_for_get(&client, &dialog_url);
    assert!(
        page.contains("data-testid=\"report-approve\"") && page.contains("id=\"manifest-data\""),
        "expected review page: {page}"
    );
    assert!(!page.contains("wizard-nav"), "wizard-nav leaked: {page}");

    let finish_url = dialog_url
        .split_once("/report/")
        .map(|(base, _)| format!("{base}/api/report/finish"))
        .expect("report URL");
    let ack = client
        .post(&finish_url)
        .json(&serde_json::json!({
            "approved": true,
            "comments": "from cli test",
            "panels": [
                { "path": "panels/fail.xhtml", "label": "Fail 1", "role": "failure" },
                { "path": "panels/proposed-fix.xhtml", "label": "Proposed fix", "role": "proposal" }
            ]
        }))
        .send()
        .expect("POST finish");
    assert!(
        ack.status().is_success(),
        "finish POST failed: {}",
        ack.status()
    );

    let output = child.wait_with_output().expect("wait wyvern");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(0),
        "expected exit 0; stdout={stdout} stderr={stderr}"
    );
    let value: serde_json::Value = serde_json::from_str(stdout.trim()).expect("stdout JSON");
    assert_eq!(value["button"], "finish");
    assert_eq!(value["data"]["approved"], true);
    assert_eq!(value["data"]["comments"], "from cli test");
    assert_eq!(value["data"]["panels"].as_array().map(Vec::len), Some(2));
}

#[test]
fn extensions_xhtml_review_help_documents_flag() {
    let output = wyvern()
        .args(["report-xhtml", "--review", "--help"])
        .output()
        .expect("spawn wyvern");
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("report-xhtml-review") || stdout.contains("--review"),
        "{stdout}"
    );
}

fn wait_for_url_file(path: &Path) -> String {
    let start = std::time::Instant::now();
    loop {
        if let Ok(url) = std::fs::read_to_string(path) {
            let url = url.trim().to_string();
            if !url.is_empty() {
                return url;
            }
        }
        if start.elapsed() > Duration::from_secs(20) {
            panic!("timed out waiting for dialog URL file {}", path.display());
        }
        thread::sleep(Duration::from_millis(20));
    }
}

fn wait_for_get(client: &reqwest::blocking::Client, url: &str) -> String {
    let start = std::time::Instant::now();
    loop {
        match client.get(url).send() {
            Ok(resp) if resp.status().is_success() => {
                return resp.text().expect("html");
            }
            Ok(_) | Err(_) => {
                if start.elapsed() > Duration::from_secs(15) {
                    panic!("timed out waiting for GET {url}");
                }
                thread::sleep(Duration::from_millis(20));
            }
        }
    }
}
