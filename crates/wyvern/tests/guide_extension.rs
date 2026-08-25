//! `wyvern guide` expands the welcome hub wizard (REQ-0127).

use wyvern::extensions::{
    build_match_context, expand_and_validate, ExtensionRegistry, SHIPPED_EXTENSIONS_JSON,
};

#[test]
fn guide_argv_prefix_expands_welcome_wizard() {
    let registry = ExtensionRegistry::from_json_str(SHIPPED_EXTENSIONS_JSON).expect("shipped");
    let argv = vec!["guide".to_string()];
    let matched = registry.match_argv(&argv).expect("guide must match");
    assert_eq!(matched.extension().id.as_str(), "guide");
    let ctx = build_match_context(&matched, matched.extension());
    let expanded = expand_and_validate(matched.extension(), &ctx).expect("expand");
    assert_eq!(expanded.command["type"], "wizard");
    assert_eq!(expanded.command["page"]["id"], "home");
    assert_eq!(expanded.command["page"]["title"], "Wyvern Guide");
    assert_eq!(expanded.command["page"]["html"], "pages/home.html");
    let topics = expanded.command["config"]["topics"]
        .as_array()
        .expect("topics");
    assert_eq!(topics.len(), 4);
    let ids: Vec<&str> = topics.iter().filter_map(|t| t["id"].as_str()).collect();
    assert_eq!(ids, ["overview", "questions", "templates", "agent-dag"]);
    let ui_root = expanded
        .host_overrides
        .ui_root
        .expect("guide sets host.ui_root");
    assert!(
        ui_root.ends_with("welcome") || ui_root.join("wizard.json").is_file(),
        "ui_root should be the welcome directory: {}",
        ui_root.display()
    );
}

#[test]
fn guide_help_is_stdout_skill_card() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_wyvern"))
        .args(["guide", "--help"])
        .env("WYVERN_VIEWER", "none")
        .env_remove("WYVERN_LOG")
        .output()
        .expect("spawn");
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("guide"), "{stdout}");
    assert!(
        !stderr.trim_start().starts_with('{'),
        "help must stay stdout: {stderr}"
    );
}
