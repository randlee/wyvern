//! L1 HTTP tests for message dialog — no wry/winit.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

use wyvern_host::{run, HostError, HostOptions, ViewerMode};
use wyvern_schema::{ButtonsPreset, ChromeTitle, Command, CommandResult, MessageResult};

fn workspace_ui_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../ui")
}

fn message_command() -> Command {
    Command::Message {
        title: ChromeTitle::new("T"),
        message: "Hi".into(),
        status: None,
        buttons: ButtonsPreset::Ok,
        custom_buttons: None,
        default_button: None,
        level: None,
        icon: None,
        image: None,
        markdown: false,
    }
}

fn unique_path(prefix: &str) -> PathBuf {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("{prefix}-{}-{n}", std::process::id()))
}

fn host_options(url_file: PathBuf) -> HostOptions {
    HostOptions {
        bind: SocketAddr::from(([127, 0, 0, 1], 0)),
        ui_root: workspace_ui_root(),
        viewer: ViewerMode::None,
        dialog_url_env: true,
        dialog_url_file: Some(url_file),
        allow_non_loopback: false,
        session_timeout: wyvern_host::DEFAULT_SESSION_TIMEOUT,
        mock_picker: None,
    }
}

#[test]
fn run_message_posts_ok_via_http() {
    let url_file = unique_path("wyvern-host-url");
    let options = host_options(url_file.clone());
    let handle = thread::spawn(move || run(message_command(), options));

    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_url
        .trim_end_matches("/message/")
        .trim_end_matches('/');

    let client = reqwest::blocking::Client::new();
    let dialog: serde_json::Value = client
        .get(format!("{base}/api/dialog"))
        .send()
        .expect("GET dialog")
        .error_for_status()
        .expect("dialog status")
        .json()
        .expect("dialog json");
    assert_eq!(dialog["type"], "message");
    assert_eq!(dialog["title"], "T");
    assert_eq!(dialog["message"], "Hi");
    assert_eq!(dialog["buttons"], "ok");

    let page = client
        .get(&dialog_url)
        .send()
        .expect("GET page")
        .error_for_status()
        .expect("page status");
    let html = page.text().expect("html");
    assert!(
        html.contains("id=\"buttons\"") && html.contains("wyvern-api.js"),
        "expected message shell HTML: {html}"
    );

    let ack: serde_json::Value = client
        .post(format!("{base}/api/result"))
        .json(&serde_json::json!({"button":"ok"}))
        .send()
        .expect("POST result")
        .error_for_status()
        .expect("result status")
        .json()
        .expect("ack json");
    assert_eq!(ack["ok"], true);

    let result = handle.join().expect("host thread").expect("run ok");
    assert_eq!(
        result,
        CommandResult::Message(MessageResult {
            button: wyvern_schema::ButtonLabel::new("ok"),
        })
    );

    let _ = std::fs::remove_file(&url_file);
}

#[test]
fn begin_viewer_exit_yields_dismissed_stdout_shape() {
    // ATM-QA-004 / AC7: simulate embedded viewer OS-close via dismiss signal.
    use wyvern_host::begin;
    use wyvern_schema::ButtonLabel;

    let options = HostOptions {
        bind: SocketAddr::from(([127, 0, 0, 1], 0)),
        ui_root: workspace_ui_root(),
        viewer: ViewerMode::Embedded,
        dialog_url_env: false,
        dialog_url_file: None,
        allow_non_loopback: false,
        session_timeout: Duration::from_secs(30),
        mock_picker: None,
    };
    let handle = begin(message_command(), options).expect("begin");
    assert!(
        handle.dialog_url.contains("127.0.0.1"),
        "url={}",
        handle.dialog_url
    );
    let result = handle
        .viewer_exited_without_result()
        .expect("dismissed result");
    assert_eq!(
        result,
        CommandResult::Message(MessageResult {
            button: ButtonLabel::dismissed(),
        })
    );
}

#[test]
fn run_rejects_missing_ui_root() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let mut options = host_options(unique_path("unused"));
    options.ui_root = tmp.path().join("definitely-missing-wyvern-ui-root");
    let err = run(message_command(), options).expect_err("missing ui");
    assert!(matches!(err, HostError::UiNotFound { .. }));
}

#[test]
fn run_serves_custom_ui_root() {
    let dir = unique_path("wyvern-custom-ui");
    let message_dir = dir.join("message");
    std::fs::create_dir_all(&message_dir).expect("mkdir");
    std::fs::write(
        message_dir.join("index.html"),
        "<!doctype html><title>custom</title><body>custom-ui-marker</body>",
    )
    .expect("write");

    let url_file = dir.join("url.txt");
    let options = HostOptions {
        bind: SocketAddr::from(([127, 0, 0, 1], 0)),
        ui_root: dir.clone(),
        viewer: ViewerMode::None,
        dialog_url_env: true,
        dialog_url_file: Some(url_file.clone()),
        allow_non_loopback: false,
        session_timeout: wyvern_host::DEFAULT_SESSION_TIMEOUT,
        mock_picker: None,
    };
    let handle = thread::spawn(move || run(message_command(), options));
    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_url
        .trim_end_matches("/message/")
        .trim_end_matches('/');

    let client = reqwest::blocking::Client::new();
    let html = client
        .get(&dialog_url)
        .send()
        .expect("GET")
        .error_for_status()
        .expect("status")
        .text()
        .expect("text");
    assert!(
        html.contains("custom-ui-marker"),
        "unexpected html from {dialog_url}: {html}"
    );

    let _ = client
        .post(format!("{base}/api/result"))
        .json(&serde_json::json!({"button":"ok"}))
        .send();
    let _ = handle.join();

    let _ = std::fs::remove_dir_all(&dir);
}

fn wait_for_url_file(path: &std::path::Path) -> String {
    for _ in 0..200 {
        if let Ok(url) = std::fs::read_to_string(path) {
            let url = url.trim().to_string();
            if url.starts_with("http://") {
                return url;
            }
        }
        thread::sleep(Duration::from_millis(25));
    }
    panic!("timed out waiting for dialog URL file {}", path.display());
}
