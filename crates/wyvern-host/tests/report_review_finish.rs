//! L1: review-mode `POST /api/report/finish` contract (REQ-HOST-0142 / REQ-0144).

mod support;
use support::http::{http_client, wait_for_url_file};

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

use wyvern_host::{begin, DialogHandle, HostOptions, ViewerMode};
use wyvern_schema::{
    Command, ManifestPanelPath, PanelRole, ReportCommand, ReportMode, ReportPagePath,
    ReportPanelEntry, ReportTitle,
};

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

fn review_panels() -> Vec<ReportPanelEntry> {
    vec![
        ReportPanelEntry {
            path: ManifestPanelPath::new("panels/fail.xhtml"),
            label: Some("Fail 1".into()),
            role: Some(PanelRole::Failure),
        },
        ReportPanelEntry {
            path: ManifestPanelPath::new("panels/proposed-fix.xhtml"),
            label: Some("Proposed fix".into()),
            role: Some(PanelRole::Proposal),
        },
    ]
}

fn report_command(mode: ReportMode, panels: Option<Vec<ReportPanelEntry>>) -> Command {
    Command::Report(ReportCommand {
        title: ReportTitle::new("XHTML review"),
        page: ReportPagePath::new("pages/view.xhtml"),
        mode,
        panels,
        width: None,
        height: None,
    })
}

fn write_report_ui_root() -> PathBuf {
    let root = unique_path("wyvern-report-review-ui");
    std::fs::create_dir_all(root.join("pages")).expect("pages");
    std::fs::write(
        root.join("pages").join("view.xhtml"),
        "<!DOCTYPE html><html><body class=\"report report--review\"><main>ok</main></body></html>",
    )
    .expect("write page");
    root
}

fn host_options(ui_root: PathBuf, url_file: PathBuf, timeout: Duration) -> HostOptions {
    HostOptions {
        bind: SocketAddr::from(([127, 0, 0, 1], 0)),
        ui_root,
        shared_ui_root: workspace_ui_root(),
        viewer: ViewerMode::None,
        dialog_url_env: true,
        dialog_url_file: Some(url_file),
        allow_non_loopback: false,
        session_timeout: timeout,
        mock_picker: None,
    }
}

fn start_report(
    mode: ReportMode,
    panels: Option<Vec<ReportPanelEntry>>,
    timeout: Duration,
) -> (DialogHandle, String, PathBuf, reqwest::blocking::Client) {
    let ui_root = write_report_ui_root();
    let url_file = unique_path("wyvern-report-review-url");
    let handle = begin(
        report_command(mode, panels),
        host_options(ui_root, url_file.clone(), timeout),
    )
    .expect("begin");
    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_url
        .split_once("/report/")
        .map(|(b, _)| b.to_string())
        .expect("report path");
    let client = http_client();
    wait_for_get(&client, &dialog_url);
    (handle, base, url_file, client)
}

fn wait_for_get(client: &reqwest::blocking::Client, url: &str) {
    let start = std::time::Instant::now();
    loop {
        match client.get(url).send() {
            Ok(resp) if resp.status().is_success() => return,
            Ok(_) | Err(_) => {
                if start.elapsed() > Duration::from_secs(15) {
                    panic!("timed out waiting for GET {url}");
                }
                thread::sleep(Duration::from_millis(20));
            }
        }
    }
}

fn is_transient_http_send(err: &reqwest::Error) -> bool {
    if err.is_connect() || err.is_timeout() || err.is_request() {
        return true;
    }
    let msg = err.to_string();
    msg.contains("Connection reset")
        || msg.contains("connection reset")
        || msg.contains("Broken pipe")
        || msg.contains("connection closed")
        || msg.contains("os error 54")
        || msg.contains("os error 104")
}

