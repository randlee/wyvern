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
fn cli_type_message_level_accepted() {
    let (code, stdout, stderr) =
        run_json(r#"{"type":"message","title":"T","message":"Hi","buttons":"ok","level":"info"}"#);
    assert_eq!(code, 0, "stderr={stderr}");
    assert_eq!(stdout.trim(), r#"{"button":"dismissed"}"#);
    assert!(stderr.trim().is_empty(), "stderr={stderr}");
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
fn cli_valid_message_emits_dismissed() {
    let (code, stdout, stderr) =
        run_json(r#"{"type":"message","title":"T","message":"Hi","buttons":"ok"}"#);
    assert_eq!(code, 0, "stderr={stderr}");
    assert_eq!(stdout.trim(), r#"{"button":"dismissed"}"#);
    assert!(stderr.trim().is_empty(), "stderr={stderr}");
}

#[test]
fn cli_valid_input_emits_dismissed() {
    let (code, stdout, stderr) = run_json(r#"{"type":"input","title":"Name","message":"Enter"}"#);
    assert_eq!(code, 0, "stderr={stderr}");
    assert_eq!(stdout.trim(), r#"{"button":"dismissed"}"#);
    assert!(stderr.trim().is_empty(), "stderr={stderr}");
}

#[test]
fn cli_valid_input_file_mode_emits_dismissed() {
    let (code, stdout, stderr) =
        run_json(r#"{"type":"input","title":"T","message":"M","mode":"file"}"#);
    assert_eq!(code, 0, "stderr={stderr}");
    assert_eq!(stdout.trim(), r#"{"button":"dismissed"}"#);
    assert!(stderr.trim().is_empty(), "stderr={stderr}");
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
fn cli_valid_markdown_file_emits_dismissed() {
    let dir = std::env::temp_dir().join(format!("wyvern-b5-cli-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("doc.md");
    std::fs::write(
        &path,
        "# Hello\n\n- list\n\n```\ncode\n```\n\n| A | B |\n|---|---|\n| 1 | 2 |\n",
    )
    .unwrap();

    let json = format!(
        r#"{{"type":"markdown","file":"{}"}}"#,
        path.to_str().unwrap().replace('\\', "\\\\")
    );
    let (code, stdout, stderr) = run_json(&json);
    assert_eq!(code, 0, "stderr={stderr}");
    assert_eq!(stdout.trim(), r#"{"button":"dismissed"}"#);
    assert!(stderr.trim().is_empty(), "stderr={stderr}");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn cli_markdown_md_shorthand_emits_dismissed() {
    let dir = std::env::temp_dir().join(format!("wyvern-b5-sh-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("notes.md");
    std::fs::write(&path, "# Notes\n\nBody\n").unwrap();

    let output = wyvern()
        .arg(path.to_str().unwrap())
        .output()
        .expect("spawn wyvern");
    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(code, 0, "stderr={stderr}");
    assert_eq!(stdout.trim(), r#"{"button":"dismissed"}"#);
    assert!(stderr.trim().is_empty(), "stderr={stderr}");

    let _ = std::fs::remove_dir_all(&dir);
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
fn cli_markdown_content_validation_error() {
    let (code, stdout, stderr) = run_json(r##"{"type":"markdown","content":"# Hi"}"##);
    assert_ne!(code, 0);
    assert!(stdout.trim().is_empty(), "stdout={stdout}");
    let value = stderr_json(&stderr);
    assert_eq!(value["error"], "validation");
    assert_eq!(value["field"], "content");
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
