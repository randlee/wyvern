//! Near-miss classification and structured stderr JSON (REQ-0136).
//!
//! Called from `main` after [`super::ExtensionRegistry::match_with_diagnostics`]
//! returns no match and before [`crate::load_command_input`]. Do not parse
//! path-like tokens as inline JSON.

use wyvern_schema::{ErrorCode, StderrError};

use crate::error::EmitError;

use super::{
    build_skill_record, ends_with_suffix, format_skill_card, BinaryName, ExtensionId,
    ExtensionRegistry, PathRequiresProbe,
};

/// Extension that would have matched argv but was skipped for `requires`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkippedExtension {
    /// Extension id that was skipped.
    pub id: ExtensionId,
    /// Required binaries that were not on `PATH`.
    pub missing: Vec<BinaryName>,
}

/// Result of walking the registry with skip diagnostics.
#[derive(Debug)]
pub struct MatchOutcome<'a> {
    /// First extension that matched argv and had all `requires` present.
    pub matched: Option<super::ExtensionMatch<'a>>,
    /// Spec matches skipped because required binaries were absent.
    pub skipped: Vec<SkippedExtension>,
}

/// Why remainder argv did not match an extension (REQ-0136).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NearMissKind {
    /// Path or token is not a known suffix, filename, or prefix.
    UnknownInput {
        /// Offending argv token.
        token: String,
    },
    /// First prefix tokens matched; later prefix tokens are missing.
    IncompletePrefix {
        /// Extension that owns the full prefix.
        extension_id: ExtensionId,
        /// Remaining argv to type (for example `compose render`).
        hint: String,
    },
    /// Full prefix matched; required suffix path is absent or wrong.
    BarePrefix {
        /// Extension that owns the prefix.
        extension_id: ExtensionId,
        /// Invocation line including the missing suffix placeholder.
        usage: String,
    },
    /// Path would match, but every candidate was skipped for `requires`.
    SkippedRequires {
        /// Path token that would have matched.
        path: String,
        /// Skipped candidates and their missing binaries.
        skipped: Vec<SkippedExtension>,
    },
}

impl NearMissKind {
    /// Process exit code for this near-miss (`2` parse / `4` validation).
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::UnknownInput { .. } => ErrorCode::ParseError.exit_code(),
            Self::IncompletePrefix { .. }
            | Self::BarePrefix { .. }
            | Self::SkippedRequires { .. } => ErrorCode::ValidationError.exit_code(),
        }
    }
}

/// Classify a no-match remainder using the Phase G near-miss table.
///
/// Returns `None` for inline JSON (`{` / `[`) and `.json` file fallthrough so
/// [`crate::load_command_input`] can run. Path-like unknown tokens become
/// [`NearMissKind::UnknownInput`] instead of a JSON parse error.
#[must_use]
pub fn classify_near_miss(
    registry: &ExtensionRegistry,
    argv: &[String],
    skipped: &[SkippedExtension],
) -> Option<NearMissKind> {
    if argv.is_empty() {
        return None;
    }
    if argv.len() == 1 {
        let token = argv[0].as_str();
        if token.starts_with('{') || token.starts_with('[') {
            return None;
        }
        if token.starts_with('-') {
            return None;
        }
        if is_json_command_file(token) {
            return None;
        }
    }
    if !skipped.is_empty() {
        let path = argv
            .iter()
            .find(|token| looks_path_like(token))
            .cloned()
            .unwrap_or_else(|| argv[0].clone());
        return Some(NearMissKind::SkippedRequires {
            path,
            skipped: skipped.to_vec(),
        });
    }
    if let Some(kind) = find_bare_prefix(registry, argv) {
        return Some(kind);
    }
    if let Some(kind) = find_incomplete_prefix(registry, argv) {
        return Some(kind);
    }
    Some(NearMissKind::UnknownInput {
        token: argv[0].clone(),
    })
}

