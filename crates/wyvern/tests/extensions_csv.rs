//! Integration tests for csv-suffix, csv-table-alias, and csv-md extensions.

mod test_support;

use std::path::PathBuf;

use test_support::{AbsentProbe, PresentProbe};
use wyvern::extensions::{
    binary_on_path, build_match_context, expand_and_validate, expand_command_host,
    ExtensionRegistry,
};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root")
}

fn load_shipped() -> ExtensionRegistry {
    let defaults = workspace_root().join("share/wyvern/extensions.json");
    ExtensionRegistry::load(&defaults, None).expect("shipped registry")
}

fn sample_csv() -> String {
    workspace_root()
        .join("fixtures/sample.csv")
        .to_string_lossy()
        .into_owned()
}

/// csv-suffix, csv-table-alias, and csv-md are registered in the shipped file.
#[test]
fn csv_extensions_registered() {
    let registry = load_shipped();
    let ids: Vec<&str> = registry
        .extensions()
        .iter()
        .map(|ext| ext.id.as_str())
        .collect();
    assert!(
        ids.contains(&"csv-suffix"),
        "csv-suffix missing from shipped registry: {ids:?}"
    );
    assert!(
        ids.contains(&"csv-table-alias"),
        "csv-table-alias missing from shipped registry: {ids:?}"
    );
    assert!(
        ids.contains(&"csv-md"),
        "csv-md missing from shipped registry: {ids:?}"
    );
}

/// csv-suffix matches .csv files.
#[test]
fn csv_suffix_matches_csv_file() {
    let registry = load_shipped();
    let argv = vec![sample_csv()];
    let matched = registry
        .match_argv_with(&argv, &PresentProbe)
        .expect("csv-suffix must match .csv with python3 present");
    assert_eq!(matched.extension().id.as_str(), "csv-suffix");
}

/// csv-suffix does NOT match when python3 is absent (requires: ["python3"]).
#[test]
fn csv_suffix_no_match_without_python3() {
    let registry = load_shipped();
    let argv = vec![sample_csv()];
    let matched = registry.match_argv_with(&argv, &AbsentProbe);
    assert!(
        matched.is_none(),
        "csv-suffix must not match when python3 absent"
    );
}

/// Injected [`AbsentProbe`] reports python3 absent → csv-suffix does not match.
///
/// Named for the sprint gate `extensions_csv_requires_python3`.
#[test]
fn extensions_csv_requires_python3() {
    csv_suffix_no_match_without_python3();
}

/// csv-table-alias matches `table report.csv`.
#[test]
fn csv_table_alias_matches() {
    let registry = load_shipped();
    let argv = vec!["table".to_string(), sample_csv()];
    let matched = registry
        .match_argv_with(&argv, &PresentProbe)
        .expect("csv-table-alias must match 'table *.csv'");
    assert_eq!(matched.extension().id.as_str(), "csv-table-alias");
}

/// csv-table-alias inherits `requires: ["python3"]` from csv-suffix.
#[test]
fn csv_table_alias_no_match_without_python3() {
    let registry = load_shipped();
    let argv = vec!["table".to_string(), sample_csv()];
    let matched = registry.match_argv_with(&argv, &AbsentProbe);
    assert!(
        matched.is_none(),
        "csv-table-alias must not match when python3 absent"
    );
}

/// csv-md matches `md report.csv`.
#[test]
fn csv_md_matches() {
    let registry = load_shipped();
    let argv = vec!["md".to_string(), sample_csv()];
    let matched = registry
        .match_argv_with(&argv, &PresentProbe)
        .expect("csv-md must match 'md *.csv'");
    assert_eq!(matched.extension().id.as_str(), "csv-md");
}

/// csv-suffix expand produces wizard command with pages/view.html.
///
/// `expand_and_validate` runs preexec. Phase-2 expand is used here so the
/// test does not depend on python3 being on PATH.
#[test]
fn csv_suffix_expand_produces_wizard() {
    let registry = load_shipped();
    let argv = vec![sample_csv()];
    let matched = registry
        .match_argv_with(&argv, &PresentProbe)
        .expect("must match");
    let mut ctx = build_match_context(&matched, matched.extension());
    ctx.tmpdir =
        Some(std::env::temp_dir().join(format!("wyvern-csv-expand-test-{}", std::process::id())));
    let (command, host) = expand_command_host(matched.extension(), &ctx).expect("expand");
    assert_eq!(command["type"], "wizard");
    assert_eq!(command["page"]["html"], "pages/view.html");
    assert_eq!(command["page"]["layout"], "workspace");
    // Schema clamp: sprint asked 960x640; viewer max is 800x600.
    assert_eq!(command["width"], 800);
    assert_eq!(command["height"], 600);
    assert_eq!(
        host.ui_root.as_deref(),
        Some(ctx.tmpdir.as_deref().expect("tmpdir"))
    );
}

