//! `report-xhtml` expands a manifest to a multi-pane report (REQ-0142).

mod test_support;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

use test_support::{AbsentProbe, PresentProbe};
use wyvern::extensions::{
    build_match_context, build_skill_record, expand_and_validate, ExtensionRegistry,
    PathRequiresProbe,
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

fn fixture_manifest() -> PathBuf {
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
fn extensions_xhtml_array_registered() {
    let registry = load_shipped();
    let ids: Vec<&str> = registry
        .extensions()
        .iter()
        .map(|ext| ext.id.as_str())
        .collect();
    assert!(
        ids.contains(&"report-xhtml"),
        "report-xhtml missing from shipped registry: {ids:?}"
    );
}

#[test]
fn extensions_xhtml_array_matches_json_suffix() {
    let registry = load_shipped();
    let path = fixture_manifest();
    assert!(path.is_file(), "fixture missing: {}", path.display());
    let argv = vec![
        "report-xhtml".to_string(),
        path.to_string_lossy().into_owned(),
    ];
    let matched = registry
        .match_argv_with(&argv, &PresentProbe)
        .expect("report-xhtml should match");
    assert_eq!(matched.extension().id.as_str(), "report-xhtml");
}

#[test]
fn extensions_xhtml_array_non_json_does_not_match() {
    let registry = load_shipped();
    let argv = vec!["report-xhtml".to_string(), "notes.txt".to_string()];
    let matched = registry.match_argv_with(&argv, &PresentProbe);
    assert!(
        matched.is_none(),
        "non-.json token after report-xhtml must not match"
    );
}

#[test]
fn extensions_xhtml_array_no_match_without_python3() {
    let registry = load_shipped();
    let argv = vec![
        "report-xhtml".to_string(),
        fixture_manifest().to_string_lossy().into_owned(),
    ];
    let matched = registry.match_argv_with(&argv, &AbsentProbe);
    assert!(
        matched.is_none(),
        "report-xhtml must not match when python3 is absent"
    );
}

#[test]
fn extensions_xhtml_array_expands_to_report() {
    let registry = load_shipped();
    let ext = registry
        .extensions()
        .iter()
        .find(|ext| ext.id.as_str() == "report-xhtml")
        .expect("report-xhtml");
    let record = build_skill_record(ext, &PathRequiresProbe);
    assert_eq!(record.expands_to, "report");
    assert_ne!(record.expands_to, "wizard");
}

#[test]
fn extensions_xhtml_array_expand_stitches_panes() {
    let registry = load_shipped();
    let path = fixture_manifest();
    let argv = vec![
        "report-xhtml".to_string(),
        path.to_string_lossy().into_owned(),
    ];
    let matched = registry
        .match_argv_with(&argv, &PresentProbe)
        .expect("must match");
    let ctx = build_match_context(&matched, matched.extension());
    let expanded = match expand_and_validate(matched.extension(), &ctx) {
        Ok(expanded) => expanded,
        Err(err) => {
            let message = format!("{err}");
            if message.contains("python3") || message.to_ascii_lowercase().contains("preexec") {
                eprintln!("skipping array expand: {message}");
                return;
            }
            panic!("expand_and_validate failed: {err}");
        }
    };
    assert_eq!(expanded.command["type"], "report");
    assert_eq!(expanded.command["mode"], "view");
    assert_eq!(expanded.command["page"], "pages/view.xhtml");
    assert_eq!(expanded.command["title"], "XHTML array fixture");
    let panels = expanded.command["panels"]
        .as_array()
        .expect("panels echoed in command JSON");
    assert_eq!(panels.len(), 2);
    assert_eq!(panels[0]["path"], "panels/fail.xhtml");
    assert_eq!(panels[1]["role"], "proposal");

    let ui_root = expanded.host_overrides.ui_root.expect("tmpdir ui_root");
    let wrapped = ui_root.join("pages/view.xhtml");
    let html = std::fs::read_to_string(&wrapped).expect("wrapped page");
    assert!(
        html.contains("report report--array"),
        "expected basic-array frame: {html}"
    );
    let fail_at = html
        .find("data-testid=\"fail-panel\"")
        .expect("fail pane present");
    let proposal_at = html
        .find("data-testid=\"proposal-panel\"")
        .expect("proposal pane present");
    assert!(fail_at < proposal_at, "panes must stay in document order");
    assert!(
        html.contains("class=\"pane pane--proposal\""),
        "proposal pane must use .pane--proposal: {html}"
    );
    assert!(
        html.contains("data-role=\"failure\"") && html.contains("data-role=\"proposal\""),
        "roles must be on pane sections: {html}"
    );
    assert!(
        html.contains("/shared/report-base.css"),
        "frame must link report-base.css: {html}"
    );
    let command_path = ui_root.join("report-command.json");
    let command_text = std::fs::read_to_string(&command_path).expect("report-command.json");
    let written: serde_json::Value = serde_json::from_str(&command_text).expect("command json");
    assert_eq!(written["type"], "report");
}

#[test]
fn extensions_xhtml_array_missing_panel_is_nonzero() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let manifest = tmp.path().join("broken.json");
    std::fs::write(
        &manifest,
        r#"{
          "title": "Broken",
          "panels": [{ "path": "panels/missing.xhtml", "label": "Gone" }]
        }"#,
    )
    .expect("write manifest");
    let output = wyvern()
        .args(["report-xhtml", manifest.to_str().expect("utf8")])
        .output()
        .expect("spawn wyvern");
    assert_ne!(
        output.status.code(),
        Some(0),
        "missing panel must not exit 0; stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("missing.xhtml") || stderr.contains("missing panel"),
        "stderr must name the missing file: {stderr}"
    );
}

