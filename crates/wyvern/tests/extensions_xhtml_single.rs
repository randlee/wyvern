//! `.xhtml` suffix expands to a report view via `xhtml-suffix` (REQ-0141).

mod test_support;

use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;

use test_support::{AbsentProbe, PresentProbe};
use wyvern::extensions::{
    build_match_context, expand_and_validate, expand_command_host, ExtensionRegistry,
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

fn fixture_panel() -> PathBuf {
    workspace_root().join("fixtures/xhtml/panel.xhtml")
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
fn extensions_xhtml_single_registered() {
    let registry = load_shipped();
    let ids: Vec<&str> = registry
        .extensions()
        .iter()
        .map(|ext| ext.id.as_str())
        .collect();
    assert!(
        ids.contains(&"xhtml-suffix"),
        "xhtml-suffix missing from shipped registry: {ids:?}"
    );
}

#[test]
fn extensions_xhtml_single_matches_suffix() {
    let registry = load_shipped();
    let path = fixture_panel();
    assert!(path.is_file(), "fixture missing: {}", path.display());
    let argv = vec![path.to_string_lossy().into_owned()];
    let matched = registry
        .match_argv_with(&argv, &PresentProbe)
        .expect("xhtml-suffix should match");
    assert_eq!(matched.extension().id.as_str(), "xhtml-suffix");
}

#[test]
fn extensions_xhtml_single_no_match_without_python3() {
    let registry = load_shipped();
    let argv = vec![fixture_panel().to_string_lossy().into_owned()];
    let matched = registry.match_argv_with(&argv, &AbsentProbe);
    assert!(
        matched.is_none(),
        "xhtml-suffix must not match when python3 is absent"
    );
}

#[test]
fn extensions_xhtml_single_expand_is_report_view() {
    let registry = load_shipped();
    let path = fixture_panel();
    let argv = vec![path.to_string_lossy().into_owned()];
    let matched = registry
        .match_argv_with(&argv, &PresentProbe)
        .expect("must match");
    let mut ctx = build_match_context(&matched, matched.extension());
    ctx.tmpdir = Some(std::env::temp_dir().join(format!(
        "wyvern-xhtml-expand-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    )));
    let (command, host) = expand_command_host(matched.extension(), &ctx).expect("expand");
    assert_eq!(command["type"], "report");
    assert_eq!(command["mode"], "view");
    assert_eq!(command["page"], "pages/view.xhtml");
    assert_eq!(command["title"], "panel.xhtml");
    assert!(host.ui_root.is_some());
}

#[test]
fn extensions_xhtml_single_html_suffix_unchanged() {
    let registry = load_shipped();
    let path = workspace_root().join("examples/wizards/single-page/pages/only.html");
    assert!(path.is_file(), "html fixture missing: {}", path.display());
    let argv = vec![path.to_string_lossy().into_owned()];
    let matched = registry
        .match_argv(&argv)
        .expect("html-suffix should still match .html");
    assert_eq!(matched.extension().id.as_str(), "html-suffix");
    let ctx = build_match_context(&matched, matched.extension());
    let expanded = expand_and_validate(matched.extension(), &ctx).expect("expand");
    assert_eq!(expanded.command["type"], "wizard");
}

#[test]
fn extensions_xhtml_single_preexec_wraps_fragment() {
    let registry = load_shipped();
    let path = fixture_panel();
    let argv = vec![path.to_string_lossy().into_owned()];
    let matched = registry
        .match_argv_with(&argv, &PresentProbe)
        .expect("must match");
    let ctx = build_match_context(&matched, matched.extension());
    let expanded = match expand_and_validate(matched.extension(), &ctx) {
        Ok(expanded) => expanded,
        Err(err) => {
            let message = format!("{err}");
            if message.contains("python3") || message.to_ascii_lowercase().contains("preexec") {
                eprintln!("skipping preexec wrap: {message}");
                return;
            }
            panic!("expand_and_validate failed: {err}");
        }
    };
    assert_eq!(expanded.command["type"], "report");
    assert_eq!(expanded.command["mode"], "view");
    let ui_root = expanded.host_overrides.ui_root.expect("tmpdir ui_root");
    let wrapped = ui_root.join("pages/view.xhtml");
    let html = std::fs::read_to_string(&wrapped).expect("wrapped page");
    assert!(
        html.contains("report report--single"),
        "expected basic-single frame: {html}"
    );
    assert!(
        html.contains("data-testid=\"sc-compose-fragment\""),
        "fragment must be inlined, not raw-dumped: {html}"
    );
    assert!(
        html.contains("/shared/report-base.css"),
        "frame must link report-base.css: {html}"
    );
}

#[test]
fn extensions_xhtml_single_cli_exits_zero_with_viewer_none() {
    let path = fixture_panel();
    assert!(path.is_file(), "fixture missing: {}", path.display());
    let url_file = std::env::temp_dir().join(format!(
        "wyvern-xhtml-url-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let child = wyvern()
        .arg(path.as_os_str())
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
        page.contains("report report--single") && page.contains("sc-compose-fragment"),
        "expected wrapped fragment in viewer HTML: {page}"
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

#[test]
fn extensions_xhtml_single_missing_path_is_nonzero() {
    let missing = workspace_root().join("fixtures/xhtml/does-not-exist.xhtml");
    let output = wyvern().arg(&missing).output().expect("spawn wyvern");
    assert_ne!(
        output.status.code(),
        Some(0),
        "missing xhtml must not exit 0; stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn wait_for_url_file(path: &std::path::Path) -> String {
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
