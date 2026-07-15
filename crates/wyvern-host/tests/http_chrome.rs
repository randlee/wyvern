//! L1 HTTP tests for chrome dialog — no wry/winit.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

use wyvern_host::{run, HostOptions, ViewerMode};
use wyvern_schema::{ChromeResult, ChromeStatus, ChromeTitle, Command, CommandResult};

fn workspace_ui_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../ui")
}

fn chrome_command() -> Command {
    Command::Chrome {
        title: ChromeTitle::new("Test Chrome"),
        status: None,

        width: None,
        height: None,
    }
}

fn chrome_command_with_status(status: impl Into<String>) -> Command {
    Command::Chrome {
        title: ChromeTitle::new("Test Chrome"),
        status: Some(ChromeStatus::new(status)),

        width: None,
        height: None,
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
        shared_ui_root: workspace_ui_root(),
        viewer: ViewerMode::None,
        dialog_url_env: true,
        dialog_url_file: Some(url_file),
        allow_non_loopback: false,
        session_timeout: wyvern_host::DEFAULT_SESSION_TIMEOUT,
        mock_picker: None,
    }
}

/// Poll `GET /api/dialog` until HTTP 200 (URL file alone is not readiness).
fn wait_for_dialog_ready(client: &reqwest::blocking::Client, base: &str) -> serde_json::Value {
    let url = format!("{base}/api/dialog");
    let start = std::time::Instant::now();
    loop {
        match client.get(&url).send() {
            Ok(resp) if resp.status() == reqwest::StatusCode::OK => {
                return resp.json().expect("dialog json");
            }
            Ok(_) | Err(_) => {
                if start.elapsed() > Duration::from_secs(15) {
                    panic!("timed out waiting for GET /api/dialog at {url}");
                }
                thread::sleep(Duration::from_millis(20));
            }
        }
    }
}

fn dialog_base(dialog_url: &str) -> String {
    dialog_url
        .trim_end_matches("/chrome/")
        .trim_end_matches('/')
        .to_string()
}

#[test]
fn run_chrome_posts_ok_via_http() {
    let url_file = unique_path("wyvern-chrome-url");
    let options = host_options(url_file.clone());
    let handle = thread::spawn(move || run(chrome_command(), options));

    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_base(&dialog_url);

    let client = reqwest::blocking::Client::new();
    let dialog = wait_for_dialog_ready(&client, &base);
    assert_eq!(dialog["type"], "chrome");
    assert_eq!(dialog["title"], "Test Chrome");
    assert!(dialog.get("status").is_none() || dialog["status"].is_null());

    let page = client
        .get(&dialog_url)
        .send()
        .expect("GET page")
        .error_for_status()
        .expect("page status");
    let html = page.text().expect("html");
    assert!(
        html.contains("id=\"dialog\"") && html.contains("wyvern-api.js"),
        "expected chrome shell HTML: {html}"
    );

    let ack: serde_json::Value = client
        .post(format!("{base}/api/result"))
        .json(&serde_json::json!({"button": "ok"}))
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
        CommandResult::Chrome(ChromeResult {
            button: wyvern_schema::ButtonLabel::new("ok"),
        })
    );

    let _ = std::fs::remove_file(&url_file);
}

#[test]
fn run_chrome_dismissed_via_beacon() {
    let url_file = unique_path("wyvern-chrome-dismissed-url");
    let options = host_options(url_file.clone());
    let handle = thread::spawn(move || run(chrome_command(), options));

    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_base(&dialog_url);

    let client = reqwest::blocking::Client::new();
    let _ = wait_for_dialog_ready(&client, &base);

    let _ = client
        .post(format!("{base}/api/result"))
        .json(&serde_json::json!({"button": "dismissed"}))
        .send()
        .expect("POST dismissed");

    let result = handle.join().expect("host thread").expect("run ok");
    assert_eq!(
        result,
        CommandResult::Chrome(ChromeResult {
            button: wyvern_schema::ButtonLabel::new("dismissed"),
        })
    );

    let _ = std::fs::remove_file(&url_file);
}

#[test]
fn run_chrome_dialog_payload_includes_status() {
    let url_file = unique_path("wyvern-chrome-status-url");
    let options = host_options(url_file.clone());
    let handle = thread::spawn(move || run(chrome_command_with_status("Ready"), options));

    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_base(&dialog_url);

    let client = reqwest::blocking::Client::new();
    let dialog = wait_for_dialog_ready(&client, &base);
    assert_eq!(dialog["type"], "chrome");
    assert_eq!(dialog["title"], "Test Chrome");
    assert_eq!(dialog["status"], "Ready");

    let _ = client
        .post(format!("{base}/api/result"))
        .json(&serde_json::json!({"button": "ok"}))
        .send();
    let _ = handle.join();

    let _ = std::fs::remove_file(&url_file);
}

#[test]
fn run_chrome_result_rejects_missing_button() {
    let url_file = unique_path("wyvern-chrome-bad-result-url");
    let options = host_options(url_file.clone());
    let handle = thread::spawn(move || run(chrome_command(), options));

    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_base(&dialog_url);

    let client = reqwest::blocking::Client::new();
    let _ = wait_for_dialog_ready(&client, &base);

    let status = client
        .post(format!("{base}/api/result"))
        .json(&serde_json::json!({"not_button": "ok"}))
        .send()
        .expect("POST result")
        .status();
    assert_eq!(status, 400, "expected 400 for missing button field");

    // Clean up by posting a valid result so the thread unblocks.
    let _ = client
        .post(format!("{base}/api/result"))
        .json(&serde_json::json!({"button": "dismissed"}))
        .send();
    let _ = handle.join();

    let _ = std::fs::remove_file(&url_file);
}

fn wait_for_url_file(path: &std::path::Path) -> String {
    let start = std::time::Instant::now();
    loop {
        if let Ok(url) = std::fs::read_to_string(path) {
            let url = url.trim().to_string();
            if url.starts_with("http://") {
                return url;
            }
        }
        if start.elapsed() > Duration::from_secs(15) {
            panic!("timed out waiting for dialog URL file {}", path.display());
        }
        thread::sleep(Duration::from_millis(25));
    }
}