/// Retry POST on connection-reset during graceful shutdown (FTQ-001 / FTQ-002).
fn post_json_tolerate_transient(
    client: &reqwest::blocking::Client,
    url: &str,
    body: &serde_json::Value,
) -> reqwest::blocking::Response {
    let start = std::time::Instant::now();
    loop {
        match client.post(url).json(body).send() {
            Ok(resp) => return resp,
            Err(err) if is_transient_http_send(&err) => {
                if start.elapsed() > Duration::from_secs(8) {
                    panic!("POST {url} failed after transient retries: {err}");
                }
                thread::sleep(Duration::from_millis(25));
            }
            Err(err) => panic!("POST {url}: {err}"),
        }
    }
}

/// Bind-only start for the session-timeout test (FTQ-003).
///
/// Skips `wait_for_get` so the 15s readiness poll cannot consume
/// [`HostOptions::session_timeout`].
fn start_report_skip_get(
    mode: ReportMode,
    panels: Option<Vec<ReportPanelEntry>>,
    timeout: Duration,
) -> (DialogHandle, PathBuf) {
    let ui_root = write_report_ui_root();
    let url_file = unique_path("wyvern-report-review-timeout-url");
    let handle = begin(
        report_command(mode, panels),
        host_options(ui_root, url_file.clone(), timeout),
    )
    .expect("begin");
    let _ = wait_for_url_file(&url_file);
    (handle, url_file)
}

fn matching_finish_body(approved: bool, comments: &str) -> serde_json::Value {
    serde_json::json!({
        "approved": approved,
        "comments": comments,
        "panels": [
            { "path": "panels/fail.xhtml", "label": "Fail 1", "role": "failure" },
            { "path": "panels/proposed-fix.xhtml", "label": "Proposed fix", "role": "proposal" }
        ]
    })
}

#[test]
fn report_review_finish_approve_emits_stdout_data() {
    let (handle, base, url_file, client) = start_report(
        ReportMode::Review,
        Some(review_panels()),
        Duration::from_secs(30),
    );
    let ack = client
        .post(format!("{base}/api/report/finish"))
        .json(&matching_finish_body(true, "looks good"))
        .send()
        .expect("POST finish");
    assert_eq!(ack.status(), reqwest::StatusCode::OK);
    let result = handle.await_result().expect("await");
    let value: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&result).expect("ser")).expect("json");
    assert_eq!(value["button"], "finish");
    assert_eq!(value["data"]["approved"], true);
    assert_eq!(value["data"]["comments"], "looks good");
    assert_eq!(value["data"]["panels"][0]["path"], "panels/fail.xhtml");
    assert_eq!(value["data"]["panels"][1]["role"], "proposal");
    let _ = std::fs::remove_file(url_file);
}

#[test]
fn report_review_finish_cancel_sets_approved_false() {
    let (handle, base, url_file, client) = start_report(
        ReportMode::Review,
        Some(review_panels()),
        Duration::from_secs(30),
    );
    let ack = client
        .post(format!("{base}/api/report/finish"))
        .json(&matching_finish_body(false, ""))
        .send()
        .expect("POST finish");
    assert_eq!(ack.status(), reqwest::StatusCode::OK);
    let result = handle.await_result().expect("await");
    let value: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&result).expect("ser")).expect("json");
    assert_eq!(value["button"], "finish");
    assert_eq!(value["data"]["approved"], false);
    assert_eq!(value["data"]["comments"], "");
    let _ = std::fs::remove_file(url_file);
}

#[test]
fn report_view_finish_route_is_unregistered() {
    let (handle, base, url_file, client) =
        start_report(ReportMode::View, None, Duration::from_secs(30));
    let response = client
        .post(format!("{base}/api/report/finish"))
        .json(&matching_finish_body(true, ""))
        .send()
        .expect("POST finish");
    assert_eq!(
        response.status(),
        reqwest::StatusCode::NOT_FOUND,
        "view mode must not register finish"
    );
    let _ = handle.viewer_exited_without_result();
    let _ = std::fs::remove_file(url_file);
}

