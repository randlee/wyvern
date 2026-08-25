//! Synthetic `share/wyvern/examples/xhtml-review/` tree (Phase H h.5).

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

fn example_dir() -> PathBuf {
    workspace_root().join("share/wyvern/examples/xhtml-review")
}

fn packaged_example_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("share/wyvern/examples/xhtml-review")
}

fn embedded_example_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("embedded/share/wyvern/examples/xhtml-review")
}

fn load_shipped() -> ExtensionRegistry {
    let defaults = workspace_root().join("share/wyvern/extensions.json");
    ExtensionRegistry::load(&defaults, None).expect("shipped registry")
}

fn wyvern() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_wyvern"));
    cmd.env_remove("WYVERN_LOG");
    cmd.env_remove("WYVERN_SHARE");
    cmd.env_remove("WYVERN_VIEWER_BIN");
    cmd.env("WYVERN_VIEWER", "none");
    cmd
}

fn example_rel_paths() -> &'static [&'static str] {
    &[
        "README.md",
        "review-view.json",
        "review-review.json",
        "panels/fail-1.xhtml",
        "panels/fail-2.xhtml",
        "panels/fail-3.xhtml",
        "panels/proposed-fix.xhtml",
    ]
}

fn read_tree_files(root: &Path) -> String {
    let mut blob = String::new();
    for rel in example_rel_paths() {
        let path = root.join(rel);
        blob.push_str(&std::fs::read_to_string(&path).unwrap_or_else(|err| {
            panic!("read {}: {err}", path.display());
        }));
        blob.push('\n');
    }
    blob
}

#[test]
fn examples_xhtml_review_tree_exists() {
    let root = example_dir();
    for rel in example_rel_paths() {
        let path = root.join(rel);
        assert!(path.is_file(), "missing example file: {}", path.display());
    }
}

#[test]
fn examples_xhtml_review_has_no_wizard_nav() {
    let blob = read_tree_files(&example_dir());
    assert!(
        !blob.contains("wizard-nav"),
        "report example must not reference wizard-nav"
    );
}

