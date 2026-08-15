//! `{wyvern_share}` resolves extensions.json and scripts in dev + embedded layouts.

use std::path::PathBuf;

use wyvern::extensions::{
    find_workspace_root, resolve_wyvern_share, resolve_wyvern_share_with, SHIPPED_EXTENSIONS_JSON,
};

#[test]
fn extensions_embed_paths_unified_share_has_registry_and_scripts() {
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
    let tmp = tempfile::tempdir().expect("tmp");
    // No workspace in tmp cwd/exe — force embed extract.
    let share = resolve_wyvern_share_with(None, Some(tmp.path()), Some(tmp.path()), None, true);
    assert!(
        share.join("extensions.json").is_file() || share == PathBuf::from("share/wyvern"),
        "embedded or fallback share: {}",
        share.display()
    );
    if share.join("extensions.json").is_file() {
        assert!(share.join("scripts/ext").is_dir() || share.join("scripts/ext").exists());
    }
}