#[test]
fn report_review_unknown_field_is_400() {
    let (handle, base, url_file, client) = start_report(
        ReportMode::Review,
        Some(review_panels()),
        Duration::from_secs(30),
    );
    let mut body = matching_finish_body(true, "x");
    body["extra"] = serde_json::json!(true);
    let response = client
        .post(format!("{base}/api/report/finish"))
        .json(&body)
        .send()
        .expect("POST finish");
    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
    let err: serde_json::Value = response.json().expect("err");
    assert_eq!(err["code"], "REPORT_FINISH_UNKNOWN_FIELD");
    let _ = handle.viewer_exited_without_result();
    let _ = std::fs::remove_file(url_file);
}

#[test]
fn report_review_panels_mismatch_is_400() {
    let (handle, base, url_file, client) = start_report(
        ReportMode::Review,
        Some(review_panels()),
        Duration::from_secs(30),
    );
    let body = serde_json::json!({
        "approved": true,
        "comments": "",
        "panels": [{ "path": "panels/other.xhtml", "label": "Nope", "role": "info" }]
    });
    let response = client
        .post(format!("{base}/api/report/finish"))
        .json(&body)
        .send()
        .expect("POST finish");
    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
    let err: serde_json::Value = response.json().expect("err");
    assert_eq!(err["code"], "REPORT_FINISH_PANELS_MISMATCH");
    let _ = handle.viewer_exited_without_result();
    let _ = std::fs::remove_file(url_file);
}

#[test]
fn report_review_invalid_json_is_400() {
    let (handle, base, url_file, client) = start_report(
        ReportMode::Review,
        Some(review_panels()),
        Duration::from_secs(30),
    );
    let response = client
        .post(format!("{base}/api/report/finish"))
        .header("content-type", "application/json")
        .body("not-json")
        .send()
        .expect("POST finish");
    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
    let err: serde_json::Value = response.json().expect("err");
    assert_eq!(err["code"], "REPORT_FINISH_INVALID_JSON");
    let _ = handle.viewer_exited_without_result();
    let _ = std::fs::remove_file(url_file);
}

#[test]
fn report_review_duplicate_finish_is_409() {
    let (handle, base, url_file, client) = start_report(
        ReportMode::Review,
        Some(review_panels()),
        Duration::from_secs(30),
    );
    let finish_url = format!("{base}/api/report/finish");
    // Concurrent POSTs so both hit the live server before graceful shutdown.
    let ready = Arc::new(Barrier::new(2));
    let client_a = client.clone();
    let client_b = client.clone();
    let url_a = finish_url.clone();
    let url_b = finish_url;
    let body_a = matching_finish_body(true, "");
    let body_b = matching_finish_body(false, "again");
    let ready_a = Arc::clone(&ready);
    let ready_b = Arc::clone(&ready);
    let first = thread::spawn(move || {
        ready_a.wait();
        post_json_tolerate_transient(&client_a, &url_a, &body_a)
    });
    let second = thread::spawn(move || {
        ready_b.wait();
        post_json_tolerate_transient(&client_b, &url_b, &body_b)
    });
    let a = first.join().expect("join a");
    let b = second.join().expect("join b");
    let statuses = [a.status(), b.status()];
    assert!(
        statuses.contains(&reqwest::StatusCode::OK)
            && statuses.contains(&reqwest::StatusCode::CONFLICT),
        "expected one 200 and one 409, got {statuses:?}"
    );
    let conflict = if a.status() == reqwest::StatusCode::CONFLICT {
        a
    } else {
        b
    };
    let err: serde_json::Value = conflict.json().expect("err");
    assert_eq!(err["code"], "REPORT_FINISH_ALREADY_COMPLETE");
    let _ = handle.await_result();
    let _ = std::fs::remove_file(url_file);
}