/// Serialize a near-miss as the existing [`StderrError`] envelope.
///
/// # Errors
///
/// Returns [`EmitError::Serialize`] when the envelope cannot be serialized.
pub fn emit_near_miss(kind: &NearMissKind) -> Result<String, EmitError> {
    let (code, message, cause, recovery) = match kind {
        NearMissKind::UnknownInput { token } => (
            ErrorCode::ParseError,
            format!("unknown input '{token}'"),
            format!("No shipped extension matches '{token}'"),
            vec![
                "Use a supported suffix such as .md, .html, .csv, or wizard.json".into(),
                "Or a prefix such as md <file.csv>, table <file.csv>, or compose render".into(),
                "Run wyvern --help to list skills".into(),
                "Run wyvern extensions list".into(),
            ],
        ),
        NearMissKind::IncompletePrefix { extension_id, hint } => (
            ErrorCode::ValidationError,
            format!("incomplete prefix for '{extension_id}'"),
            format!("'{extension_id}' expects `{hint}`"),
            vec![
                format!("Continue with: wyvern {hint}"),
                format!("Run wyvern {hint} --help"),
                "Run wyvern --help to list skills".into(),
            ],
        ),
        NearMissKind::BarePrefix {
            extension_id,
            usage,
        } => (
            ErrorCode::ValidationError,
            format!("extension '{extension_id}' requires a matching path"),
            format!("Usage: {usage}"),
            vec![
                format!("Pass a path as in: {usage}"),
                format!("Run wyvern {} --help", prefix_from_usage(usage)),
                "Run wyvern --help to list skills".into(),
            ],
        ),
        NearMissKind::SkippedRequires { path, skipped } => {
            let (id_summary, missing) = skipped_requires_summary(skipped);
            let mut recovery = vec![format!("Install {missing} and retry")];
            for skipped_ext in skipped {
                let example = skill_example_line(skipped_ext.id.as_str())
                    .unwrap_or_else(|| format!("wyvern {path}"));
                recovery.push(format!("Example ({}): {example}", skipped_ext.id));
            }
            recovery.push("Run wyvern extensions list to see requires".into());
            recovery.push("Run wyvern --help to list skills".into());
            (
                ErrorCode::ValidationError,
                format!("extension(s) '{id_summary}' skipped; missing {missing}"),
                format!(
                    "'{path}' matched skipped extension(s) but required binaries are not on PATH"
                ),
                recovery,
            )
        }
    };
    let mut envelope = StderrError::new(code, message)
        .cause(cause)
        .docs("docs/wyvern/requirements.md (REQ-0136)");
    for step in recovery {
        envelope = envelope.recovery(step);
    }
    envelope.to_json_string().map_err(EmitError::Serialize)
}

fn is_json_command_file(token: &str) -> bool {
    std::path::Path::new(token)
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
}

fn looks_path_like(token: &str) -> bool {
    token.contains('/') || token.contains('\\') || std::path::Path::new(token).extension().is_some()
}

fn find_bare_prefix(registry: &ExtensionRegistry, argv: &[String]) -> Option<NearMissKind> {
    let mut best: Option<(&super::ExtensionDef, usize)> = None;
    for ext in registry.extensions() {
        let spec = &ext.match_spec;
        let Some(prefix) = &spec.argv_prefix else {
            continue;
        };
        let Some(suffix) = &spec.arg_suffix else {
            continue;
        };
        if !prefix_tokens_match(prefix, argv) {
            continue;
        }
        let rest = &argv[prefix.len()..];
        if rest
            .iter()
            .any(|token| ends_with_suffix(token, suffix.as_str()))
        {
            continue;
        }
        if best.is_none_or(|(_, len)| prefix.len() > len) {
            best = Some((ext, prefix.len()));
        }
    }
    best.map(|(ext, _)| {
        let record = build_skill_record(ext, &PathRequiresProbe);
        NearMissKind::BarePrefix {
            extension_id: ext.id.clone(),
            usage: record.invocation,
        }
    })
}

