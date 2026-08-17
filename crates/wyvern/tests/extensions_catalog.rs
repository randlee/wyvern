//! Catalog tests for `extensions list` / `show` (REQ-0132, REQ-0137).

use std::process::Command;

use wyvern::extensions::{
    build_skill_record, build_skill_records, format_skill_card, ExtensionRegistry,
    PathRequiresProbe, SHIPPED_EXTENSIONS_JSON,
};
use wyvern::usage_message;

const REQUIRED_KEYS: &[&str] = &[
    "id",
    "match_kind",
    "invocation",
    "requires",
    "args",
    "expands_to",
    "description",
    "examples",
    "extends",
];

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

fn stderr_json(stderr: &str) -> serde_json::Value {
    let json_line = stderr
        .lines()
        .rev()
        .find(|line| line.trim_start().starts_with('{'))
        .unwrap_or(stderr.trim());
    serde_json::from_str(json_line.trim()).unwrap_or_else(|err| {
        panic!("stderr is not JSON ({err}): {stderr:?}");
    })
}

fn shipped_registry() -> ExtensionRegistry {
    ExtensionRegistry::from_json_str(SHIPPED_EXTENSIONS_JSON).expect("shipped registry")
}

fn assert_record_keys(value: &serde_json::Value) {
    let obj = value.as_object().expect("record object");
    for key in REQUIRED_KEYS {
        assert!(obj.contains_key(*key), "missing key {key} in {value}");
    }
    assert!(
        value["extends"].is_null() || value["extends"].is_string(),
        "extends must be string or null: {value}"
    );
    assert!(
        value["description"].is_null() || value["description"].is_string(),
        "description must be string or null: {value}"
    );
    assert!(
        value["examples"].is_array(),
        "examples must be array: {value}"
    );
}

#[test]
fn list_json_is_array_of_skill_records() {
    let (code, stdout, stderr) = run(&["extensions", "list", "--json"]);
    assert_eq!(code, 0, "stderr={stderr}");
    assert!(
        stdout.starts_with('['),
        "stdout must start with [: {stdout}"
    );
    let records: Vec<serde_json::Value> = serde_json::from_str(stdout.trim()).expect("JSON array");
    assert!(records.len() >= 7, "length={}", records.len());
    for record in &records {
        assert_record_keys(record);
    }
}

#[test]
fn bare_extensions_matches_list_and_includes_markdown_suffix() {
    let (code, stdout, stderr) = run(&["extensions"]);
    assert_eq!(code, 0, "stderr={stderr}");
    assert!(
        stdout.contains("markdown-suffix"),
        "bare extensions must list markdown-suffix: {stdout}"
    );
    let (list_code, list_stdout, list_stderr) = run(&["extensions", "list"]);
    assert_eq!(list_code, 0, "stderr={list_stderr}");
    assert_eq!(
        stdout, list_stdout,
        "bare extensions must match extensions list"
    );
}

#[test]
fn list_text_marks_requires_availability() {
    let (code, stdout, stderr) = run(&["extensions", "list"]);
    assert_eq!(code, 0, "stderr={stderr}");
    assert!(
        stdout.contains("[available]") || stdout.contains("[missing]"),
        "{stdout}"
    );
    assert!(
        stdout.contains("csv-table-alias")
            && stdout.contains("csv-suffix")
            && (stdout.contains("Extends") || stdout.to_ascii_lowercase().contains("alias")),
        "csv-table-alias must note extends/alias of csv-suffix: {stdout}"
    );
}

#[test]
fn show_csv_md_matches_json_record() {
    let (code, stdout, stderr) = run(&["extensions", "show", "csv-md"]);
    assert_eq!(code, 0, "stderr={stderr}");
    let (json_code, json_stdout, json_stderr) = run(&["extensions", "show", "csv-md", "--json"]);
    assert_eq!(json_code, 0, "stderr={json_stderr}");
    let record: serde_json::Value = serde_json::from_str(json_stdout.trim()).expect("object");
    assert_record_keys(&record);
    assert_eq!(record["id"], "csv-md");
    for fact in [
        record["id"].as_str(),
        record["match_kind"].as_str(),
        record["invocation"].as_str(),
        record["expands_to"].as_str(),
        record["description"].as_str(),
    ]
    .into_iter()
    .flatten()
    {
        assert!(stdout.contains(fact), "text missing {fact:?}: {stdout}");
    }
    if let Some(examples) = record["examples"].as_array() {
        for example in examples.iter().filter_map(|v| v.as_str()) {
            assert!(
                stdout.contains(example),
                "text missing example {example}: {stdout}"
            );
        }
    }
    if let Some(requires) = record["requires"].as_array() {
        for req in requires {
            if let Some(binary) = req["binary"].as_str() {
                assert!(stdout.contains(binary), "text missing {binary}: {stdout}");
            }
        }
    }
}