#[test]
fn report_review_finish_and_result_are_mutually_exclusive() {
    let (handle, base, url_file, client) = start_report(
        ReportMode::Review,
        Some(review_panels()),
        Duration::from_secs(30),
    );
    let finish_url = format!("{base}/api/report/finish");
    let result_url = format!("{base}/api/result");
    let ready = Arc::new(Barrier::new(2));
    let client_finish = client.clone();
    let client_result = client;
    let finish_body = matching_finish_body(true, "");
    let dismiss_body = serde_json::json!({ "button": "dismissed" });
    let ready_finish = Arc::clone(&ready);
    let ready_result = Arc::clone(&ready);
    let finish = thread::spawn(move || {
        ready_finish.wait();
        post_json_tolerate_transient(&client_finish, &finish_url, &finish_body)
    });
    let dismiss = thread::spawn(move || {
        ready_result.wait();
        post_json_tolerate_transient(&client_result, &result_url, &dismiss_body)
    });
    let finish = finish.join().expect("join finish");
    let dismiss = dismiss.join().expect("join result");
    let ok = reqwest::StatusCode::OK;
    let conflict = reqwest::StatusCode::CONFLICT;
    assert!(
        (finish.status() == ok && dismiss.status() == conflict)
            || (dismiss.status() == ok && finish.status() == conflict),
        "expected exclusive terminal actions, finish={} result={}",
        finish.status(),
        dismiss.status()
    );
    if finish.status() == conflict {
        let err: serde_json::Value = finish.json().expect("err");
        assert_eq!(err["code"], "REPORT_FINISH_ALREADY_COMPLETE");
    }
    let _ = handle.await_result();
    let _ = std::fs::remove_file(url_file);
}

#[test]
fn report_review_http_result_emits_dismissed() {
    let (handle, base, url_file, client) = start_report(
        ReportMode::Review,
        Some(review_panels()),
        Duration::from_secs(30),
    );
    let ack = client
        .post(format!("{base}/api/result"))
        .json(&serde_json::json!({ "button": "dismissed" }))
        .send()
        .expect("POST result");
    assert_eq!(ack.status(), reqwest::StatusCode::OK);
    let result = handle.await_result().expect("await");
    let json = serde_json::to_string(&result).expect("ser");
    assert_eq!(json, r#"{"button":"dismissed"}"#);
    let _ = std::fs::remove_file(url_file);
}

#[test]
fn report_review_os_close_emits_dismissed() {
    let (handle, _base, url_file, _client) = start_report(
        ReportMode::Review,
        Some(review_panels()),
        Duration::from_secs(30),
    );
    let result = handle.viewer_exited_without_result().expect("dismiss");
    let json = serde_json::to_string(&result).expect("ser");
    assert_eq!(json, r#"{"button":"dismissed"}"#);
    let _ = std::fs::remove_file(url_file);
}

#[test]
fn report_review_timeout_emits_dismissed() {
    let (handle, url_file) = start_report_skip_get(
        ReportMode::Review,
        Some(review_panels()),
        Duration::from_secs(1),
    );
    let result = handle.await_result().expect("timeout dismiss");
    let json = serde_json::to_string(&result).expect("ser");
    assert_eq!(json, r#"{"button":"dismissed"}"#);
    let _ = std::fs::remove_file(url_file);
}

#[test]
fn report_review_shared_js_is_mounted() {
    let (handle, base, url_file, client) = start_report(
        ReportMode::Review,
        Some(review_panels()),
        Duration::from_secs(30),
    );
    let js = client
        .get(format!("{base}/shared/report-review.js"))
        .send()
        .expect("GET js");
    assert!(js.status().is_success());
    let text = js.text().expect("js");
    assert!(
        text.contains("/api/report/finish") && !text.contains("wizard-nav"),
        "review JS must post finish and omit wizard-nav: {text}"
    );
    let _ = handle.viewer_exited_without_result();
    let _ = std::fs::remove_file(url_file);
}
