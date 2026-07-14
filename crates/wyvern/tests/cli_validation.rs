//! CLI integration: load → validate → run → emit.

use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

fn wyvern() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_wyvern"));
    // Isolate from developer/CI WYVERN_LOG so stdout/stderr assertions stay stable.
    cmd.env_remove("WYVERN_LOG");
    cmd.env("WYVERN_UI_ROOT", workspace_ui_root());
    cmd
}

fn workspace_ui_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../ui")
}

/// Spawns `wyvern`; detects child panic/abort/signal failures.
fn run_wyvern(cmd: &mut Command) -> std::process::Output {
    cmd.env_remove("WYVERN_LOG");
    let output = cmd.output().expect("spawn wyvern");
    assert_child_ok(&output);
    output
}

fn run_json(json: &str) -> (i32, String, String) {
    let output = run_wyvern(wyvern().arg(json));
    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    (code, stdout, stderr)
}

mod child_assert {
    /// Pure predicate — unit-testable without synthesizing `ExitStatus`.
    /// `code == -1` covers Unix signal exits where `status.code()` is `None`.
    pub(super) fn child_failed(stderr: &str, code: i32) -> bool {
        stderr.contains("panicked at")
            || stderr.contains("misaligned pointer")
            || stderr.contains("cannot unwind")
            || stderr.contains("abort")
            || code == -1
    }

    pub(super) fn assert_child_ok(output: &std::process::Output) {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let code = output.status.code().unwrap_or(-1);
        assert!(
            !child_failed(&stderr, code),
            "wyvern child panicked/aborted (use --test-threads=1 on macOS):\n\
             code={code:?}\nstderr={stderr}"
        );
    }
}

use child_assert::{assert_child_ok, child_failed};

fn stderr_json(stderr: &str) -> serde_json::Value {
    // Host may print WYVERN_DIALOG_URL=… before JSON error lines.
    let json_line = stderr
        .lines()
        .rev()
        .find(|line| line.trim_start().starts_with('{'))
        .unwrap_or(stderr.trim());
    serde_json::from_str(json_line.trim()).unwrap_or_else(|err| {
        panic!("stderr is not JSON ({err}): {stderr:?}");
    })
}

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("bind")
        .local_addr()
        .expect("addr")
        .port()
}

#[test]
fn child_failed_detects_panic_marker() {
    assert!(child_failed("thread 'main' panicked at winit\n", 0));
    assert!(!child_failed("", 0));
}

#[test]
fn child_failed_detects_signal_exit() {
    assert!(child_failed("", -1)); // None mapped via unwrap_or(-1)
}