#[test]
fn show_unknown_id_exits_nonzero() {
    let (code, stdout, stderr) = run(&["extensions", "show", "no-such-id"]);
    assert_ne!(code, 0, "stdout={stdout} stderr={stderr}");
    let value = stderr_json(&stderr);
    assert_eq!(value["code"], "VALIDATION_ERROR");
    assert!(
        value["message"]
            .as_str()
            .unwrap_or("")
            .contains("no-such-id"),
        "{stderr}"
    );
}

#[test]
fn list_unknown_flag_is_validation_error() {
    let (code, stdout, stderr) = run(&["extensions", "list", "--foo"]);
    assert_ne!(code, 0, "stdout={stdout} stderr={stderr}");
    let value = stderr_json(&stderr);
    assert_eq!(value["error"], "validation");
    assert_eq!(value["code"], "VALIDATION_ERROR");
    assert!(
        value["message"].as_str().unwrap_or("").contains("--foo"),
        "{stderr}"
    );
}

#[test]
fn extensions_help_mentions_show() {
    let (code, stdout, stderr) = run(&["extensions", "--help"]);
    assert_eq!(code, 0, "stderr={stderr}");
    assert!(stdout.contains("show"), "{stdout}");
}

#[test]
fn req_0132_catalog_covers_list_json_and_show() {
    let registry = shipped_registry();
    let records = build_skill_records(&registry, &PathRequiresProbe);
    assert!(records.len() >= 7);
    let json = serde_json::to_value(&records).expect("serialize");
    assert!(json.is_array());
    for record in json.as_array().expect("array") {
        assert_record_keys(record);
    }
    let csv_md = registry
        .extensions()
        .iter()
        .find(|ext| ext.id.as_str() == "csv-md")
        .expect("csv-md");
    let card = format_skill_card(&build_skill_record(csv_md, &PathRequiresProbe));
    assert!(card.contains("csv-md"), "{card}");
    assert!(card.contains("wyvern md"), "{card}");
}

#[test]
fn req_0137_registry_help_parity() {
    let registry = shipped_registry();
    let help = usage_message();
    let (code, list_json, stderr) = run(&["extensions", "list", "--json"]);
    assert_eq!(code, 0, "stderr={stderr}");
    let listed: Vec<serde_json::Value> = serde_json::from_str(list_json.trim()).expect("list json");
    let listed_ids: Vec<&str> = listed
        .iter()
        .filter_map(|record| record["id"].as_str())
        .collect();

    assert!(
        registry.extensions().len() >= 7,
        "expected seven shipped skills"
    );
    for ext in registry.extensions() {
        let id = ext.id.as_str();
        assert!(
            listed_ids.contains(&id),
            "list --json missing {id}: {listed_ids:?}"
        );
        assert!(help.contains(id), "global --help missing id {id}: {help}");
        let description = ext
            .description
            .as_deref()
            .map(str::trim)
            .filter(|text| !text.is_empty());
        assert!(
            description.is_some(),
            "shipped skill {id} must have a non-empty description"
        );
        assert!(
            ext.examples
                .iter()
                .any(|example| !example.trim().is_empty()),
            "shipped skill {id} must have at least one examples string"
        );
        if let Some(suffix) = ext
            .match_spec
            .positional_suffix
            .as_ref()
            .map(|token| token.as_str())
        {
            assert!(
                help.contains(suffix),
                "global --help missing suffix {suffix} for {id}"
            );
        }
        if let Some(filename) = ext.match_spec.filename.as_ref().map(|token| token.as_str()) {
            assert!(
                help.contains(filename),
                "global --help missing filename {filename} for {id}"
            );
        }
        if let Some(prefix) = &ext.match_spec.argv_prefix {
            let prefix_s = prefix
                .iter()
                .map(|token| token.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            assert!(
                help.contains(&prefix_s),
                "global --help missing prefix {prefix_s} for {id}"
            );
        }
    }
}

#[test]
fn compose_render_shipped_preexec_args_use_output_and_env_prefix() {
    let shipped: serde_json::Value =
        serde_json::from_str(SHIPPED_EXTENSIONS_JSON).expect("shipped json");
    let compose = shipped["extensions"]
        .as_array()
        .expect("extensions")
        .iter()
        .find(|ext| ext["id"] == "compose-render")
        .expect("compose-render");
    let args: Vec<&str> = compose["preexec"]["args"]
        .as_array()
        .expect("args")
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect();
    assert!(args.contains(&"--output"), "{args:?}");
    assert!(!args.contains(&"--out"), "{args:?}");
    assert!(!args.contains(&"--env"), "{args:?}");
    assert!(
        args.iter().any(|token| token.contains("env-prefix")),
        "{args:?}"
    );
    assert!(!args.contains(&"--format"), "{args:?}");
    assert!(!args.contains(&"html"), "{args:?}");
}

#[test]
fn all_shipped_skills_have_description_and_examples() {
    let registry = shipped_registry();
    for ext in registry.extensions() {
        assert!(
            ext.description
                .as_deref()
                .is_some_and(|text| !text.trim().is_empty()),
            "{}",
            ext.id
        );
        assert!(
            !ext.examples.is_empty() && ext.examples.iter().all(|ex| !ex.trim().is_empty()),
            "{}",
            ext.id
        );
    }
}
