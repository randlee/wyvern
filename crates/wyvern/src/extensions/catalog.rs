//! Skill catalog: [`SkillRecord`], list/show JSON, and the sole text formatter.
//!
//! `wyvern --help`, extension prefix `--help`, `extensions list`, and
//! `extensions show` all call [`format_skill_card`] after
//! [`build_skill_record`]. There is no second formatter.

use std::collections::BTreeSet;

use serde::Serialize;
use serde_json::Value;

use super::{
    match_kind_summary, ArgName, BinaryName, ExtensionDef, ExtensionId, ExtensionRegistry,
    MatchToken, PreexecSpec, RequiresProbe, SkillSource,
};

/// One declared `{arg:name}` / `{arg:name:repeat}` flag.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SkillArg {
    /// Flag name without leading dashes.
    pub name: ArgName,
    /// `true` when the template uses `{arg:name}` (missing is an error).
    pub required: bool,
    /// `true` when the template uses `{arg:name:repeat}`.
    pub repeat: bool,
}

/// One `preexec.requires` binary and its current PATH availability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SkillRequire {
    /// Bare binary name from the registry.
    pub binary: BinaryName,
    /// Result of [`RequiresProbe::binary_on_path`] at build time.
    pub available: bool,
}

/// One catalog / help record for a resolved extension (REQ-0132).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SkillRecord {
    /// Extension id.
    pub id: ExtensionId,
    /// Human match DSL (`prefix: compose render`, `prefix+suffix: md .csv`).
    pub match_kind: String,
    /// Copy-paste invocation pattern, including declared flags.
    pub invocation: String,
    /// Required binaries and whether each is on `PATH`.
    pub requires: Vec<SkillRequire>,
    /// Declared `{arg:*}` flags.
    pub args: Vec<SkillArg>,
    /// Expand command `type` from inline JSON or `command_from_file` contents.
    pub expands_to: String,
    /// One-line agent-facing summary from the registry, if present.
    pub description: Option<String>,
    /// Copy-paste example lines.
    pub examples: Vec<String>,
    /// Parent extension id when `extends` is set; otherwise `null` on the wire.
    pub extends: Option<ExtensionId>,
    /// `shipped` defaults or `project` `.wyvern/extensions.json`.
    pub source: SkillSource,
}

/// Build a help-oriented [`SkillRecord`] from a resolved extension.
///
/// `probe` is evaluated at call time (not cached). Help still builds a card
/// when required binaries are missing.
#[must_use]
pub fn build_skill_record(ext: &ExtensionDef, probe: &dyn RequiresProbe) -> SkillRecord {
    let args = declared_skill_args(ext);
    let invocation = invocation_line(ext, &args);
    let examples = if ext.examples.is_empty() {
        vec![example_line(ext, &invocation, &args)]
    } else {
        ext.examples.clone()
    };
    SkillRecord {
        id: ext.id.clone(),
        match_kind: match_kind_summary(&ext.match_spec),
        invocation,
        requires: ext
            .requires()
            .iter()
            .map(|binary| SkillRequire {
                binary: binary.clone(),
                available: probe.binary_on_path(binary.as_str()),
            })
            .collect(),
        args,
        expands_to: expands_to(ext),
        description: ext
            .description
            .as_ref()
            .map(|text| text.trim().to_string())
            .filter(|text| !text.is_empty()),
        examples,
        extends: ext.extends.clone(),
        source: ext.source,
    }
}

/// Recovery `--help` using the invocation prefix, not the extension id.
///
/// Prefix skills: `wyvern compose render --help`. Suffix/filename skills:
/// `wyvern extensions show <id>` (path `--help` also works at match time).
#[must_use]
pub fn skill_help_command(ext: &ExtensionDef) -> String {
    if let Some(prefix) = &ext.match_spec.argv_prefix {
        if !prefix.is_empty() {
            return format!("wyvern {} --help", join_prefix(prefix));
        }
    }
    format!("wyvern extensions show {}", ext.id)
}

