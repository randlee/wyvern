//! Integration tests for compose-render extension.
//! Match-time PATH checks use [`test_support`] probes so tests never re-read PATH.

mod test_support;

use std::path::PathBuf;

use test_support::{AbsentProbe, PresentProbe};
use wyvern::extensions::{
    build_match_context, expand_command_host, expand_preexec_args, format_extensions_list,
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

/// Without sc-compose on PATH, compose render argv must NOT match.
///
/// Uses [`AbsentProbe`] so the test is deterministic and does not re-read PATH
/// after a presence guard (TOCTOU).
#[test]
fn compose_render_no_match_without_sc_compose() {
    let registry = load_shipped();
    let argv = vec!["compose".to_string(), "render".to_string()];
    let matched = registry.match_argv_with(&argv, &AbsentProbe);
    assert!(
        matched.is_none(),
        "compose render must not match when sc-compose is absent"
    );
}

/// The compose-render extension is always registered and shows in the list.
#[test]
fn compose_render_registered() {
    let registry = load_shipped();
    let compose = registry
        .extensions()
        .iter()
        .find(|e| e.id.as_str() == "compose-render");
    assert!(
        compose.is_some(),
        "compose-render not found in shipped registry"
    );
    let ext = compose.expect("compose-render");
    assert!(
        ext.requires().iter().any(|req| req == "sc-compose"),
        "compose-render must require sc-compose"
    );
}

/// extensions list output includes (requires: sc-compose).
#[test]
fn compose_render_list_shows_requires() {
    let registry = load_shipped();
    let list = format_extensions_list(&registry);
    assert!(
        list.contains("compose-render") && list.contains("sc-compose"),
        "list output must mention compose-render and sc-compose; got: {list}"
    );
}

/// With a present-path probe, compose render argv matches.
#[test]
fn compose_render_matches_when_sc_compose_present() {
    let registry = load_shipped();
    let argv = vec![
        "compose".to_string(),
        "render".to_string(),
        "--root".to_string(),
        "fixtures/compose-minimal".to_string(),
        "--file".to_string(),
        "page.j2".to_string(),
    ];
    let matched = registry
        .match_argv_with(&argv, &PresentProbe)
        .expect("compose render must match with PresentProbe");
    assert_eq!(matched.extension().id.as_str(), "compose-render");
}

/// `--var-file` is forwarded in preexec args (pure expand; no PATH dependency).
#[test]
fn compose_render_var_file_in_expand_args() {
    let registry = load_shipped();
    let argv = vec![
        "compose".to_string(),
        "render".to_string(),
        "--root".to_string(),
        "fixtures/compose-minimal".to_string(),
        "--file".to_string(),
        "page.j2".to_string(),
        "--var-file".to_string(),
        "fixtures/compose-minimal/vars.json".to_string(),
    ];
    let matched = registry
        .match_argv_with(&argv, &PresentProbe)
        .expect("compose render must match with PresentProbe");
    let mut ctx = build_match_context(&matched, matched.extension());
    // `{tmpdir}` is referenced in preexec args; expand_preexec_args does not create it.
    ctx.tmpdir = Some(std::env::temp_dir().join("wyvern-compose-test"));
    let (_cmd, expanded_args) =
        expand_preexec_args(matched.extension(), &ctx).expect("expand args");
    assert!(
        expanded_args
            .iter()
            .any(|a| a == "fixtures/compose-minimal/vars.json"),
        "var-file path must appear in expanded preexec args; got: {expanded_args:?}"
    );
}

/// Expand produces the wizard command regardless of sc-compose presence.
///
/// `expand_and_validate` would run preexec. Phase-2 expand instead requires
/// `{rendered_basename}` and `{tmpdir}` to be seeded — unresolved
/// `{rendered_basename}` is an error, not a leftover template token.
#[test]
fn compose_render_expand_produces_correct_html_path() {
    let registry = load_shipped();
    let argv = vec![
        "compose".to_string(),
        "render".to_string(),
        "--root".to_string(),
        "/some/root".to_string(),
        "--file".to_string(),
        "page.j2".to_string(),
    ];
    let matched = registry
        .match_argv_with(&argv, &PresentProbe)
        .expect("compose render must match with PresentProbe");
    assert_eq!(matched.extension().id.as_str(), "compose-render");
    let mut ctx = build_match_context(&matched, matched.extension());
    ctx.rendered_basename = Some("page.html".to_string());
    ctx.tmpdir = Some(std::env::temp_dir().join("wyvern-compose-expand-test"));
    let (command, host) = expand_command_host(matched.extension(), &ctx).expect("expand");
    assert_eq!(command["type"], "wizard");
    assert_eq!(command["page"]["id"], "compose-preview");
    assert_eq!(command["page"]["html"], "pages/page.html");
    assert_eq!(
        host.ui_root.as_deref(),
        Some(ctx.tmpdir.as_deref().expect("tmpdir"))
    );
}
