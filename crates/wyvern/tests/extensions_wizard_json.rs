//! Positional `wizard.json` filename match: load command file and infer ui_root.

use std::path::PathBuf;

use wyvern::extensions::{
    build_match_context, expand_and_validate, infer_wizard_root, ExtensionRegistry,
    SHIPPED_EXTENSIONS_JSON,
};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root")
}

fn workspace_share_dir() -> PathBuf {
    workspace_root().join("share/wyvern")
}

fn isolate_workspace_share() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        std::env::set_var("WYVERN_SHARE", workspace_share_dir());
    });
}

fn load_shipped() -> ExtensionRegistry {
    isolate_workspace_share();
    ExtensionRegistry::load_default().unwrap_or_else(|_| {
        ExtensionRegistry::from_json_str(SHIPPED_EXTENSIONS_JSON).expect("shipped")
    })
}

fn fixture(rel: &str) -> PathBuf {
    workspace_root().join(rel)
}

fn wyvern_bin() -> std::process::Command {
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_wyvern"));
    cmd.env_remove("WYVERN_LOG");
    cmd.env("WYVERN_SHARE", workspace_share_dir());
    cmd
}

#[test]
fn extensions_wizard_json_expands_turbo_flow() {
    let registry = load_shipped();
    let path = fixture("examples/wizards/turbo-flow/wizard.json");
    assert!(path.is_file(), "fixture missing: {}", path.display());
    let argv = vec![path.to_string_lossy().into_owned()];
    let matched = registry
        .match_argv(&argv)
        .expect("wizard-json-suffix should match");
    assert_eq!(matched.extension().id, "wizard-json-suffix");

    let ctx = build_match_context(&matched, matched.extension());
    let expanded = expand_and_validate(matched.extension(), &ctx).expect("expand");

    let on_disk: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("read wizard.json"))
            .expect("parse wizard.json");
    assert_eq!(expanded.command["type"], "wizard");
    assert_eq!(expanded.command, on_disk);

    let expected_root = infer_wizard_root(&path);
    assert_eq!(
        expanded.host_overrides.ui_root.as_deref(),
        Some(expected_root.as_path())
    );
    assert!(
        expected_root.ends_with("turbo-flow"),
        "ui_root={} expected .../turbo-flow",
        expected_root.display()
    );
}

#[test]
fn extensions_wizard_json_expands_single_page() {
    let registry = load_shipped();
    let path = fixture("examples/wizards/single-page/wizard.json");
    assert!(path.is_file(), "fixture missing: {}", path.display());
    let argv = vec![path.to_string_lossy().into_owned()];
    let matched = registry
        .match_argv(&argv)
        .expect("wizard-json-suffix should match");
    assert_eq!(matched.extension().id, "wizard-json-suffix");

    let ctx = build_match_context(&matched, matched.extension());
    let expanded = expand_and_validate(matched.extension(), &ctx).expect("expand");
    assert_eq!(expanded.command["type"], "wizard");
    assert_eq!(expanded.command["page"]["html"], "pages/only.html");

    let expected_root = infer_wizard_root(&path);
    assert_eq!(
        expanded.host_overrides.ui_root.as_deref(),
        Some(expected_root.as_path())
    );
    assert!(
        expected_root.ends_with("single-page"),
        "ui_root={} expected .../single-page",
        expected_root.display()
    );
}

#[test]
fn extensions_wizard_json_list_shows_all_three() {
    let registry = load_shipped();
    let ids: Vec<&str> = registry
        .extensions()
        .iter()
        .map(|ext| ext.id.as_str())
        .collect();
    assert!(
        ids.contains(&"markdown-suffix"),
        "missing markdown-suffix in {ids:?}"
    );
    assert!(
        ids.contains(&"html-suffix"),
        "missing html-suffix in {ids:?}"
    );
    assert!(
        ids.contains(&"wizard-json-suffix"),
        "missing wizard-json-suffix in {ids:?}"
    );

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
    assert!(stdout.contains("html-suffix"), "{stdout}");
    assert!(stdout.contains("wizard-json-suffix"), "{stdout}");
    assert!(stdout.contains("suffix: .html"), "{stdout}");
    assert!(stdout.contains("filename: wizard.json"), "{stdout}");
}