/// Build a [`SkillRecord`] for every merged extension, in registry order.
#[must_use]
pub fn build_skill_records(
    registry: &ExtensionRegistry,
    probe: &dyn RequiresProbe,
) -> Vec<SkillRecord> {
    registry
        .extensions()
        .iter()
        .map(|ext| build_skill_record(ext, probe))
        .collect()
}

/// Format one skill as the single help / list / show text card.
///
/// g.1 `--help` and g.3 `list` / `show` must call this function. There is no
/// second formatter.
#[must_use]
pub fn format_skill_card(record: &SkillRecord) -> String {
    let mut out = String::new();
    out.push_str(record.id.as_str());
    out.push('\n');
    out.push_str(&record.match_kind);
    out.push('\n');
    if let Some(description) = &record.description {
        out.push_str(description);
        out.push('\n');
    }
    out.push_str("Usage: ");
    out.push_str(&record.invocation);
    out.push('\n');
    out.push_str("Requires: ");
    if record.requires.is_empty() {
        out.push_str("(none)");
    } else {
        out.push_str(
            &record
                .requires
                .iter()
                .map(|req| {
                    let status = if req.available {
                        "available"
                    } else {
                        "missing"
                    };
                    format!("{} [{status}]", req.binary)
                })
                .collect::<Vec<_>>()
                .join(", "),
        );
    }
    out.push('\n');
    out.push_str("Expands to: ");
    out.push_str(&record.expands_to);
    out.push('\n');
    if let Some(parent) = &record.extends {
        out.push_str("Extends: ");
        out.push_str(parent.as_str());
        out.push_str(" (alias)\n");
    }
    out.push_str("Example: ");
    if let Some(example) = record.examples.first() {
        out.push_str(example);
        for extra in record.examples.iter().skip(1) {
            out.push('\n');
            out.push_str("         ");
            out.push_str(extra);
        }
    } else {
        out.push_str(&record.invocation);
    }
    out.push('\n');
    out
}

fn expands_to(ext: &ExtensionDef) -> String {
    if let Some(ty) = ext
        .expand
        .as_ref()
        .and_then(|spec| spec.command.as_ref())
        .and_then(|command| command.get("type"))
        .and_then(Value::as_str)
        .filter(|ty| !ty.is_empty())
    {
        return ty.to_string();
    }
    if let Some(path_tmpl) = ext
        .expand
        .as_ref()
        .and_then(|spec| spec.command_from_file.as_deref())
    {
        if let Some(ty) = type_from_command_from_file(path_tmpl) {
            return ty;
        }
    }
    "wizard".to_string()
}

/// Read `type` from resolvable `command_from_file` JSON, or the emitted filename.
///
/// `{tmpdir}/report-command.json` is written by report preexec (h.2) and is
/// always `type: "report"` — catalog listing cannot wait for preexec.
fn type_from_command_from_file(path_tmpl: &str) -> Option<String> {
    if let Some(path) = resolve_static_command_path(path_tmpl) {
        if let Some(ty) = read_command_type(&path) {
            return Some(ty);
        }
    }
    let name = path_tmpl.rsplit(['/', '\\']).next().unwrap_or(path_tmpl);
    (name == "report-command.json").then(|| "report".to_string())
}

fn resolve_static_command_path(path_tmpl: &str) -> Option<std::path::PathBuf> {
    const SHARE: &str = "{wyvern_share}";
    let open_braces = path_tmpl.chars().filter(|ch| *ch == '{').count();
    if let Some(rest) = path_tmpl.strip_prefix(SHARE) {
        if open_braces > 1 {
            return None;
        }
        let mut path = super::resolve_wyvern_share();
        let rest = rest.trim_start_matches('/');
        if !rest.is_empty() {
            path.push(rest);
        }
        return path.is_file().then_some(path);
    }
    if open_braces > 0 {
        return None;
    }
    let path = std::path::PathBuf::from(path_tmpl);
    path.is_file().then_some(path)
}

