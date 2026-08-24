//! L1: report bind URL `/report/{page}` and shared CSS (REQ-HOST-0140 / 0141).

mod support;
use support::http::{http_client, wait_for_url_file};

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

use wyvern_host::{begin, HostOptions, ViewerMode};
use wyvern_schema::{Command, ReportCommand, ReportMode, ReportPagePath, ReportTitle};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn workspace_ui_root() -> PathBuf {
    workspace_root().join("ui")
}

fn unique_path(prefix: &str) -> PathBuf {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("{prefix}-{}-{n}", std::process::id()))
}

fn report_command() -> Command {
    Command::Report(ReportCommand {
        title: ReportTitle::new("XHTML review"),
        page: ReportPagePath::new("pages/view.xhtml"),
        mode: ReportMode::View,
        panels: None,
        width: None,
        height: None,
    })
}

fn write_report_ui_root() -> PathBuf {
    let root = unique_path("wyvern-report-ui");
    std::fs::create_dir_all(root.join("pages")).expect("pages");
    std::fs::write(
        root.join("pages").join("view.xhtml"),
        "<!DOCTYPE html><html><body class=\"report report--single\"><main class=\"report-body\"><section data-testid=\"sc-compose-fragment\">ok</section></main></body></html>",
    )
    .expect("write page");
    root
}

fn wait_for_get(client: &reqwest::blocking::Client, url: &str) -> reqwest::blocking::Response {
    let start = std::time::Instant::now();
    loop {
        match client.get(url).send() {
            Ok(resp) if resp.status().is_success() => return resp,
            Ok(_) | Err(_) => {
                if start.elapsed() > Duration::from_secs(15) {
                    panic!("timed out waiting for GET {url}");
                }
                thread::sleep(Duration::from_millis(20));
            }
        }
    }
}

#[test]
fn report_view_bind_url_resolves_page_and_shared_css() {
    let url_file = unique_path("wyvern-report-view-url");
    let ui_root = write_report_ui_root();
    let options = HostOptions {
        bind: SocketAddr::from(([127, 0, 0, 1], 0)),
        ui_root,
        shared_ui_root: workspace_ui_root(),
        viewer: ViewerMode::None,
        dialog_url_env: true,
        dialog_url_file: Some(url_file.clone()),
        allow_non_loopback: false,
        session_timeout: Duration::from_secs(30),
        mock_picker: None,
    };
    let handle = begin(report_command(), options).expect("begin");
    let dialog_url = wait_for_url_file(&url_file);
    assert!(
        dialog_url.contains("/report/pages/view.xhtml"),
        "expected /report/{{page}} bind URL, got {dialog_url}"
    );
    assert!(
        !dialog_url.contains("/wizard/"),
        "report must not use wizard URLs: {dialog_url}"
    );

    let client = http_client();
    let page = wait_for_get(&client, &dialog_url).text().expect("html");
    assert!(
        page.contains("sc-compose-fragment"),
        "expected report page body: {page}"
    );

    let base = dialog_url
        .split_once("/report/")
        .map(|(b, _)| b.to_string())
        .expect("report path");
    let css = wait_for_get(&client, &format!("{base}/shared/report-base.css"))
        .text()
        .expect("css");
    assert!(
        css.contains(".report-body") || css.contains("body.report"),
        "expected report-base.css: {css}"
    );

    let dialog = client
        .get(format!("{base}/api/dialog"))
        .send()
        .expect("GET /api/dialog");
    assert!(
        !dialog.status().is_success(),
        "GET /api/dialog must reject report sessions, got {}",
        dialog.status()
    );

    let _ = handle.viewer_exited_without_result();
    let _ = std::fs::remove_file(&url_file);
}

#[test]
fn report_view_invalid_result_recovery_mentions_dismissed() {
    let url_file = unique_path("wyvern-report-result-url");
    let ui_root = write_report_ui_root();
    let options = HostOptions {
        bind: SocketAddr::from(([127, 0, 0, 1], 0)),
        ui_root,
        shared_ui_root: workspace_ui_root(),
        viewer: ViewerMode::None,
        dialog_url_env: true,
        dialog_url_file: Some(url_file.clone()),
        allow_non_loopback: false,
        session_timeout: Duration::from_secs(30),
        mock_picker: None,
    };
    let handle = begin(report_command(), options).expect("begin");
    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_url
        .split_once("/report/")
        .map(|(b, _)| b.to_string())
        .expect("report path");

    let client = http_client();
    let response = client
        .post(format!("{base}/api/result"))
        .json(&serde_json::json!({}))
        .send()
        .expect("POST result");
    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
    let body: serde_json::Value = response.json().expect("error json");
    let recovery = body["recovery"].as_array().expect("recovery");
    assert!(
        recovery.iter().any(|s| {
            s.as_str()
                .is_some_and(|step| step.contains(r#"{"button":"dismissed"}"#))
        }),
        "report dismiss recovery missing: {body}"
    );

    let _ = client
        .post(format!("{base}/api/result"))
        .json(&serde_json::json!({"button": "dismissed"}))
        .send()
        .expect("POST dismiss");
    let result = handle.await_result().expect("await result");
    assert_eq!(
        serde_json::to_string(&result).expect("serialize"),
        r#"{"button":"dismissed"}"#
    );
    let _ = std::fs::remove_file(&url_file);
}