fn find_incomplete_prefix(registry: &ExtensionRegistry, argv: &[String]) -> Option<NearMissKind> {
    let mut best: Option<(&super::ExtensionDef, usize)> = None;
    for ext in registry.extensions() {
        let Some(prefix) = &ext.match_spec.argv_prefix else {
            continue;
        };
        if prefix.is_empty() || argv.is_empty() || argv.len() >= prefix.len() {
            continue;
        }
        if !prefix
            .iter()
            .zip(argv.iter())
            .all(|(expected, got)| expected.as_str() == got)
        {
            continue;
        }
        if best.is_none_or(|(_, len)| prefix.len() > len) {
            best = Some((ext, prefix.len()));
        }
    }
    best.map(|(ext, _)| {
        let hint = ext
            .match_spec
            .argv_prefix
            .as_ref()
            .map(|prefix| {
                prefix
                    .iter()
                    .map(super::MatchToken::as_str)
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_default();
        NearMissKind::IncompletePrefix {
            extension_id: ext.id.clone(),
            hint,
        }
    })
}

fn prefix_tokens_match(prefix: &[super::MatchToken], argv: &[String]) -> bool {
    argv.len() >= prefix.len()
        && prefix
            .iter()
            .zip(argv.iter())
            .all(|(expected, got)| expected.as_str() == got)
}

fn prefix_from_usage(usage: &str) -> String {
    usage
        .strip_prefix("wyvern ")
        .unwrap_or(usage)
        .split_whitespace()
        .take_while(|part| {
            !part.starts_with('<') && !part.starts_with('[') && !part.starts_with('-')
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Bound how many skipped ids appear in the human/JSON summary.
const MAX_SKIPPED_SUMMARY: usize = 4;

fn skipped_requires_summary(skipped: &[SkippedExtension]) -> (String, String) {
    let shown = skipped.len().min(MAX_SKIPPED_SUMMARY);
    let ids = skipped[..shown]
        .iter()
        .map(|s| s.id.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let id_summary = if skipped.len() > MAX_SKIPPED_SUMMARY {
        format!("{ids} (+{} more)", skipped.len() - MAX_SKIPPED_SUMMARY)
    } else if ids.is_empty() {
        "extension".into()
    } else {
        ids
    };
    let mut missing = Vec::new();
    for skipped in skipped {
        for bin in &skipped.missing {
            let name = bin.as_str();
            if !missing.iter().any(|seen: &String| seen == name) {
                missing.push(name.to_string());
            }
        }
    }
    let missing = if missing.is_empty() {
        "required binaries".into()
    } else {
        missing.join(", ")
    };
    (id_summary, missing)
}

fn skill_example_line(id: &str) -> Option<String> {
    let registry = ExtensionRegistry::from_json_str(super::SHIPPED_EXTENSIONS_JSON).ok()?;
    let ext = registry
        .extensions()
        .iter()
        .find(|ext| ext.id.as_str() == id)?;
    let record = build_skill_record(ext, &PathRequiresProbe);
    let card = format_skill_card(&record);
    card.lines()
        .find_map(|line| line.strip_prefix("Example: "))
        .map(ToOwned::to_owned)
        .or_else(|| record.examples.first().cloned())
        .or(Some(record.invocation))
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

    fn shipped() -> ExtensionRegistry {
        ExtensionRegistry::from_json_str(SHIPPED_EXTENSIONS_JSON).expect("shipped")
    }

    #[test]
    fn unknown_txt_is_parse_error_not_json() {
        let registry = shipped();
        let argv = vec!["notes.txt".into()];
        let outcome = registry.match_with_diagnostics(&argv);
        let kind = classify_near_miss(&registry, &argv, &outcome.skipped).expect("near-miss");
        assert!(matches!(kind, NearMissKind::UnknownInput { .. }));
        let json = emit_near_miss(&kind).expect("emit");
        assert!(json.contains("PARSE_ERROR"), "{json}");
        assert!(json.contains("unknown input"), "{json}");
        assert!(!json.contains("not valid JSON"), "{json}");
        assert_eq!(kind.exit_code(), 2);
    }

    #[test]
    fn inline_json_is_not_a_near_miss() {
        let registry = shipped();
        let argv = vec![r#"{"type":"message"}"#.into()];
        assert!(classify_near_miss(&registry, &argv, &[]).is_none());
    }

    #[test]
    fn json_file_falls_through() {
        let registry = shipped();
        let argv = vec!["cmd.json".into()];
        assert!(classify_near_miss(&registry, &argv, &[]).is_none());
    }

    #[test]
    fn md_bare_prefix_names_csv_md_and_file_csv() {
        let registry = shipped();
        let argv = vec!["md".into()];
        let kind = classify_near_miss(&registry, &argv, &[]).expect("bare");
        match &kind {
            NearMissKind::BarePrefix {
                extension_id,
                usage,
            } => {
                assert_eq!(extension_id.as_str(), "csv-md");
                assert!(usage.contains("<file.csv>"), "{usage}");
            }
            other => panic!("expected BarePrefix, got {other:?}"),
        }
        let json = emit_near_miss(&kind).expect("emit");
        assert!(json.contains("VALIDATION_ERROR"), "{json}");
        assert!(json.contains("<file.csv>"), "{json}");
        assert_eq!(kind.exit_code(), 4);
    }

    #[test]
    fn compose_incomplete_prefix_hints_compose_render() {
        let registry = shipped();
        let argv = vec!["compose".into()];
        let kind = classify_near_miss(&registry, &argv, &[]).expect("incomplete");
        match &kind {
            NearMissKind::IncompletePrefix { extension_id, hint } => {
                assert_eq!(extension_id.as_str(), "compose-render");
                assert_eq!(hint, "compose render");
            }
            other => panic!("expected IncompletePrefix, got {other:?}"),
        }
        let json = emit_near_miss(&kind).expect("emit");
        assert!(json.contains("compose render"), "{json}");
        assert!(json.contains("compose-render"), "{json}");
    }

    #[test]
    fn csv_skipped_requires_names_python3() {
        let registry = shipped();
        let argv = vec!["sample.csv".into()];
        let outcome = registry.match_with_diagnostics_with(&argv, &Absent);
        assert!(outcome.matched.is_none());
        assert!(
            outcome
                .skipped
                .iter()
                .any(|s| s.id == "csv-suffix" && s.missing.iter().any(|b| b == "python3")),
            "{:?}",
            outcome.skipped
        );
        let kind = classify_near_miss(&registry, &argv, &outcome.skipped).expect("skipped");
        let json = emit_near_miss(&kind).expect("emit");
        assert!(json.contains("csv-suffix"), "{json}");
        assert!(json.contains("python3"), "{json}");
        assert!(json.contains("wyvern"), "{json}");
        let _ = format_skill_card(&build_skill_record(
            registry
                .extensions()
                .iter()
                .find(|e| e.id.as_str() == "csv-suffix")
                .expect("csv"),
            &Absent,
        ));
    }

    #[test]
    fn skipped_requires_lists_all_skipped_extensions() {
        let json = r#"{
          "version": 1,
          "extensions": [
            {
              "id": "one-csv",
              "match": { "positional_suffix": ".csv" },
              "preexec": { "cmd": "python3", "requires": ["python3"] },
              "expand": { "command": { "type": "markdown", "file": "{path}" } }
            },
            {
              "id": "two-csv",
              "match": { "positional_suffix": ".csv" },
              "preexec": { "cmd": "ruby", "requires": ["ruby"] },
              "expand": { "command": { "type": "markdown", "file": "{path}" } }
            }
          ]
        }"#;
        let registry = ExtensionRegistry::from_json_str(json).expect("parse");
        let argv = vec!["sample.csv".into()];
        let outcome = registry.match_with_diagnostics_with(&argv, &Absent);
        assert_eq!(outcome.skipped.len(), 2, "{:?}", outcome.skipped);
        let kind = classify_near_miss(&registry, &argv, &outcome.skipped).expect("skipped");
        let json = emit_near_miss(&kind).expect("emit");
        assert!(json.contains("one-csv"), "{json}");
        assert!(json.contains("two-csv"), "{json}");
        assert!(json.contains("python3"), "{json}");
        assert!(json.contains("ruby"), "{json}");
        assert!(json.contains("Example (one-csv):"), "{json}");
        assert!(json.contains("Example (two-csv):"), "{json}");
    }
}
