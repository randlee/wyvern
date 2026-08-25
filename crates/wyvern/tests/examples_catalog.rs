//! Integration tests for `wyvern examples list` (bundled example catalog).

use std::path::{Path, PathBuf};
use std::process::Command;

use wyvern::examples::validate_example_folder_readmes;

fn wyvern() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_wyvern"));
    cmd.env_remove("WYVERN_LOG");
    cmd.env_remove("WYVERN_SHARE");
    cmd.env("WYVERN_VIEWER", "none");
    cmd
}

fn run(args: &[&str]) -> (i32, String, String) {
    let output = wyvern().args(args).output().expect("spawn wyvern");
    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    (code, stdout, stderr)
}

#[test]
fn examples_list_includes_shipped_examples() {
    let (code, stdout, stderr) = run(&["examples", "list"]);
    assert_eq!(code, 0, "stderr={stderr}");
    for name in [
        "Agent DAG",
        "AskUserQuestion hook",
        "Template picker",
        "XHTML review",
    ] {
        assert!(stdout.contains(name), "missing {name}: {stdout}");
    }
    assert!(stdout.contains("README: examples/"), "{stdout}");
}

#[test]
fn bare_examples_matches_list() {
    let (code, stdout, stderr) = run(&["examples"]);
    assert_eq!(code, 0, "stderr={stderr}");
    let (list_code, list_stdout, list_stderr) = run(&["examples", "list"]);
    assert_eq!(list_code, 0, "stderr={list_stderr}");
    assert_eq!(stdout, list_stdout);
}

#[test]
fn examples_list_json_is_array_of_records() {
    let (code, stdout, stderr) = run(&["examples", "list", "--json"]);
    assert_eq!(code, 0, "stderr={stderr}");
    let records: Vec<serde_json::Value> = serde_json::from_str(stdout.trim()).expect("json array");
    assert!(records.len() >= 4, "length={}", records.len());
    for record in &records {
        assert!(record["name"].is_string(), "{record}");
        assert!(record["description"].is_string(), "{record}");
        assert!(record["readme"].is_string(), "{record}");
        assert!(
            record["readme"]
                .as_str()
                .unwrap_or("")
                .ends_with("README.md"),
            "{record}"
        );
    }
}

#[test]
fn examples_help_mentions_list() {
    let (code, stdout, stderr) = run(&["examples", "--help"]);
    assert_eq!(code, 0, "stderr={stderr}");
    assert!(stdout.contains("list"), "{stdout}");
    assert!(stdout.contains("name:"), "{stdout}");
}

#[test]
fn global_help_mentions_examples_list() {
    let (code, stdout, stderr) = run(&["--help"]);
    assert_eq!(code, 0, "stderr={stderr}");
    assert!(stdout.contains("wyvern examples list"), "{stdout}");
}

fn workspace_share_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .join("../../share/wyvern")
        .canonicalize()
        .unwrap_or_else(|_| manifest.join("../../share/wyvern"))
}

fn packaged_share_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("share/wyvern")
}

fn assert_example_readme_contract(share_root: &Path, label: &str) {
    let violations = validate_example_folder_readmes(share_root).unwrap_or_else(|err| {
        panic!("validate_example_folder_readmes failed for {label}: {err}");
    });
    assert!(
        violations.is_empty(),
        "{label} example README contract violations:\n{}",
        violations
            .iter()
            .map(|v| format!("- {v}"))
            .collect::<Vec<_>>()
            .join("\n")
    );

    let examples_root = share_root.join("examples");
    let dir_count = std::fs::read_dir(&examples_root)
        .unwrap_or_else(|err| panic!("read {label} examples dir: {err}"))
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .count();
    assert!(
        dir_count >= 4,
        "{label} must audit at least four shipped example folders; found {dir_count}"
    );
}

#[test]
fn shipped_example_folders_have_compliant_readme_frontmatter() {
    assert_example_readme_contract(&workspace_share_root(), "workspace share/wyvern");
}

#[test]
fn packaged_example_folders_have_compliant_readme_frontmatter() {
    assert_example_readme_contract(&packaged_share_root(), "crates/wyvern/share/wyvern");
}
