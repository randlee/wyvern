//! Regression for issue #115: `.xhtml` matches `xhtml-suffix` and expands to a report.
//!
//! Headless `WYVERN_VIEWER=none` must exit 0. The expanded command is
//! `"type":"report"` — not `PARSE_ERROR` and not "No shipped extension matches".

mod test_support;

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use test_support::PresentProbe;
use wyvern::extensions::{build_match_context, expand_command_host, ExtensionRegistry};

const PARSE_ERROR: &str = "PARSE_ERROR";
const NO_MATCH: &str = "No shipped extension matches";

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root")
}

fn load_shipped() -> ExtensionRegistry {
    let defaults = workspace_root().join("share/wyvern/extensions.json");
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

fn assert_no_closeout_failure(label: &str, text: &str) {
    assert!(
        !text.contains(PARSE_ERROR),
        "{label} must not contain {PARSE_ERROR}: {text}"
    );
    assert!(
        !text.contains(NO_MATCH),
        "{label} must not contain {NO_MATCH}: {text}"
    );
}

#[test]
fn extensions_xhtml_suffix_matches_fixture() {
    let path = fixture_panel();
    assert!(path.is_file(), "fixture missing: {}", path.display());

    let registry = load_shipped();
    let argv = vec![path.to_string_lossy().into_owned()];
    let matched = registry
        .match_argv_with(&argv, &PresentProbe)
        .expect("xhtml-suffix should match .xhtml");
    assert_eq!(matched.extension().id.as_str(), "xhtml-suffix");
}

#[test]
fn extensions_xhtml_suffix_expand_command_is_report() {
    let path = fixture_panel();
    let registry = load_shipped();
    let argv = vec![path.to_string_lossy().into_owned()];
    let matched = registry
        .match_argv_with(&argv, &PresentProbe)
        .expect("xhtml-suffix should match");
    let mut ctx = build_match_context(&matched, matched.extension());
    ctx.tmpdir = Some(std::env::temp_dir().join(format!(
        "wyvern-xhtml-suffix-expand-{}-{}",
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
    assert!(host.ui_root.is_some());

    let rendered = serde_json::to_string(&command).expect("serialize expanded command");
    assert!(
        rendered.contains("\"type\":\"report\""),
        "expanded JSON must include type report: {rendered}"
    );
    assert_no_closeout_failure("expanded command", &rendered);
}

#[test]
fn extensions_xhtml_suffix_headless_viewer_none_exits_zero() {
    let path = fixture_panel();
    assert!(path.is_file(), "fixture missing: {}", path.display());

    let url_file = std::env::temp_dir().join(format!(
        "wyvern-xhtml-suffix-url-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let child = wyvern()
        .arg(path.as_os_str())
        .env("WYVERN_DIALOG_URL_FILE", &url_file)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn wyvern");

    let dialog_url = wait_for_url_file(&url_file);
    assert!(
        dialog_url.contains("/report/pages/view.xhtml"),
        "headless path must serve the report page: {dialog_url}"
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
        "WYVERN_VIEWER=none wyvern <fixture>.xhtml must exit 0; stdout={stdout} stderr={stderr}"
    );
    assert_no_closeout_failure("stdout", &stdout);
    assert_no_closeout_failure("stderr", &stderr);
    let value: serde_json::Value = serde_json::from_str(stdout.trim()).expect("stdout JSON");
    assert_eq!(value["button"], "dismissed");
    let _ = std::fs::remove_file(&url_file);
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
