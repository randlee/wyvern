//! Subprocess tests for g.2 near-miss diagnostics (REQ-0130, REQ-0136).

use std::fs;
use std::path::{Path, PathBuf};
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

fn run_isolated(args: &[&str], path: &Path) -> (i32, String, String) {
    let output = wyvern()
        .args(args)
        .env("PATH", path)
        .output()
        .expect("spawn wyvern");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

fn run_with_env(args: &[&str], path: Option<&Path>) -> (i32, String, String) {
    let mut cmd = wyvern();
    cmd.args(args);
    if let Some(path) = path {
        cmd.env("PATH", path);
    }
    let output = cmd.output().expect("spawn wyvern");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

fn empty_path_dir() -> tempfile::TempDir {
    tempfile::tempdir().expect("empty PATH dir")
}

#[cfg(unix)]
fn stub_bin_dir(name: &str) -> (tempfile::TempDir, PathBuf) {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().expect("stub bin dir");
    let path = dir.path().join(name);
    fs::write(&path, "#!/bin/sh\nexit 0\n").expect("write stub");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("chmod");
    (dir, path)
}

fn parse_stderr(stderr: &str) -> serde_json::Value {
    serde_json::from_str(stderr.trim()).unwrap_or_else(|err| {
        panic!("stderr is not JSON ({err}): {stderr}");
    })
}

#[test]
fn notes_txt_is_unknown_input_parse_error() {
    let empty = empty_path_dir();
    let (code, _stdout, stderr) = run_isolated(&["notes.txt"], empty.path());
    assert_eq!(code, 2, "stderr={stderr}");
    let value = parse_stderr(&stderr);
    assert_eq!(value["code"], "PARSE_ERROR");
    let message = value["message"].as_str().unwrap_or_default();
    assert!(message.contains("unknown input"), "{stderr}");
    assert!(!stderr.contains("not valid JSON"), "{stderr}");
}

#[test]
fn md_bare_prefix_is_validation_error() {
    let empty = empty_path_dir();
    let (code, _stdout, stderr) = run_isolated(&["md"], empty.path());
    assert_eq!(code, 4, "stderr={stderr}");
    let value = parse_stderr(&stderr);
    assert_eq!(value["code"], "VALIDATION_ERROR");
    assert!(stderr.contains("csv-md"), "{stderr}");
    assert!(stderr.contains("<file.csv>"), "{stderr}");
}

#[test]
fn table_bare_prefix_is_validation_error() {
    let empty = empty_path_dir();
    let (code, _stdout, stderr) = run_isolated(&["table"], empty.path());
    assert_eq!(code, 4, "stderr={stderr}");
    assert!(stderr.contains("<file.csv>"), "{stderr}");
}

#[test]
fn compose_incomplete_prefix_names_compose_render() {
    let empty = empty_path_dir();
    let (code, _stdout, stderr) = run_isolated(&["compose"], empty.path());
    assert_eq!(code, 4, "stderr={stderr}");
    let value = parse_stderr(&stderr);
    assert_eq!(value["code"], "VALIDATION_ERROR");
    assert!(stderr.contains("compose-render"), "{stderr}");
    assert!(stderr.contains("compose render"), "{stderr}");
}

#[test]
fn csv_skipped_requires_names_python3() {
    let empty = empty_path_dir();
    let tmp = tempfile::tempdir().expect("csv dir");
    let csv = tmp.path().join("sample.csv");
    fs::write(&csv, "a,b\n1,2\n").expect("write csv");
    let (code, _stdout, stderr) = run_isolated(&[csv.to_str().expect("utf8")], empty.path());
    assert_eq!(code, 4, "stderr={stderr}");
    let value = parse_stderr(&stderr);
    assert_eq!(value["code"], "VALIDATION_ERROR");
    assert!(stderr.contains("csv-suffix"), "{stderr}");
    assert!(stderr.contains("python3"), "{stderr}");
    assert!(stderr.contains("wyvern"), "{stderr}");
}

#[cfg(unix)]
#[test]
fn compose_render_missing_args_lists_root_and_file() {
    let (stub, _) = stub_bin_dir("sc-compose");
    let (code, _stdout, stderr) = run_isolated(&["compose", "render"], stub.path());
    assert_eq!(code, 4, "stderr={stderr}");
    let value = parse_stderr(&stderr);
    assert_eq!(value["code"], "VALIDATION_ERROR");
    assert!(stderr.contains("--root"), "{stderr}");
    assert!(stderr.contains("--file"), "{stderr}");
}

#[cfg(unix)]
#[test]
fn unexpected_arg_recovery_is_caller_facing() {
    let (stub, _) = stub_bin_dir("sc-compose");
    let (code, _stdout, stderr) = run_isolated(
        &[
            "compose",
            "render",
            "--root",
            "r",
            "--file",
            "f.j2",
            "--undeclared",
            "x",
        ],
        stub.path(),
    );
    assert_eq!(code, 4, "stderr={stderr}");
    assert!(
        !stderr.contains("declare them as {arg:name}") && !stderr.contains("{arg:"),
        "{stderr}"
    );
    assert!(
        stderr.contains("--root") || stderr.contains("Accepted flags"),
        "{stderr}"
    );
}

#[test]
fn inline_json_is_not_classified_as_unknown_input() {
    let (code, _stdout, stderr) = run_with_env(&[r#"{"type":"not-a-dialog"}"#], None);
    assert_ne!(code, 2, "inline JSON must not be UnknownInput: {stderr}");
    assert!(!stderr.contains("unknown input"), "{stderr}");
}

/// Project prefix skill without `requires` — platform-neutral MissingArgs path.
fn write_project_prefix_skill(dir: &Path) {
    let wyvern_dir = dir.join(".wyvern");
    fs::create_dir_all(&wyvern_dir).expect("mkdir .wyvern");
    fs::write(
        wyvern_dir.join("extensions.json"),
        r#"{
          "version": 1,
          "extensions": [{
            "id": "needs-root",
            "description": "Project prefix skill for recovery tests",
            "examples": ["wyvern demo run --root DIR"],
            "match": { "argv_prefix": ["demo", "run"] },
            "expand": { "command": { "type": "markdown", "content": "{arg:root}" } }
          }]
        }"#,
    )
    .expect("write project registry");
}

fn run_in_dir(dir: &Path, args: &[&str]) -> (i32, String, String) {
    let output = wyvern()
        .current_dir(dir)
        .args(args)
        .output()
        .expect("spawn wyvern");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

fn recovery_run_command(stderr: &str) -> String {
    let value = parse_stderr(stderr);
    value["recovery"]
        .as_array()
        .expect("recovery")
        .iter()
        .filter_map(|step| step.as_str())
        .find_map(|step| step.strip_prefix("Run "))
        .filter(|cmd| cmd.starts_with("wyvern ") && cmd.contains("--help"))
        .unwrap_or_else(|| panic!("no Run wyvern … --help recovery in {stderr}"))
        .to_string()
}

#[test]
fn missing_args_recovery_uses_invocation_prefix_and_exits_zero() {
    let tmp = tempfile::tempdir().expect("tmp");
    write_project_prefix_skill(tmp.path());
    let (code, _stdout, stderr) = run_in_dir(tmp.path(), &["demo", "run"]);
    assert_eq!(code, 4, "stderr={stderr}");
    assert!(
        !stderr.contains("wyvern needs-root --help")
            && !stderr.contains("wyvern compose-render --help"),
        "recovery must not use the extension id as argv: {stderr}"
    );
    let recovery = recovery_run_command(&stderr);
    assert_eq!(recovery, "wyvern demo run --help", "{stderr}");
    let tokens: Vec<&str> = recovery
        .strip_prefix("wyvern ")
        .expect("wyvern prefix")
        .split_whitespace()
        .collect();
    let (help_code, help_stdout, help_stderr) = run_in_dir(tmp.path(), &tokens);
    assert_eq!(help_code, 0, "recovery argv must exit 0: {help_stderr}");
    assert!(
        help_stdout.contains("needs-root") || help_stdout.contains("Usage:"),
        "{help_stdout}"
    );
}

#[test]
fn unexpected_arg_recovery_uses_invocation_prefix() {
    let tmp = tempfile::tempdir().expect("tmp");
    write_project_prefix_skill(tmp.path());
    let (code, _stdout, stderr) = run_in_dir(
        tmp.path(),
        &["demo", "run", "--root", "r", "--undeclared", "x"],
    );
    assert_eq!(code, 4, "stderr={stderr}");
    assert!(
        !stderr.contains("declare them as {arg:name}") && !stderr.contains("{arg:"),
        "{stderr}"
    );
    assert!(
        stderr.contains("--root") || stderr.contains("Accepted flags"),
        "{stderr}"
    );
    let recovery = recovery_run_command(&stderr);
    assert_eq!(recovery, "wyvern demo run --help", "{stderr}");
    let tokens: Vec<&str> = recovery
        .strip_prefix("wyvern ")
        .expect("wyvern prefix")
        .split_whitespace()
        .collect();
    let (help_code, _help_stdout, help_stderr) = run_in_dir(tmp.path(), &tokens);
    assert_eq!(help_code, 0, "recovery argv must exit 0: {help_stderr}");
}