#[test]
fn examples_xhtml_review_panels_are_synthetic_benchmark_runs() {
    let root = example_dir();
    let fail_1 = std::fs::read_to_string(root.join("panels/fail-1.xhtml")).expect("fail-1");
    let fail_2 = std::fs::read_to_string(root.join("panels/fail-2.xhtml")).expect("fail-2");
    let fail_3 = std::fs::read_to_string(root.join("panels/fail-3.xhtml")).expect("fail-3");
    let proposal =
        std::fs::read_to_string(root.join("panels/proposed-fix.xhtml")).expect("proposed-fix");

    for (name, html) in [
        ("fail-1", fail_1.as_str()),
        ("fail-2", fail_2.as_str()),
        ("fail-3", fail_3.as_str()),
        ("proposed-fix", proposal.as_str()),
    ] {
        assert!(
            html.contains(r#"xmlns="http://www.w3.org/1999/xhtml""#)
                && html.contains(r#"class="benchmark-run""#),
            "{name} must be an atm-core-style benchmark-run fragment: {html}"
        );
    }

    assert!(
        fail_1.contains("FAIL") && fail_1.contains("12480"),
        "fail-1 must show fabricated FAIL numbers: {fail_1}"
    );
    assert!(
        fail_2.contains("FAIL") && fail_2.contains("47.2"),
        "fail-2 must show fabricated FAIL numbers: {fail_2}"
    );
    assert!(
        fail_3.contains("FAIL") && fail_3.contains("2 / 5"),
        "fail-3 must show fabricated FAIL numbers: {fail_3}"
    );
    assert!(
        proposal.contains("PASS")
            && proposal.contains("22140")
            && !proposal.contains("data-status=\"FAIL\""),
        "proposed-fix must be a visible PASS with revised numbers: {proposal}"
    );
    assert!(
        !fail_1.contains("22140") && !fail_2.contains("22140") && !fail_3.contains("22140"),
        "fail panels must not reuse the proposal admissions/s figure"
    );
}

#[test]
fn examples_xhtml_review_share_parity() {
    for rel in example_rel_paths() {
        let workspace = std::fs::read_to_string(example_dir().join(rel)).expect("workspace");
        let packaged = std::fs::read_to_string(packaged_example_dir().join(rel)).expect("packaged");
        let embedded = std::fs::read_to_string(embedded_example_dir().join(rel)).expect("embedded");
        assert_eq!(
            workspace, packaged,
            "crates/wyvern/share must track share/wyvern/examples/xhtml-review/{rel}"
        );
        assert_eq!(
            workspace, embedded,
            "crates/wyvern/embedded/share must track share/wyvern/examples/xhtml-review/{rel}"
        );
    }
}

#[test]
fn examples_xhtml_review_readme_documents_single_panel_shortcut() {
    let readme = std::fs::read_to_string(example_dir().join("README.md")).expect("README");
    assert!(
        readme.contains("wyvern share/wyvern/examples/xhtml-review/panels/fail-1.xhtml"),
        "README must document the single-panel shortcut: {readme}"
    );
}

#[test]
fn examples_xhtml_review_no_match_without_python3() {
    let registry = load_shipped();
    let argv = vec![
        "report-xhtml".to_string(),
        example_dir()
            .join("review-view.json")
            .to_string_lossy()
            .into_owned(),
    ];
    assert!(
        registry.match_argv_with(&argv, &AbsentProbe).is_none(),
        "report-xhtml must not match when python3 is absent"
    );
}

#[test]
fn examples_xhtml_review_view_expands() {
    if !PathRequiresProbe.binary_on_path("python3") {
        return;
    }
    let registry = load_shipped();
    let path = example_dir().join("review-view.json");
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
    assert_eq!(expanded.command["type"], "report");
    assert_eq!(expanded.command["mode"], "view");
    assert_eq!(expanded.command["page"], "pages/view.xhtml");
    assert_eq!(expanded.command["panels"].as_array().map(Vec::len), Some(4));

    let html = std::fs::read_to_string(
        expanded
            .host_overrides
            .ui_root
            .expect("ui_root")
            .join("pages/view.xhtml"),
    )
    .expect("html");
    assert!(
        html.contains("report report--array")
            && html.contains("data-testid=\"fail-1\"")
            && html.contains("data-testid=\"proposed-fix\"")
            && html.contains("class=\"pane pane--proposal\"")
            && html.contains("PASS"),
        "view frame must stitch fail + PASS proposal: {html}"
    );
    assert!(!html.contains("wizard-nav"), "wizard-nav leaked: {html}");
    assert!(
        !html.contains("report-review.js"),
        "view-only must omit review footer: {html}"
    );
}

#[test]
fn examples_xhtml_review_review_expands() {
    if !PathRequiresProbe.binary_on_path("python3") {
        return;
    }
    let registry = load_shipped();
    let path = example_dir().join("review-review.json");
    let argv = vec![
        "report-xhtml".to_string(),
        "--review".to_string(),
        path.to_string_lossy().into_owned(),
    ];
    let matched = registry
        .match_argv_with(&argv, &PresentProbe)
        .expect("must match report-xhtml-review");
    assert_eq!(matched.extension().id.as_str(), "report-xhtml-review");
    let ctx = build_match_context(&matched, matched.extension());
    let expanded = expand_and_validate(matched.extension(), &ctx).expect("expand");
    assert_eq!(expanded.command["type"], "report");
    assert_eq!(expanded.command["mode"], "review");
    assert_eq!(expanded.command["panels"].as_array().map(Vec::len), Some(4));

    let html = std::fs::read_to_string(
        expanded
            .host_overrides
            .ui_root
            .expect("ui_root")
            .join("pages/view.xhtml"),
    )
    .expect("html");
    assert!(
        html.contains("data-testid=\"report-approve\"")
            && html.contains("/shared/report-review.js")
            && html.contains("id=\"manifest-data\""),
        "review footer missing: {html}"
    );
    assert!(!html.contains("wizard-nav"), "wizard-nav leaked: {html}");
}

#[test]
fn examples_xhtml_review_cli_view_exits_zero() {
    let path = example_dir().join("review-view.json");
    let tmp = tempfile::tempdir().expect("tempdir");
    let url_file = tmp.path().join("dialog-url");
    let child = wyvern()
        .args(["report-xhtml", path.to_str().expect("utf8")])
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
        page.contains("data-testid=\"fail-1\"")
            && page.contains("data-testid=\"proposed-fix\"")
            && page.contains("PASS"),
        "expected synthetic panes: {page}"
    );

    let result_url = dialog_url
        .split_once("/report/")
        .map(|(base, _)| format!("{base}/api/result"))
        .expect("report URL");
    let ack = client
        .post(&result_url)
        .json(&serde_json::json!({"button": "dismissed"}))
        .send()
        .expect("POST /api/result");
    assert!(
        ack.status().is_success(),
        "dismiss POST failed: {}",
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
    assert_eq!(value["button"], "dismissed");
}

#[test]
fn examples_xhtml_review_cli_review_finish_exits_zero() {
    let path = example_dir().join("review-review.json");
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
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("http client");
    let page = wait_for_get(&client, &dialog_url);
    assert!(
        page.contains("data-testid=\"report-approve\"") && page.contains("id=\"manifest-data\""),
        "expected review page: {page}"
    );

    let finish_url = dialog_url
        .split_once("/report/")
        .map(|(base, _)| format!("{base}/api/report/finish"))
        .expect("report URL");
    let ack = client
        .post(&finish_url)
        .json(&serde_json::json!({
            "approved": true,
            "comments": "synthetic example finish",
            "panels": [
                { "path": "panels/fail-1.xhtml", "label": "Fail 1", "role": "failure" },
                { "path": "panels/fail-2.xhtml", "label": "Fail 2", "role": "failure" },
                { "path": "panels/fail-3.xhtml", "label": "Fail 3", "role": "failure" },
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
    assert_eq!(value["data"]["comments"], "synthetic example finish");
    assert_eq!(value["data"]["panels"].as_array().map(Vec::len), Some(4));
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