fn read_command_type(path: &std::path::Path) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    let value: Value = serde_json::from_str(&text).ok()?;
    value
        .get("type")
        .and_then(Value::as_str)
        .filter(|ty| !ty.is_empty())
        .map(ToOwned::to_owned)
}

fn invocation_line(ext: &ExtensionDef, args: &[SkillArg]) -> String {
    let mut parts = vec!["wyvern".to_string()];
    let spec = &ext.match_spec;
    if let Some(prefix) = &spec.argv_prefix {
        for token in prefix {
            parts.push(token.as_str().to_string());
        }
        if let Some(suffix) = &spec.arg_suffix {
            parts.push(format!("<file{}>", suffix.as_str()));
        }
        for arg in args {
            parts.push(format_arg(arg));
        }
        return parts.join(" ");
    }
    if let Some(filename) = &spec.filename {
        parts.push(format!("path/to/{}", filename.as_str()));
        return parts.join(" ");
    }
    if let Some(suffix) = &spec.positional_suffix {
        parts.push(format!("file{}", suffix.as_str()));
        return parts.join(" ");
    }
    parts.join(" ")
}

fn example_line(ext: &ExtensionDef, invocation: &str, args: &[SkillArg]) -> String {
    let spec = &ext.match_spec;
    if let Some(prefix) = &spec.argv_prefix {
        let prefix_s = join_prefix(prefix);
        if let Some(suffix) = &spec.arg_suffix {
            return format!("wyvern {prefix_s} data{}", suffix.as_str());
        }
        if !args.is_empty() {
            return format!("wyvern {prefix_s} {}", example_args(args));
        }
        return format!("wyvern {prefix_s}");
    }
    invocation.to_string()
}

