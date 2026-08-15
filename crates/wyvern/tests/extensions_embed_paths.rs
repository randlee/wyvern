//! `{wyvern_share}` resolves extensions.json and scripts in dev + embedded layouts.

use std::path::{Path, PathBuf};

use wyvern::extensions::{
    find_workspace_root, resolve_wyvern_share, resolve_wyvern_share_with, ExtensionRegistry,
    ScriptAssets, ShareAssets, SHIPPED_EXTENSIONS_JSON,
};

struct EnvVarGuard {
    key: &'static str,
    prev: Option<String>,
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        match &self.prev {
            Some(value) => std::env::set_var(self.key, value),
            None => std::env::remove_var(self.key),
        }
    }
}

fn remove_env_var_guard(key: &'static str) -> EnvVarGuard {
    let prev = std::env::var(key).ok();
    std::env::remove_var(key);
    EnvVarGuard { key, prev }
}

#[test]
fn shipped_extensions_json_is_valid() {
    ExtensionRegistry::from_json_str(SHIPPED_EXTENSIONS_JSON).expect("shipped JSON");
}

#[test]
fn wyvern_share_resolve_finds_extensions_json() {
    let _guard = remove_env_var_guard("WYVERN_SHARE");
    let registry = ExtensionRegistry::load_default().expect("load_default");
    assert!(registry
        .extensions()
        .iter()
        .any(|ext| ext.id == "markdown-suffix"));
    let share = resolve_wyvern_share();
    assert!(
        share.join("extensions.json").is_file(),
        "extensions.json missing under {}",
        share.display()
    );
}

#[test]
fn shared_assets_embed_contains_extensions_json() {
    let names: Vec<String> = ShareAssets::iter().map(|p| p.to_string()).collect();
    assert!(
        names
            .iter()
            .any(|n| n == "extensions.json" || n.ends_with("extensions.json")),
        "ShareAssets missing extensions.json: {names:?}"
    );
}

#[test]
fn script_assets_embed_contains_placeholder() {
    let names: Vec<String> = ScriptAssets::iter().map(|p| p.to_string()).collect();
    assert!(
        names
            .iter()
            .any(|n| n == "placeholder.py" || n.ends_with("placeholder.py")),
        "ScriptAssets missing placeholder.py: {names:?}"
    );
}

#[test]
fn extensions_embed_paths_unified_share_has_registry_and_scripts() {
    let _guard = remove_env_var_guard("WYVERN_SHARE");
    let share = resolve_wyvern_share();
    assert!(
        share.join("extensions.json").is_file(),
        "extensions.json missing under {}: {}",
        share.display(),
        share.join("extensions.json").display()
    );
    let scripts = share.join("scripts/ext");
    assert!(
        scripts.is_dir(),
        "scripts/ext missing under {}",
        share.display()
    );
    let json = std::fs::read_to_string(share.join("extensions.json")).expect("read");
    assert!(json.contains("markdown-suffix"), "{json}");
}

#[test]
fn extensions_embed_paths_dev_workspace_layout() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = find_workspace_root(&manifest).expect("workspace from crate dir");
    assert!(
        workspace.join("share/wyvern/extensions.json").is_file(),
        "dev layout missing share/wyvern/extensions.json under {}",
        workspace.display()
    );
    assert!(
        workspace.join("scripts/ext").is_dir(),
        "dev layout missing scripts/ext under {}",
        workspace.display()
    );
    let shipped = std::fs::read_to_string(workspace.join("share/wyvern/extensions.json"))
        .expect("read shipped");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&shipped).expect("file json"),
        serde_json::from_str::<serde_json::Value>(SHIPPED_EXTENSIONS_JSON).expect("embed json")
    );
}

#[test]
fn extensions_embed_paths_embedded_extract_layout() {
    let _guard = remove_env_var_guard("WYVERN_SHARE");
    let tmp = tempfile::tempdir().expect("tmp");
    // No workspace in tmp cwd/exe — force embed extract.
    let share = resolve_wyvern_share_with(None, Some(tmp.path()), Some(tmp.path()), None, true);
    assert!(
        share.join("extensions.json").is_file() || share == Path::new("share/wyvern"),
        "embedded or fallback share: {}",
        share.display()
    );
    if share.join("extensions.json").is_file() {
        assert!(share.join("scripts/ext").is_dir() || share.join("scripts/ext").exists());
    }
}