#[test]
fn cli_chrome_missing_title_validation_error() {
    let (code, _stdout, stderr) = run_json(r#"{"type":"chrome"}"#);
    assert_ne!(code, 0);
    let value = stderr_json(&stderr);
    assert_eq!(value["error"], "validation");
    assert_eq!(value["field"], "title");
}

#[test]
fn cli_empty_object_missing_type() {
    let (code, _stdout, stderr) = run_json("{}");
    assert_ne!(code, 0);
    let value = stderr_json(&stderr);
    assert_eq!(value["error"], "validation");
    assert_eq!(value["field"], "type");
}

#[test]
fn cli_type_null_validation_error() {
    let (code, _stdout, stderr) = run_json(r#"{"type":null,"title":"T"}"#);
    assert_ne!(code, 0);
    let value = stderr_json(&stderr);
    assert_eq!(value["error"], "validation");
    assert_eq!(value["field"], "type");
    assert!(value["message"].as_str().unwrap().contains("null"));
}

#[test]
fn cli_unknown_field_validation_error() {
    let (code, _stdout, stderr) = run_json(r#"{"type":"chrome","title":"T","extra":1}"#);
    assert_ne!(code, 0);
    let value = stderr_json(&stderr);
    assert_eq!(value["error"], "validation");
    assert_eq!(value["field"], "extra");
}

#[test]
fn cli_type_message_level_invalid_validation_error() {
    let (code, stdout, stderr) = run_json(
        r#"{"type":"message","title":"T","message":"Hi","buttons":"ok","level":"critical"}"#,
    );
    assert_ne!(code, 0);
    assert!(stdout.trim().is_empty(), "stdout={stdout}");
    let value = stderr_json(&stderr);
    assert_eq!(value["error"], "validation");
    assert_eq!(value["field"], "level");
    assert!(value["message"]
        .as_str()
        .unwrap()
        .contains("expected one of"));
}

#[test]
fn cli_type_message_missing_buttons_validation_error() {
    let (code, _stdout, stderr) = run_json(r#"{"type":"message","title":"T","message":"Hi"}"#);
    assert_ne!(code, 0);
    let value = stderr_json(&stderr);
    assert_eq!(value["error"], "validation");
    assert_eq!(value["field"], "buttons");
}

#[test]
fn cli_input_multiline_with_file_validation_error() {
    let (code, stdout, stderr) =
        run_json(r#"{"type":"input","title":"T","message":"M","mode":"file","multiline":true}"#);
    assert_ne!(code, 0);
    assert!(stdout.trim().is_empty(), "stdout={stdout}");
    let value = stderr_json(&stderr);
    assert_eq!(value["error"], "validation");
    assert_eq!(value["field"], "multiline");
}

#[test]
fn cli_input_filter_with_text_validation_error() {
    let (code, stdout, stderr) =
        run_json(r#"{"type":"input","title":"T","message":"M","filter":["*.rs"]}"#);
    assert_ne!(code, 0);
    assert!(stdout.trim().is_empty(), "stdout={stdout}");
    let value = stderr_json(&stderr);
    assert_eq!(value["error"], "validation");
    assert_eq!(value["field"], "filter");
}

#[test]
fn cli_type_unknown_validation_error() {
    let (code, _stdout, stderr) = run_json(r#"{"type":"unknown"}"#);
    assert_ne!(code, 0);
    let value = stderr_json(&stderr);
    assert_eq!(value["error"], "validation");
    assert_eq!(value["field"], "type");
}

#[test]
fn cli_markdown_missing_file_is_io() {
    let (code, stdout, stderr) =
        run_json(r#"{"type":"markdown","file":"/definitely/missing/wyvern-b5.md"}"#);
    assert_ne!(code, 0);
    assert!(stdout.trim().is_empty(), "stdout={stdout}");
    let value = stderr_json(&stderr);
    assert_eq!(value["error"], "io");
    assert_eq!(value["field"], "file");
}

#[test]
fn cli_markdown_both_file_and_content_validation_error() {
    let (code, stdout, stderr) =
        run_json(r##"{"type":"markdown","file":"doc.md","content":"# Hi"}"##);
    assert_ne!(code, 0);
    assert!(stdout.trim().is_empty(), "stdout={stdout}");
    let value = stderr_json(&stderr);
    assert_eq!(value["error"], "validation");
    assert_eq!(value["field"], "file");
    assert!(value["message"]
        .as_str()
        .unwrap()
        .contains("exactly one of"));
}

#[test]
fn cli_action_show_state_error() {
    let (code, _stdout, stderr) = run_json(r#"{"action":"show"}"#);
    assert_ne!(code, 0);
    let value = stderr_json(&stderr);
    assert_eq!(value["error"], "state");
    assert_eq!(value["field"], "action");
}

#[test]
fn cli_wrong_title_type_expected_got() {
    let (code, _stdout, stderr) = run_json(r#"{"type":"chrome","title":123}"#);
    assert_ne!(code, 0);
    let value = stderr_json(&stderr);
    assert_eq!(value["error"], "validation");
    assert_eq!(value["field"], "title");
    let message = value["message"].as_str().unwrap();
    assert!(message.contains("expected string"));
    assert!(message.contains("number"));
}

/// README Phase B acceptance #5 — wizard still Phase D validation error.
#[test]
fn cli_wizard_still_validation_error() {
    let (code, stdout, stderr) = run_json(r#"{"type":"wizard","title":"T"}"#);
    assert_ne!(code, 0);
    assert!(stdout.trim().is_empty(), "stdout={stdout}");
    let value = stderr_json(&stderr);
    assert_eq!(value["error"], "validation");
    assert_eq!(value["field"], "type");
}

#[test]
fn cli_chrome_unsupported_type_at_runtime() {
    let (code, stdout, stderr) = run_json(r#"{"type":"chrome","title":"T"}"#);
    assert_ne!(code, 0);
    assert!(stdout.trim().is_empty(), "stdout={stdout}");
    let value = stderr_json(&stderr);
    assert_eq!(value["error"], "host_error");
    assert_eq!(value["code"], "UNSUPPORTED_TYPE");
}

#[test]
fn cli_message_viewer_none_posts_ok() {
    let port = free_port();
    let bind = format!("127.0.0.1:{port}");
    let json = r#"{"type":"message","title":"T","message":"Hi","buttons":"ok"}"#;
    let child = wyvern()
        .args([json, "--viewer", "none", "--bind", &bind])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn");

    let client = wait_for_http(&format!("http://{bind}/api/dialog"));
    let dialog: serde_json::Value = client
        .get(format!("http://{bind}/api/dialog"))
        .send()
        .expect("dialog")
        .json()
        .expect("json");
    assert_eq!(dialog["type"], "message");

    let ack: serde_json::Value = client
        .post(format!("http://{bind}/api/result"))
        .json(&serde_json::json!({"button":"ok"}))
        .send()
        .expect("post")
        .json()
        .expect("ack");
    assert_eq!(ack["ok"], true);

    let output = child.wait_with_output().expect("wait");
    assert_child_ok(&output);
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), r#"{"button":"ok"}"#);
}

#[test]
fn cli_message_omitted_viewer_defaults_to_none() {
    let port = free_port();
    let bind = format!("127.0.0.1:{port}");
    let json = r#"{"type":"message","title":"T","message":"Hi","buttons":"ok"}"#;
    let child = wyvern()
        .args([json, "--bind", &bind])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn");

    let client = wait_for_http(&format!("http://{bind}/api/dialog"));
    let _ = client
        .post(format!("http://{bind}/api/result"))
        .json(&serde_json::json!({"button":"ok"}))
        .send()
        .expect("post");

    let output = child.wait_with_output().expect("wait");
    assert_child_ok(&output);
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), r#"{"button":"ok"}"#);
}

fn wait_for_http(url: &str) -> reqwest::blocking::Client {
    let client = reqwest::blocking::Client::new();
    for _ in 0..200 {
        if client
            .get(url)
            .send()
            .map(|r| r.status().is_success())
            .unwrap_or(false)
        {
            return client;
        }
        thread::sleep(Duration::from_millis(25));
    }
    panic!("timed out waiting for {url}");
}
