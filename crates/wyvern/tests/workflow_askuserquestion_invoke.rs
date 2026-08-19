//! `--invoke`: PreToolUse stdin → question envelope → REQ-0067 answers.

use std::io::Write;
use std::path::PathBuf;

use serde_json::{json, Value};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn apply_script() -> PathBuf {
    workspace_root().join("scripts/ext/apply-askuserquestion-hook.py")
}

fn write_script(dir: &std::path::Path, name: &str, body: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, body).expect("write mock");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&path, perms).unwrap();
    }
    path
}

fn sample_pretool_use() -> Value {
    json!({
        "hook_event_name": "PreToolUse",
        "tool_name": "AskUserQuestion",
        "tool_input": {
            "questions": [
                {
                    "question": "Pick one",
                    "header": "Pick",
                    "options": [{ "label": "A" }, { "label": "B" }],
                    "multiSelect": false
                }
            ]
        }
    })
}

#[test]
fn invoke_maps_pretooluse_to_question_answers() {
    let tmp = tempfile::tempdir().unwrap();
    let captured = tmp.path().join("envelope.json");
    let mock = write_script(
        tmp.path(),
        "mock-wyvern",
        &format!(
            r#"#!/usr/bin/env python3
import json, sys
open({:?}, "w").write(sys.argv[1])
envelope = json.loads(sys.argv[1])
assert envelope["type"] == "question"
print(json.dumps({{
    "questions": envelope["questions"],
    "answers": {{ envelope["questions"][0]["question"]: "A" }},
    "response": ""
}}))
"#,
            captured
        ),
    );

    let mut child = std::process::Command::new("python3")
        .arg(apply_script())
        .arg("--invoke")
        .env("WYVERN_BIN", &mock)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn invoke");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(sample_pretool_use().to_string().as_bytes())
        .expect("stdin write");
    let output = child.wait_with_output().expect("wait");
    assert!(
        output.status.success(),
        "invoke failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let envelope: Value =
        serde_json::from_str(&std::fs::read_to_string(&captured).expect("captured")).unwrap();
    assert_eq!(envelope["type"], "question");
    assert_eq!(envelope["questions"][0]["question"], "Pick one");
    assert_eq!(envelope["questions"][0]["header"], "Pick");
    assert_eq!(envelope["questions"][0]["options"][0]["label"], "A");

    let stdout: Value = serde_json::from_slice(&output.stdout).expect("answers json");
    assert_eq!(stdout["answers"]["Pick one"], "A");
    assert_eq!(stdout["response"], "");
    assert_eq!(stdout["questions"][0]["question"], "Pick one");
}

#[test]
fn invoke_refuses_python_as_wyvern_bin() {
    let output = std::process::Command::new("python3")
        .arg(apply_script())
        .arg("--invoke")
        .env("WYVERN_BIN", "python3")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            child
                .stdin
                .as_mut()
                .expect("stdin")
                .write_all(sample_pretool_use().to_string().as_bytes())?;
            child.wait_with_output()
        })
        .expect("spawn");
    assert_ne!(output.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Python interpreter") || stderr.contains("WYVERN_BIN"),
        "{stderr}"
    );
}
