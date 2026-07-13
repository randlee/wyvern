//! CLI integration: load → validate → run → emit.

use std::process::Command;

use serial_test::serial;

fn wyvern() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_wyvern"));
    // Auto-dismiss so chrome GUI paths do not block the test harness.
    cmd.env("WYVERN_AUTO_DISMISS", "1");
    // Isolate from developer/CI WYVERN_LOG so stdout/stderr assertions stay stable.
    cmd.env_remove("WYVERN_LOG");
    cmd
}

/// Spawns `wyvern` with auto-dismiss; detects child panic/abort/signal failures.
fn run_wyvern(cmd: &mut Command) -> std::process::Output {
    cmd.env("WYVERN_AUTO_DISMISS", "1").env_remove("WYVERN_LOG");
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
    serde_json::from_str(stderr.trim()).unwrap_or_else(|err| {
        panic!("stderr is not JSON ({err}): {stderr:?}");
    })
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
#[serial]
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
#[serial]
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
#[serial]
fn cli_valid_message_emits_dismissed() {
    let (code, stdout, stderr) =
        run_json(r#"{"type":"message","title":"T","message":"Hi","buttons":"ok"}"#);
    assert_eq!(code, 0, "stderr={stderr}");
    assert_eq!(stdout.trim(), r#"{"button":"dismissed"}"#);
    assert!(stderr.trim().is_empty(), "stderr={stderr}");
}

#[test]
#[serial]
fn cli_valid_input_emits_dismissed() {
    let (code, stdout, stderr) = run_json(r#"{"type":"input","title":"Name","message":"Enter"}"#);
    assert_eq!(code, 0, "stderr={stderr}");
    assert_eq!(stdout.trim(), r#"{"button":"dismissed"}"#);
    assert!(stderr.trim().is_empty(), "stderr={stderr}");
}

#[test]
#[serial]
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
#[serial]
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
#[serial]
fn cli_markdown_md_shorthand_emits_dismissed() {
    let dir = std::env::temp_dir().join(format!("wyvern-b5-sh-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("notes.md");
    std::fs::write(&path, "# Notes\n\nBody\n").unwrap();

    let output = run_wyvern(wyvern().arg(path.to_str().unwrap()));
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
#[serial]
fn cli_markdown_content_inline_emits_dismissed() {
    let (code, stdout, stderr) = run_json(r##"{"type":"markdown","content":"# Hi"}"##);
    assert_eq!(code, 0, "stderr={stderr}");
    assert_eq!(stdout.trim(), r#"{"button":"dismissed"}"#);
    assert!(stderr.trim().is_empty(), "stderr={stderr}");
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

/// README Phase B acceptance #4 — question opens; OS close → REQ-0068 shape.
#[test]
#[serial]
fn cli_question_auto_dismiss_emits_req_0068() {
    let (code, stdout, stderr) = run_json(
        r#"{"type":"question","questions":[{"question":"Output format?","header":"Format","options":[{"label":"JSON","description":"Structured","preview":"<pre>{\"ok\":true}</pre>"},{"label":"Plain","description":"Text only"}],"multiSelect":false}]}"#,
    );
    assert_eq!(code, 0, "stderr={stderr}");
    assert!(stderr.trim().is_empty(), "stderr={stderr}");
    let value: serde_json::Value = serde_json::from_str(stdout.trim()).expect("stdout json");
    assert_eq!(value["button"], "dismissed");
    assert_eq!(value["answers"], serde_json::json!({}));
    assert_eq!(value["response"], "");
    assert_eq!(value["questions"][0]["question"], "Output format?");
    assert_eq!(
        value["questions"][0]["options"][0]["preview"],
        r#"<pre>{"ok":true}</pre>"#
    );
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
