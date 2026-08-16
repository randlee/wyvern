//! Skill-card catalog stub (g.1).
//!
//! Builds a minimal [`SkillRecord`] for match-time `--help` and formats it
//! with [`format_skill_card`]. g.3 extends the same types for `list` / `show`.

use std::collections::BTreeSet;

use serde_json::Value;

use super::{
    match_kind_summary, ArgName, BinaryName, ExtensionDef, ExtensionId, MatchToken, PreexecSpec,
    RequiresProbe,
};

/// One declared `{arg:name}` / `{arg:name:repeat}` flag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillArg {
    /// Flag name without leading dashes.
    pub name: ArgName,
    /// `true` when the template uses `{arg:name}` (missing is an error).
    pub required: bool,
    /// `true` when the template uses `{arg:name:repeat}`.
    pub repeat: bool,
}

/// One `preexec.requires` binary and its current PATH availability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillRequire {
    /// Bare binary name from the registry.
    pub binary: BinaryName,
    /// Result of [`RequiresProbe::binary_on_path`] at build time.
    pub available: bool,
}

/// Minimal skill record for help (g.3 adds catalog JSON fields).
#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// Expand command `type`, or `"wizard"` when using `command_from_file`.
    pub expands_to: String,
    /// Copy-paste example lines.
    pub examples: Vec<String>,
}

/// Build a help-oriented [`SkillRecord`] from a resolved extension.
///
/// `probe` is evaluated at call time (not cached). Help still builds a card
/// when required binaries are missing.
#[must_use]
pub fn build_skill_record(ext: &ExtensionDef, probe: &dyn RequiresProbe) -> SkillRecord {
    let args = declared_skill_args(ext);
    let invocation = invocation_line(ext, &args);
    let examples = vec![example_line(ext, &invocation, &args)];
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
        examples,
    }
}

/// Format one skill as the single help / list / show text card.
///
/// g.1 `--help` and g.3 `list` / `show` must call this function. There is no
/// second formatter.
#[must_use]
pub fn format_skill_card(record: &SkillRecord) -> String {
    let mut out = String::new();
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
                .map(|req| req.binary.to_string())
                .collect::<Vec<_>>()
                .join(", "),
        );
    }
    out.push('\n');
    out.push_str("Expands to: ");
    out.push_str(&record.expands_to);
    out.push('\n');
    out.push_str("Example: ");
    if let Some(example) = record.examples.first() {
        out.push_str(example);
    } else {
        out.push_str(&record.invocation);
    }
    out.push('\n');
    out
}

fn expands_to(ext: &ExtensionDef) -> String {
    ext.expand
        .as_ref()
        .and_then(|spec| spec.command.as_ref())
        .and_then(|command| command.get("type"))
        .and_then(Value::as_str)
        .unwrap_or("wizard")
        .to_string()
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

fn declared_skill_args(ext: &ExtensionDef) -> Vec<SkillArg> {
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
        ordered.push(SkillArg {
            name: ArgName::new(name),
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
    use crate::extensions::{ExtensionRegistry, RequiresProbe, SHIPPED_EXTENSIONS_JSON};

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
        assert!(card.contains("wyvern md"), "{card}");
        assert!(card.contains("Requires:"), "{card}");
        assert!(card.contains("Example:"), "{card}");
    }
}
