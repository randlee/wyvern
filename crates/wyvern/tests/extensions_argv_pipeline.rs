//! Argv remainder pipeline: host flags stripped; prefix extensions reach the matcher.

use std::path::PathBuf;

use wyvern::extensions::{
    build_match_context, expand_command_host, ExtensionMatch, ExtensionRegistry, HostOverrides,
};
use wyvern::{apply_host_overrides, parse_cli_args};

fn args(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| (*s).to_string()).collect()
}

fn prefix_registry() -> ExtensionRegistry {
    ExtensionRegistry::from_json_str(
        r#"{
          "version": 1,
          "extensions": [
            {
              "id": "compose-render",
              "match": { "argv_prefix": ["compose", "render"] },
              "expand": {
                "command": { "type": "markdown", "content": "{arg:root}" }
              }
            },
            {
              "id": "table-csv",
              "match": { "argv_prefix": ["table"], "arg_suffix": ".csv" },
              "expand": {
                "command": { "type": "markdown", "file": "{path}" }
              }
            },
            {
              "id": "md-csv",
              "match": { "argv_prefix": ["md"], "arg_suffix": ".csv" },
              "expand": {
                "command": { "type": "markdown", "file": "{path}" }
              }
            }
          ]
        }"#,
    )
    .expect("registry")
}

#[test]
fn extensions_argv_pipeline_compose_render_survives_parse() {
    let parsed = parse_cli_args(&args(&["compose", "render", "--root", "/tmp/ui"])).expect("parse");
    assert_eq!(
        parsed.positionals,
        args(&["compose", "render", "--root", "/tmp/ui"])
    );
    let registry = prefix_registry();
    let matched = registry
        .match_argv(&parsed.positionals)
        .expect("compose render must reach matcher");
    assert!(matches!(matched, ExtensionMatch::Prefix { .. }));
    assert_eq!(matched.extension().id, "compose-render");
    let ctx = build_match_context(&matched, matched.extension());
    let (cmd, _) = expand_command_host(matched.extension(), &ctx).expect("expand");
    assert_eq!(cmd["content"], "/tmp/ui");
}

#[test]
fn extensions_argv_pipeline_prefix_suffix_table_and_md() {
    let registry = prefix_registry();
    let table = parse_cli_args(&args(&["table", "report.csv"])).expect("parse");
    let matched = registry
        .match_argv(&table.positionals)
        .expect("table prefix+suffix");
    assert!(matches!(matched, ExtensionMatch::PrefixSuffix { .. }));
    assert_eq!(matched.path(), Some("report.csv"));

    let md = parse_cli_args(&args(&["md", "report.csv"])).expect("parse");
    let matched = registry
        .match_argv(&md.positionals)
        .expect("md prefix+suffix");
    assert_eq!(matched.extension().id, "md-csv");
}

#[test]
fn extensions_argv_pipeline_host_flags_stripped() {
    let parsed = parse_cli_args(&args(&[
        "--viewer",
        "none",
        "--ui-root",
        "./cli-ui",
        "compose",
        "render",
        "--root",
        "R",
    ]))
    .expect("parse");
    assert_eq!(
        parsed.positionals,
        args(&["compose", "render", "--root", "R"])
    );
    assert_eq!(parsed.host.ui_root, PathBuf::from("./cli-ui"));
}

#[test]
fn extensions_argv_pipeline_host_ui_root_overrides_cli() {
    let mut parsed = parse_cli_args(&args(&["--ui-root", "./cli-ui", "doc.md"])).expect("parse");
    apply_host_overrides(
        &mut parsed.host,
        &HostOverrides {
            ui_root: Some(PathBuf::from("/ext-ui")),
        },
    );
    assert_eq!(parsed.host.ui_root, PathBuf::from("/ext-ui"));
}

#[test]
fn extensions_argv_pipeline_version_unchanged() {
    let parsed = parse_cli_args(&args(&["--version"])).expect("parse");
    assert_eq!(parsed.positionals, args(&["--version"]));
    let parsed = parse_cli_args(&args(&["-V"])).expect("parse");
    assert_eq!(parsed.positionals, args(&["-V"]));
}

fn wyvern_bin() -> std::process::Command {
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_wyvern"));
    cmd.env_remove("WYVERN_LOG");
    cmd
}

#[test]
fn extensions_list_prints_markdown_suffix() {
    let output = wyvern_bin()
        .args(["extensions", "list"])
        .output()
        .expect("spawn");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("markdown-suffix"), "{stdout}");
    assert!(stdout.contains("suffix: .md"), "{stdout}");
}

#[test]
fn extensions_version_flag_prints_package_version() {
    let output = wyvern_bin().arg("--version").output().expect("spawn");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")), "{stdout}");
}

#[test]
fn extensions_unknown_suffix_falls_through_to_parse_error() {
    let output = wyvern_bin().arg("notes.txt").output().expect("spawn");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    let json: serde_json::Value = serde_json::from_str(
        stderr
            .lines()
            .rev()
            .find(|l| l.trim_start().starts_with('{'))
            .unwrap_or(stderr.trim()),
    )
    .unwrap_or_else(|_| panic!("stderr not JSON: {stderr}"));
    assert_eq!(json["error"], "parse");
}