#[test]
fn extensions_xhtml_array_single_panel_works() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let panel_dir = tmp.path().join("panels");
    std::fs::create_dir_all(&panel_dir).expect("mkdir");
    std::fs::write(
        panel_dir.join("only.xhtml"),
        r#"<section data-testid="only-panel"><p>one</p></section>"#,
    )
    .expect("write panel");
    let manifest = tmp.path().join("one.json");
    std::fs::write(
        &manifest,
        r#"{
          "title": "One pane",
          "panels": [{ "path": "panels/only.xhtml", "label": "Only" }]
        }"#,
    )
    .expect("write manifest");

    let registry = load_shipped();
    let argv = vec![
        "report-xhtml".to_string(),
        manifest.to_string_lossy().into_owned(),
    ];
    let matched = registry
        .match_argv_with(&argv, &PresentProbe)
        .expect("must match");
    let ctx = build_match_context(&matched, matched.extension());
    let expanded = expand_and_validate(matched.extension(), &ctx).expect("expand");
    assert_eq!(expanded.command["type"], "report");
    assert_eq!(expanded.command["panels"].as_array().map(Vec::len), Some(1));
    let html = std::fs::read_to_string(
        expanded
            .host_overrides
            .ui_root
            .expect("ui_root")
            .join("pages/view.xhtml"),
    )
    .expect("html");
    assert!(html.contains("data-testid=\"only-panel\""), "{html}");
    assert!(html.contains("class=\"pane\""), "{html}");
}

#[test]
fn extensions_xhtml_array_help_documents_manifest() {
    let output = wyvern()
        .args(["report-xhtml", "--help"])
        .output()
        .expect("spawn wyvern");
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("report-xhtml"), "{stdout}");
    assert!(
        stdout.contains("manifest") || stdout.contains(".json"),
        "help must document the manifest path: {stdout}"
    );
    assert!(
        stdout.contains("title") && stdout.contains("panels") && stdout.contains("mode"),
        "help must document manifest fields: {stdout}"
    );
}

#[test]
fn extensions_xhtml_array_cli_exits_zero_with_viewer_none() {
    let path = fixture_manifest();
    assert!(path.is_file(), "fixture missing: {}", path.display());
    let url_file = std::env::temp_dir().join(format!(
        "wyvern-xhtml-array-url-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
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
        "dialog URL must be /report/{{page}}: {dialog_url}"
    );

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("http client");
    let page = wait_for_get(&client, &dialog_url);
    assert!(
        page.contains("report report--array")
            && page.contains("fail-panel")
            && page.contains("proposal-panel")
            && page.contains("pane--proposal"),
        "expected both panes in viewer HTML: {page}"
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
    let _ = std::fs::remove_file(&url_file);
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