/// `wyvern table report.csv` expand is identical to the suffix form.
#[test]
fn csv_table_alias_expand_matches_suffix() {
    let registry = load_shipped();
    let path = sample_csv();
    let suffix_argv = vec![path.clone()];
    let alias_argv = vec!["table".to_string(), path];
    let suffix = registry
        .match_argv_with(&suffix_argv, &PresentProbe)
        .expect("suffix");
    let alias = registry
        .match_argv_with(&alias_argv, &PresentProbe)
        .expect("alias");
    let mut suffix_ctx = build_match_context(&suffix, suffix.extension());
    let mut alias_ctx = build_match_context(&alias, alias.extension());
    let tmp = std::env::temp_dir().join(format!(
        "wyvern-csv-alias-expand-test-{}",
        std::process::id()
    ));
    suffix_ctx.tmpdir = Some(tmp.clone());
    alias_ctx.tmpdir = Some(tmp);
    let (suffix_cmd, suffix_host) =
        expand_command_host(suffix.extension(), &suffix_ctx).expect("suffix expand");
    let (alias_cmd, alias_host) =
        expand_command_host(alias.extension(), &alias_ctx).expect("alias expand");
    assert_eq!(suffix_cmd, alias_cmd);
    assert_eq!(suffix_host, alias_host);
}

/// csv-md expand produces markdown command.
#[test]
fn csv_md_expand_produces_markdown() {
    let registry = load_shipped();
    let argv = vec!["md".to_string(), sample_csv()];
    let matched = registry
        .match_argv_with(&argv, &PresentProbe)
        .expect("must match");
    let ext = matched.extension();
    assert!(
        ext.preexec.as_ref().is_some_and(|pre| pre.stdout.is_some()),
        "csv-md must have preexec.stdout configured"
    );
    let mut ctx = build_match_context(&matched, ext);
    ctx.preexec_stdout = Some("| name | age |\n| --- | --- |\n| Alice | 30 |\n".into());
    let (command, host) = expand_command_host(ext, &ctx).expect("expand");
    assert_eq!(command["type"], "markdown");
    assert_eq!(command["title"], "sample.csv");
    assert_eq!(
        command["content"],
        ctx.preexec_stdout.as_deref().expect("seeded stdout")
    );
    assert!(host.ui_root.is_none());
}

/// Preexec stages the tmpdir layout when python3 is actually on PATH.
#[test]
fn csv_suffix_preexec_writes_tmpdir_layout() {
    // TOCTOU: single binary_on_path guard is adequate for CI
    if !binary_on_path("python3") {
        return;
    }
    let registry = load_shipped();
    let argv = vec![sample_csv()];
    let matched = registry
        .match_argv_with(&argv, &PresentProbe)
        .expect("must match");
    let ctx = build_match_context(&matched, matched.extension());
    let expanded = expand_and_validate(matched.extension(), &ctx).expect("expand");
    assert_eq!(expanded.command["type"], "wizard");
    assert_eq!(expanded.command["page"]["html"], "pages/view.html");
    let tmp = expanded
        .temp_guard
        .as_ref()
        .expect("temp_guard")
        .path()
        .to_path_buf();
    assert!(
        tmp.join("data/rows.json").is_file(),
        "missing {}",
        tmp.join("data/rows.json").display()
    );
    assert!(tmp.join("pages/view.html").is_file());
    assert!(tmp.join("shared/table.js").is_file());
    assert!(tmp.join("shared/table.css").is_file());
}

/// `wyvern md fixtures/sample.csv` expand → valid markdown command JSON.
#[test]
fn csv_md_expand_and_validate_markdown_content() {
    // TOCTOU: single binary_on_path guard is adequate for CI
    if !binary_on_path("python3") {
        return;
    }
    let registry = load_shipped();
    let argv = vec!["md".to_string(), sample_csv()];
    let matched = registry
        .match_argv_with(&argv, &PresentProbe)
        .expect("must match");
    let ctx = build_match_context(&matched, matched.extension());
    let expanded = expand_and_validate(matched.extension(), &ctx).expect("expand");
    assert_eq!(expanded.command["type"], "markdown");
    let content = expanded.command["content"]
        .as_str()
        .expect("markdown content");
    assert!(content.contains("Alice"), "{content}");
    assert!(content.contains('|'), "{content}");
}
