//! Integration tests for `wyvern examples list` (bundled example catalog).

use std::process::Command;

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
