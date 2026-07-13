//! CLI integration: load → validate → run → emit.

use std::process::Command;

fn wyvern() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_wyvern"));
    // Auto-dismiss so chrome GUI paths do not block the test harness.
    cmd.env("WYVERN_AUTO_DISMISS", "1");
    // Isolate from developer/CI WYVERN_LOG so stdout/stderr assertions stay stable.
    cmd.env_remove("WYVERN_LOG");
    cmd
}

fn run_json(json: &str) -> (i32, String, String) {
    let output = wyvern().arg(json).output().expect("spawn wyvern");
    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    (code, stdout, stderr)
}

fn stderr_json(stderr: &str) -> serde_json::Value {
    serde_json::from_str(stderr.trim()).unwrap_or_else(|err| {
        panic!("stderr is not JSON ({err}): {stderr:?}");
    })
}

#[test]
fn cli_valid_chrome_emits_dismissed() {
    let (code, stdout, stderr) = run_json(r#"{"type":"chrome","title":"T"}"#);
    assert_eq!(code, 0, "stderr={stderr}");
    assert_eq!(stdout.trim(), r#"{"button":"dismissed"}"#);
    assert!(stderr.trim().is_empty(), "stderr={stderr}");
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
fn cli_type_message_not_implemented() {
    let (code, _stdout, stderr) = run_json(r#"{"type":"message","message":"Hi"}"#);
    assert_ne!(code, 0);
    let value = stderr_json(&stderr);
    assert_eq!(value["error"], "validation");
    assert_eq!(value["field"], "type");
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
