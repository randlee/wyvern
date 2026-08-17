//! Subprocess tests for preexec spawn vs nonzero-exit recovery (REQ-0136).

use std::fs;
use std::path::Path;
use std::process::Command;

fn wyvern() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_wyvern"));
    cmd.env_remove("WYVERN_LOG");
    cmd.env_remove("WYVERN_VIEWER_BIN");
    cmd.env_remove("CARGO_BIN_EXE_wyvern-viewer");
    cmd.env_remove("WYVERN_SHARE");
    cmd.env("WYVERN_VIEWER", "none");
    cmd
}

fn run_in_dir(args: &[&str], cwd: &Path, path: Option<&Path>) -> (i32, String) {
    let mut cmd = wyvern();
    cmd.args(args).current_dir(cwd);
    if let Some(path) = path {
        cmd.env("PATH", path);
    }
    let output = cmd.output().expect("spawn wyvern");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

fn write_project_registry(dir: &Path, json: &str) {
    let wyvern_dir = dir.join(".wyvern");
    fs::create_dir_all(&wyvern_dir).expect("mkdir .wyvern");
    fs::write(wyvern_dir.join("extensions.json"), json).expect("write registry");
}

fn parse_stderr(stderr: &str) -> serde_json::Value {
    serde_json::from_str(stderr.trim()).unwrap_or_else(|err| {
        panic!("stderr is not JSON ({err}): {stderr}");
    })
}

#[test]
fn spawn_not_found_recovery_mentions_install() {
    let tmp = tempfile::tempdir().expect("tmp");
    write_project_registry(
        tmp.path(),
        r#"{
          "version": 1,
          "extensions": [{
            "id": "missing-bin",
            "match": { "argv_prefix": ["gone"] },
            "preexec": { "cmd": "wyvern-g2-missing-bin-xyz" },
            "expand": { "command": { "type": "markdown", "content": "x" } }
          }]
        }"#,
    );
    let empty = tempfile::tempdir().expect("empty PATH");
    let (code, stderr) = run_in_dir(&["gone"], tmp.path(), Some(empty.path()));
    assert_eq!(code, 3, "stderr={stderr}");
    let value = parse_stderr(&stderr);
    assert_eq!(value["code"], "IO_ERROR");
    assert!(stderr.contains("wyvern-g2-missing-bin-xyz"), "{stderr}");
    let recovery = value["recovery"]
        .as_array()
        .map(|steps| {
            steps
                .iter()
                .filter_map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_default();
    assert!(
        recovery.to_ascii_lowercase().contains("install")
            || recovery.to_ascii_lowercase().contains("path"),
        "{stderr}"
    );
}

#[cfg(unix)]
#[test]
fn nonzero_exit_puts_stderr_in_cause_not_install() {
    let tmp = tempfile::tempdir().expect("tmp");
    write_project_registry(
        tmp.path(),
        r#"{
          "version": 1,
          "extensions": [{
            "id": "failing-helper",
            "match": { "argv_prefix": ["boom"] },
            "preexec": {
              "cmd": "sh",
              "args": ["-c", "echo known-preexec-stderr >&2; exit 7"]
            },
            "expand": { "command": { "type": "markdown", "content": "x" } }
          }]
        }"#,
    );
    let (code, stderr) = run_in_dir(&["boom"], tmp.path(), None);
    assert_eq!(code, 3, "stderr={stderr}");
    let value = parse_stderr(&stderr);
    assert_eq!(value["code"], "IO_ERROR");
    let cause = value["cause"].as_str().unwrap_or_default();
    assert!(
        cause.contains("known-preexec-stderr"),
        "cause must include child stderr: {stderr}"
    );
    let lower = stderr.to_ascii_lowercase();
    assert!(
        !lower.contains("install binaries") && !lower.contains("install 'sh'"),
        "nonzero-exit recovery must not recommend installing binaries: {stderr}"
    );
}

#[cfg(unix)]
#[test]
fn md_nonexistent_csv_does_not_recommend_binary_install() {
    if !wyvern::extensions::binary_on_path("python3") {
        return;
    }
    let tmp = tempfile::tempdir().expect("tmp");
    let missing = tmp.path().join("no-such-file.csv");
    let (code, stderr) = run_in_dir(&["md", missing.to_str().expect("utf8")], tmp.path(), None);
    assert_eq!(code, 3, "stderr={stderr}");
    let value = parse_stderr(&stderr);
    let cause = value["cause"].as_str().unwrap_or_default();
    assert!(
        !cause.is_empty() || stderr.contains("python3") || stderr.contains("No such"),
        "expected structured preexec/IO envelope: {stderr}"
    );
    let lower = stderr.to_ascii_lowercase();
    assert!(
        !lower.contains("install binaries") && !lower.contains("install 'python3'"),
        "python3 ran; recovery must not recommend binary install: {stderr}"
    );
}