fn join_prefix(prefix: &[MatchToken]) -> String {
    prefix
        .iter()
        .map(MatchToken::as_str)
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_arg(arg: &SkillArg) -> String {
    match (arg.name.as_str(), arg.required, arg.repeat) {
        ("root", true, _) => "--root <DIR>".into(),
        ("file", true, _) => "--file <FILE>".into(),
        ("var", _, true) => "[--var k=v]".into(),
        ("var-file", _, true) => "[--var-file vars.json]".into(),
        ("env-prefix", _, true) => "[--env-prefix PREFIX]".into(),
        (name, true, _) => format!("--{name} <{}>", placeholder(name)),
        (name, false, true) => format!("[--{name} …]"),
        (name, false, false) => format!("[--{name} <{}>]", placeholder(name)),
    }
}

fn example_args(args: &[SkillArg]) -> String {
    args.iter()
        .map(|arg| match (arg.name.as_str(), arg.required, arg.repeat) {
            ("root", true, _) => "--root DIR".into(),
            ("file", true, _) => "--file FILE.j2".into(),
            ("var", _, true) => "[--var k=v]".into(),
            ("var-file", _, true) => "[--var-file vars.json]".into(),
            ("env-prefix", _, true) => "[--env-prefix PREFIX]".into(),
            (name, true, _) => format!("--{name} {}", placeholder(name)),
            (name, false, true) => format!("[--{name} …]"),
            (name, false, false) => format!("[--{name} {}]", placeholder(name)),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn placeholder(name: &str) -> String {
    name.replace('-', "_").to_ascii_uppercase()
}

pub(crate) fn declared_skill_args(ext: &ExtensionDef) -> Vec<SkillArg> {
    let mut vars = Vec::new();
    let mut seen = BTreeSet::new();
    if let Some(PreexecSpec { cmd, args, .. }) = &ext.preexec {
        collect_template_vars(cmd, &mut vars, &mut seen);
        for arg in args {
            collect_template_vars(arg, &mut vars, &mut seen);
        }
    }
    if let Some(exp) = &ext.expand {
        if let Some(cmd) = &exp.command {
            collect_value_vars(cmd, &mut vars, &mut seen);
        }
        if let Some(path) = &exp.command_from_file {
            collect_template_vars(path, &mut vars, &mut seen);
        }
        if let Some(ui) = exp.host.as_ref().and_then(|h| h.ui_root.as_ref()) {
            collect_template_vars(ui, &mut vars, &mut seen);
        }
    }
    let mut ordered: Vec<SkillArg> = Vec::new();
    for var in vars {
        let Some(rest) = var.strip_prefix("arg:") else {
            continue;
        };
        let (name, repeat) = match rest.strip_suffix(":repeat") {
            Some(name) => (name, true),
            None => (rest, false),
        };
        if let Some(existing) = ordered.iter_mut().find(|arg| arg.name.as_str() == name) {
            if repeat {
                existing.repeat = true;
            } else {
                existing.required = true;
            }
            continue;
        }
        let Some(arg_name) = ArgName::new(name) else {
            continue;
        };
        ordered.push(SkillArg {
            name: arg_name,
            required: !repeat,
            repeat,
        });
    }
    ordered
}

fn collect_template_vars(template: &str, into: &mut Vec<String>, seen: &mut BTreeSet<String>) {
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        let after = &rest[start + 1..];
        let Some(end) = after.find('}') else {
            break;
        };
        let name = after[..end].to_string();
        if seen.insert(name.clone()) {
            into.push(name);
        }
        rest = &after[end + 1..];
    }
}

fn collect_value_vars(value: &Value, into: &mut Vec<String>, seen: &mut BTreeSet<String>) {
    match value {
        Value::String(s) => collect_template_vars(s, into, seen),
        Value::Array(items) => {
            for item in items {
                collect_value_vars(item, into, seen);
            }
        }
        Value::Object(map) => {
            for v in map.values() {
                collect_value_vars(v, into, seen);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extensions::{
        ExtensionRegistry, RequiresProbe, SkillSource, SHIPPED_EXTENSIONS_JSON,
    };

    struct Absent;

    impl RequiresProbe for Absent {
        fn binary_on_path(&self, _name: &str) -> bool {
            false
        }
    }

    #[test]
    fn compose_card_lists_flags_requires_and_example() {
        let registry = ExtensionRegistry::from_json_str(SHIPPED_EXTENSIONS_JSON).expect("shipped");
        let ext = registry
            .extensions()
            .iter()
            .find(|e| e.id.as_str() == "compose-render")
            .expect("compose-render");
        let record = build_skill_record(ext, &Absent);
        assert_eq!(record.id.to_string(), "compose-render");
        assert_eq!(record.source, SkillSource::Shipped);
        assert_eq!(skill_help_command(ext), "wyvern compose render --help");
        assert!(record.args.iter().any(|arg| arg.name.as_str() == "root"));
        assert!(!record.requires.iter().any(|r| r.available));
        let card = format_skill_card(&record);
        let root_at = card.find("--root").expect("root");
        let file_at = card.find("--file").expect("file");
        assert!(root_at < file_at, "{card}");
        assert!(card.contains("--env-prefix"), "{card}");
        assert!(card.contains("Requires:"), "{card}");
        assert!(card.contains("sc-compose"), "{card}");
        assert!(card.contains("Example:"), "{card}");
    }

    #[test]
    fn md_card_does_not_require_csv_path() {
        let registry = ExtensionRegistry::from_json_str(SHIPPED_EXTENSIONS_JSON).expect("shipped");
        let ext = registry
            .extensions()
            .iter()
            .find(|e| e.id.as_str() == "csv-md")
            .expect("csv-md");
        let card = format_skill_card(&build_skill_record(ext, &Absent));
        assert_eq!(skill_help_command(ext), "wyvern md --help");
        assert!(card.contains("wyvern md"), "{card}");
        assert!(card.contains("Requires:"), "{card}");
        assert!(card.contains("Example:"), "{card}");
    }

    #[test]
    fn compose_render_shipped_preexec_uses_output_and_env_prefix() {
        let shipped: Value = serde_json::from_str(SHIPPED_EXTENSIONS_JSON).expect("json");
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
            .filter_map(Value::as_str)
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
    fn registry_accepts_missing_description_and_examples() {
        let json = r#"{
          "version": 1,
          "extensions": [{
            "id": "plain",
            "match": { "positional_suffix": ".md" },
            "expand": { "command": { "type": "markdown", "file": "{path}" } }
          }]
        }"#;
        let registry = ExtensionRegistry::from_json_str(json).expect("parse");
        let record = build_skill_record(&registry.extensions()[0], &Absent);
        assert!(record.description.is_none());
        assert_eq!(record.extends, None);
        assert_eq!(record.examples.len(), 1);
        assert_eq!(record.source, SkillSource::Shipped);
        assert_eq!(
            skill_help_command(&registry.extensions()[0]),
            "wyvern extensions show plain"
        );
    }

    #[test]
    fn declared_skill_args_skips_empty_template_names() {
        let json = r#"{
          "version": 1,
          "extensions": [{
            "id": "empty-arg",
            "match": { "positional_suffix": ".md" },
            "preexec": { "cmd": "true", "args": ["{arg:}", "{arg:  }", "{arg:root}"] },
            "expand": { "command": { "type": "markdown", "file": "{path}" } }
          }]
        }"#;
        let registry = ExtensionRegistry::from_json_str(json).expect("parse");
        let args = declared_skill_args(&registry.extensions()[0]);
        assert_eq!(args.len(), 1, "{args:?}");
        assert_eq!(args[0].name.as_str(), "root");
    }

    #[test]
    fn expands_to_reads_type_from_command_json_file() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("emitted.json");
        std::fs::write(
            &path,
            r#"{"type":"report","title":"t","page":"pages/view.xhtml"}"#,
        )
        .expect("write");
        let json = format!(
            r#"{{
          "version": 1,
          "extensions": [{{
            "id": "from-file",
            "match": {{ "argv_prefix": ["from-file"] }},
            "expand": {{ "command_from_file": "{}" }}
          }}]
        }}"#,
            path.display()
        );
        let registry = ExtensionRegistry::from_json_str(&json).expect("parse");
        let record = build_skill_record(&registry.extensions()[0], &Absent);
        assert_eq!(record.expands_to, "report");
    }

    #[test]
    fn expands_to_report_command_json_template_is_report() {
        let json = r#"{
          "version": 1,
          "extensions": [{
            "id": "report-from-file",
            "match": { "argv_prefix": ["report-xhtml"], "arg_suffix": ".json" },
            "expand": { "command_from_file": "{tmpdir}/report-command.json" }
          }]
        }"#;
        let registry = ExtensionRegistry::from_json_str(json).expect("parse");
        let record = build_skill_record(&registry.extensions()[0], &Absent);
        assert_eq!(record.expands_to, "report");
    }

    #[test]
    fn shipped_report_xhtml_expands_to_report() {
        let registry = ExtensionRegistry::from_json_str(SHIPPED_EXTENSIONS_JSON).expect("shipped");
        let ext = registry
            .extensions()
            .iter()
            .find(|e| e.id.as_str() == "report-xhtml")
            .expect("report-xhtml");
        let record = build_skill_record(ext, &Absent);
        assert_eq!(record.expands_to, "report");
        assert_ne!(record.expands_to, "wizard");
    }

    #[test]
    fn skill_record_includes_catalog_fields() {
        let registry = ExtensionRegistry::from_json_str(SHIPPED_EXTENSIONS_JSON).expect("shipped");
        let records = build_skill_records(&registry, &Absent);
        assert!(records.len() >= 7, "{}", records.len());
        let alias = records
            .iter()
            .find(|record| record.id.as_str() == "csv-table-alias")
            .expect("csv-table-alias");
        assert_eq!(
            alias.extends.as_ref().map(ExtensionId::as_str),
            Some("csv-suffix")
        );
        assert!(alias.description.as_ref().is_some_and(|d| !d.is_empty()));
        assert!(!alias.examples.is_empty());
        let card = format_skill_card(alias);
        assert!(card.contains("Extends: csv-suffix (alias)"), "{card}");
        assert!(card.contains("[missing]"), "{card}");
    }
}
