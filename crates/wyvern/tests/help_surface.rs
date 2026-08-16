//! Integration tests for the g.1 help surface (REQ-0134, REQ-0135).

use std::process::Command;

fn wyvern() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_wyvern"));
    // Isolate from developer/CI env so help stdout/stderr assertions stay stable.
    cmd.env_remove("WYVERN_LOG");
    cmd.env_remove("WYVERN_VIEWER_BIN");
    cmd.env_remove("CARGO_BIN_EXE_wyvern-viewer");
    cmd.env_remove("WYVERN_SHARE");
    cmd.env("WYVERN_VIEWER", "none");
    cmd
}

fn run_help(args: &[&str]) -> (i32, String, String) {
    let output = wyvern().args(args).output().expect("spawn wyvern");
    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    (code, stdout, stderr)
}

/// Restricted PATH so help cannot depend on optional binaries such as sc-compose.
fn run_help_without_optional_bins(args: &[&str]) -> (i32, String, String) {
    let empty = tempfile::tempdir().expect("empty PATH dir");
    let output = wyvern()
        .args(args)
        .env("PATH", empty.path())
        .output()
        .expect("spawn wyvern");
    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    (code, stdout, stderr)
}

fn assert_global_help(stdout: &str) {
    assert!(stdout.contains(".csv"), "{stdout}");
    assert!(stdout.contains("table"), "{stdout}");
    assert!(stdout.contains("md data.csv"), "{stdout}");
    assert!(stdout.contains("compose render"), "{stdout}");
    assert!(stdout.contains("--env-prefix"), "{stdout}");
}

#[test]
fn global_help_long_flag_exits_zero() {
    let (code, stdout, stderr) = run_help(&["--help"]);
    assert_eq!(code, 0, "stderr={stderr}");
    assert_global_help(&stdout);
}

#[test]
fn global_help_short_flag_exits_zero() {
    let (code, stdout, stderr) = run_help(&["-h"]);
    assert_eq!(code, 0, "stderr={stderr}");
    assert_global_help(&stdout);
}

#[test]
fn global_help_subcommand_matches_long_flag() {
    let (code_help, stdout_help, stderr_help) = run_help(&["help"]);
    let (code_flag, stdout_flag, stderr_flag) = run_help(&["--help"]);
    assert_eq!(code_help, 0, "stderr={stderr_help}");
    assert_eq!(code_flag, 0, "stderr={stderr_flag}");
    assert_eq!(stdout_help, stdout_flag);
    assert_global_help(&stdout_help);
}

#[test]
fn global_help_after_host_flag_strip() {
    let (code, stdout, stderr) = run_help(&["--viewer", "none", "--help"]);
    assert_eq!(code, 0, "stderr={stderr}");
    assert_global_help(&stdout);
}

#[test]
fn compose_render_help_without_sc_compose() {
    let (code, stdout, stderr) = run_help_without_optional_bins(&["compose", "render", "--help"]);
    assert_eq!(code, 0, "stderr={stderr}");
    assert!(stdout.contains("--root"), "{stdout}");
    assert!(stdout.contains("--file"), "{stdout}");
    assert!(stdout.contains("Requires:"), "{stdout}");
    assert!(stdout.contains("Example:"), "{stdout}");
    assert!(
        !stderr.trim_start().starts_with('{'),
        "help must not emit stderr JSON: {stderr}"
    );
}

#[test]
fn compose_render_help_short_flag() {
    let (code, stdout, stderr) = run_help_without_optional_bins(&["compose", "render", "-h"]);
    assert_eq!(code, 0, "stderr={stderr}");
    assert!(stdout.contains("--root"), "{stdout}");
    assert!(stdout.contains("Requires:"), "{stdout}");
}

#[test]
fn md_help_without_csv_path() {
    let (code, stdout, stderr) = run_help(&["md", "--help"]);
    assert_eq!(code, 0, "stderr={stderr}");
    assert!(stdout.contains("Usage:"), "{stdout}");
    assert!(stdout.contains("wyvern md"), "{stdout}");
    assert!(stdout.contains("Example:"), "{stdout}");
}

#[test]
fn table_help_without_csv_path() {
    let (code, stdout, stderr) = run_help(&["table", "--help"]);
    assert_eq!(code, 0, "stderr={stderr}");
    assert!(stdout.contains("Usage:"), "{stdout}");
    assert!(stdout.contains("wyvern table"), "{stdout}");
    assert!(stdout.contains("Example:"), "{stdout}");
}

#[test]
fn extensions_help_mentions_list() {
    let (code, stdout, stderr) = run_help(&["extensions", "--help"]);
    assert_eq!(code, 0, "stderr={stderr}");
    assert!(stdout.contains("list"), "{stdout}");
    assert!(
        !stdout.contains("show"),
        "g.1 must not require show: {stdout}"
    );
}

#[test]
fn extensions_help_short_flag() {
    let (code, stdout, stderr) = run_help(&["extensions", "-h"]);
    assert_eq!(code, 0, "stderr={stderr}");
    assert!(stdout.contains("list"), "{stdout}");
}

#[test]
fn browsers_help_mentions_list_and_refresh() {
    let (code, stdout, stderr) = run_help(&["browsers", "--help"]);
    assert_eq!(code, 0, "stderr={stderr}");
    assert!(stdout.contains("list"), "{stdout}");
    assert!(stdout.contains("refresh"), "{stdout}");
}

#[test]
fn browsers_help_short_flag() {
    let (code, stdout, stderr) = run_help(&["browsers", "-h"]);
    assert_eq!(code, 0, "stderr={stderr}");
    assert!(stdout.contains("list"), "{stdout}");
    assert!(stdout.contains("refresh"), "{stdout}");
}
