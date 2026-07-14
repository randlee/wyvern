//! L1 HTTP tests for markdown dialog — content_html + result shape.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use wyvern_host::{run, HostOptions, ViewerMode};
use wyvern_schema::{ButtonsPreset, ChromeTitle, Command, CommandResult, MarkdownResult};

fn workspace_ui_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../ui")
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

fn markdown_command(content: &str) -> Command {
    Command::Markdown {
        title: Some(ChromeTitle::new("Doc")),
        file: None,
        content: Some(content.into()),
        status: Some(wyvern_schema::ChromeStatus::new("Read-only")),
        buttons: ButtonsPreset::Ok,
    
            width: None,
            height: None,}
}

fn wait_for_url_file(path: &Path) -> String {
    let start = std::time::Instant::now();
    loop {
        if let Ok(url) = std::fs::read_to_string(path) {
            let url = url.trim().to_string();
            if url.starts_with("http://") {
                return url;
            }
        }
        if start.elapsed() > Duration::from_secs(15) {
            panic!("timed out waiting for dialog URL file: {}", path.display());
        }
        thread::sleep(Duration::from_millis(20));
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

#[test]
fn run_markdown_posts_ok_via_http() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let url_file = tmp.path().join("dialog-url");
    let options = host_options(url_file.clone());
    let handle = thread::spawn(move || run(markdown_command("# Hello\n\nBody"), options));

    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_url
        .trim_end_matches("/markdown/")
        .trim_end_matches('/');

    let client = reqwest::blocking::Client::new();
    let dialog = wait_for_dialog_ready(&client, base);
    assert_eq!(dialog["type"], "markdown");
    assert_eq!(dialog["title"], "Doc");
    assert_eq!(dialog["content"], "# Hello\n\nBody");
    assert_eq!(dialog["buttons"], "ok");
    assert_eq!(dialog["status"], "Read-only");
    let content_html = dialog["content_html"].as_str().expect("content_html");
    assert!(
        content_html.contains("<h1>Hello</h1>"),
        "content_html={content_html}"
    );
    assert!(
        content_html.contains("<p>Body</p>"),
        "content_html={content_html}"
    );

    let page = client
        .get(&dialog_url)
        .send()
        .expect("GET page")
        .error_for_status()
        .expect("page status");
    let html = page.text().expect("html");
    assert!(
        html.contains("markdown-body") && html.contains("wyvern-api.js"),
        "expected markdown shell HTML: {html}"
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
        CommandResult::Markdown(MarkdownResult {
            button: wyvern_schema::ButtonLabel::new("ok"),
        })
    );
}

#[test]
fn dialog_content_html_strips_script_tags() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let url_file = tmp.path().join("dialog-url");
    let options = host_options(url_file.clone());
    let source = "Hi <script>alert(1)</script>\n\n<img src=x onerror=alert(2)>";
    let handle = thread::spawn(move || run(markdown_command(source), options));

    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_url
        .trim_end_matches("/markdown/")
        .trim_end_matches('/');

    let client = reqwest::blocking::Client::new();
    let dialog = wait_for_dialog_ready(&client, base);

    let content_html = dialog["content_html"].as_str().expect("content_html");
    let lower = content_html.to_ascii_lowercase();
    assert!(
        !lower.contains("<script") && !lower.contains("alert(1)"),
        "content_html={content_html}"
    );
    assert!(!lower.contains("onerror"), "content_html={content_html}");
    assert!(content_html.contains("Hi"), "content_html={content_html}");

    let _ = client
        .post(format!("{base}/api/result"))
        .json(&serde_json::json!({"button":"ok"}))
        .send()
        .expect("POST result")
        .error_for_status();
    let _ = handle.join().expect("host thread").expect("run ok");
}

#[test]
fn dialog_rejects_oversized_markdown_content() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let url_file = tmp.path().join("dialog-url");
    let options = host_options(url_file.clone());
    let oversized = "x".repeat(wyvern_schema::MARKDOWN_CONTENT_MAX_BYTES + 1);
    let handle = thread::spawn(move || run(markdown_command(&oversized), options));

    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_url
        .trim_end_matches("/markdown/")
        .trim_end_matches('/');

    let client = reqwest::blocking::Client::new();
    let start = std::time::Instant::now();
    let response = loop {
        match client.get(format!("{base}/api/dialog")).send() {
            Ok(resp) => break resp,
            Err(_) if start.elapsed() < Duration::from_secs(15) => {
                thread::sleep(Duration::from_millis(20));
            }
            Err(err) => panic!("GET dialog failed: {err}"),
        }
    };
    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
    let body: serde_json::Value = response.json().expect("error json");
    assert_eq!(body["error"], "bad_request");
    assert!(body["message"]
        .as_str()
        .is_some_and(|m| m.contains("exceeds maximum")));
    assert!(body["cause"].as_str().is_some_and(|s| !s.is_empty()));
    assert!(body["recovery"].as_array().is_some_and(|a| !a.is_empty()));
    assert!(body["docs"]
        .as_str()
        .is_some_and(|s| s.contains("c12-host-markdown")));

    // Fail-safe dismiss so the host thread exits.
    let _ = client
        .post(format!("{base}/api/result"))
        .json(&serde_json::json!({"button":"dismissed"}))
        .send();
    let _ = handle.join();
}

#[test]
fn result_invalid_markdown_includes_cause_recovery_docs() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let url_file = tmp.path().join("dialog-url");
    let options = host_options(url_file.clone());
    let handle = thread::spawn(move || run(markdown_command("# Hello\n"), options));

    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_url
        .trim_end_matches("/markdown/")
        .trim_end_matches('/');

    let client = reqwest::blocking::Client::new();
    let _ = wait_for_dialog_ready(&client, base);
    let response = client
        .post(format!("{base}/api/result"))
        .json(&serde_json::json!({}))
        .send()
        .expect("POST result");
    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
    let body: serde_json::Value = response.json().expect("error json");
    assert_eq!(body["error"], "bad_request");
    assert!(body["message"]
        .as_str()
        .is_some_and(|m| m.contains("button")));
    assert!(body["cause"].as_str().is_some_and(|s| !s.is_empty()));
    assert!(body["recovery"].as_array().is_some_and(|a| !a.is_empty()));
    assert!(body["docs"]
        .as_str()
        .is_some_and(|s| s.contains("http-post-schema")));

    let _ = client
        .post(format!("{base}/api/result"))
        .json(&serde_json::json!({"button":"ok"}))
        .send()
        .expect("POST result")
        .error_for_status();
    let _ = handle.join().expect("host thread").expect("run ok");
}
