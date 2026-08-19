//! `--invoke`: PreToolUse stdin → question envelope → REQ-0067 answers.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use serde_json::{json, Value};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn apply_script() -> PathBuf {
    workspace_root().join("scripts/ext/apply-askuserquestion-hook.py")
}

fn resolve_python() -> &'static str {
    for name in ["python3", "py", "python"] {
        if std::process::Command::new(name)
            .arg("-c")
            .arg("import sys")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
        {
            return name;
        }
    }
    panic!("python3, py, or python is required for AskUserQuestion invoke tests");
}

fn write_bytes(dir: &Path, name: &str, body: &[u8]) -> PathBuf {
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

fn mock_wyvern(dir: &Path, captured: &Path) -> PathBuf {
    let captured_lit = captured.to_string_lossy().replace('\\', "/");
    let py_body = format!(
        r#"import json, sys
open(r"{captured_lit}", "w").write(sys.argv[1])
envelope = json.loads(sys.argv[1])
assert envelope["type"] == "question"
print(json.dumps({{
    "questions": envelope["questions"],
    "answers": {{ envelope["questions"][0]["question"]: "A" }},
    "response": ""
}}))
"#
    );

    #[cfg(unix)]
    {
        write_bytes(
            dir,
            "mock-wyvern",
            format!("#!/usr/bin/env python3\n{py_body}").as_bytes(),
        )
    }

    #[cfg(windows)]
    {
        let py_path = write_bytes(dir, "mock-wyvern.py", py_body.as_bytes());
        let python = resolve_python();
        let cmd = format!("@echo off\r\n\"{python}\" \"{}\" %*\r\n", py_path.display());
        write_bytes(dir, "mock-wyvern.cmd", cmd.as_bytes())
    }
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

fn run_invoke(wyvern_bin: &Path) -> std::process::Output {
    let mut child = std::process::Command::new(resolve_python())
        .arg(apply_script())
        .arg("--invoke")
        .env("WYVERN_BIN", wyvern_bin)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn invoke");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(sample_pretool_use().to_string().as_bytes())
        .expect("stdin write");
    child.wait_with_output().expect("wait")
}

#[test]
fn invoke_maps_pretooluse_to_question_answers() {
    let tmp = tempfile::tempdir().unwrap();
    let captured = tmp.path().join("envelope.json");
    let mock = mock_wyvern(tmp.path(), &captured);

    let output = run_invoke(&mock);
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
    let python = resolve_python();
    let output = std::process::Command::new(python)
        .arg(apply_script())
        .arg("--invoke")
        .env("WYVERN_BIN", python)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
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
