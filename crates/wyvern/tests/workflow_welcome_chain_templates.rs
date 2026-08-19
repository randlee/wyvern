//! Welcome Templates finish JSON → CLI resolves the template-picker hop.

use std::path::PathBuf;

use serde_json::json;
use wyvern::{resolve_next_wizard, Allowlist};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn workspace_share() -> PathBuf {
    workspace_root().join("share/wyvern")
}

fn welcome_templates_next_wizard() -> serde_json::Value {
    json!({
        "path": "{wyvern_share}/examples/template-picker/wizard.json",
        "input": { "from": "welcome" },
        "ui_root": "{wyvern_share}/examples/template-picker"
    })
}

#[test]
fn templates_page_emits_required_next_wizard() {
    let page = workspace_share().join("welcome/pages/templates.html");
    let html = std::fs::read_to_string(page).expect("templates.html");
    assert!(html.contains("wizardNextWizard"));
    assert!(html.contains("{wyvern_share}/examples/template-picker/wizard.json"));
    assert!(html.contains("{wyvern_share}/examples/template-picker"));
    assert!(html.contains(r#""from": "welcome""#) || html.contains("from: \"welcome\""));
}

#[test]
fn cli_resolves_welcome_templates_hop() {
    let allowlist = Allowlist {
        share_root: workspace_share(),
        cwd: workspace_root(),
        wizard_dir: workspace_share().join("welcome"),
    };
    let finish = json!({
        "button": "finish",
        "data": {},
        "stack": [],
        "next_wizard": welcome_templates_next_wizard()
    });
    let next = resolve_next_wizard(&finish, &allowlist)
        .expect("resolve")
        .expect("hop");
    assert_eq!(next.input, json!({ "from": "welcome" }));
    assert_eq!(next.command["type"], "wizard");
    assert_eq!(next.command["page"]["id"], "pick");
    assert_eq!(
        next.command["workflow"]["post"],
        "{wyvern_share}/scripts/ext/apply-template.py"
    );
    assert!(
        next.ui_root.ends_with("template-picker"),
        "ui_root={}",
        next.ui_root.display()
    );
    assert!(
        next.wizard_dir.join("wizard.json").is_file(),
        "wizard_dir={}",
        next.wizard_dir.display()
    );
}
