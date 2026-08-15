//! Positional `.html` suffix: expand to a single-page wizard with inferred ui_root.

use std::path::PathBuf;

use wyvern::extensions::{
    build_match_context, expand_and_validate, infer_wizard_root, relpath_from_ui_root,
    ExtensionRegistry,
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

fn load_shipped() -> ExtensionRegistry {
    let defaults = workspace_share_dir().join("extensions.json");
    ExtensionRegistry::load(&defaults, None).expect("shipped registry")
}

fn fixture(rel: &str) -> PathBuf {
    workspace_root().join(rel)
}

#[test]
fn extensions_html_expands_single_page_wizard() {
    let registry = load_shipped();
    let path = fixture("examples/wizards/single-page/pages/only.html");
    assert!(path.is_file(), "fixture missing: {}", path.display());
    let argv = vec![path.to_string_lossy().into_owned()];
    let matched = registry
        .match_argv(&argv)
        .expect("html-suffix should match");
    assert_eq!(matched.extension().id.as_str(), "html-suffix");

    let ctx = build_match_context(&matched, matched.extension());
    let expanded = expand_and_validate(matched.extension(), &ctx).expect("expand");
    assert_eq!(expanded.command["type"], "wizard");
    assert_eq!(expanded.command["page"]["id"], "only");
    assert_eq!(expanded.command["page"]["title"], "only.html");
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
fn extensions_html_wizard_root() {
    let path = fixture("examples/wizards/single-page/pages/only.html");
    assert!(path.is_file(), "fixture missing: {}", path.display());
    let root = infer_wizard_root(&path);
    assert!(
        root.ends_with("single-page"),
        "wizard_root={} expected .../single-page",
        root.display()
    );
    let relpath = relpath_from_ui_root(&path, &root);
    assert_eq!(relpath, "pages/only.html");
}

#[test]
fn extensions_html_md_regression() {
    let registry = load_shipped();
    let argv = vec!["docs/readme.md".to_string()];
    let matched = registry
        .match_argv(&argv)
        .expect(".md should still match markdown-suffix");
    assert_eq!(matched.extension().id.as_str(), "markdown-suffix");
    assert_eq!(matched.path(), Some("docs/readme.md"));

    let ctx = build_match_context(&matched, matched.extension());
    let expanded = expand_and_validate(matched.extension(), &ctx).expect("expand");
    assert_eq!(expanded.command["type"], "markdown");
    assert_eq!(expanded.command["file"], "docs/readme.md");
    assert!(expanded.host_overrides.ui_root.is_none());
}

#[test]
fn extensions_html_non_wizard_json_no_match() {
    let registry = load_shipped();
    for token in [
        "notwizard.json",
        "foo-wizard.json",
        "examples/notwizard.json",
        "dir/foo-wizard.json",
    ] {
        let argv = vec![token.to_string()];
        let matched = registry.match_argv(&argv);
        assert!(
            matched.is_none(),
            "{token} must not match wizard-json-suffix (got {})",
            matched.expect("checked none").extension().id.as_str()
        );
    }
}

#[test]
fn extensions_html_relative_pages_path_infers_parent_of_pages() {
    let resolved = fixture("examples/wizards/single-page/pages/only.html");
    let root = infer_wizard_root(&resolved);
    assert!(root.ends_with("single-page"), "{}", root.display());
    assert_eq!(relpath_from_ui_root(&resolved, &root), "pages/only.html");
}
