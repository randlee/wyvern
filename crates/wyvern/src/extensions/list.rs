//! `wyvern extensions list` / `show` catalog commands (REQ-0132).

use super::{
    build_skill_record, build_skill_records, format_skill_card, ExtensionError, ExtensionId,
    ExtensionRegistry, PathRequiresProbe, SkillRecord,
};
use crate::error::{BuiltinDomain, EmitError, UsageErrorKind};
use wyvern_schema::{ErrorCode, SerializeError, StderrError};

/// Failure from the `extensions` built-in.
#[derive(Debug)]
pub enum ExtensionsCmdError {
    /// Bad argv.
    Usage {
        /// Discriminated usage class for structured stderr recovery.
        kind: UsageErrorKind,
        /// Plain-text usage.
        message: String,
    },
    /// Registry load or emit failure with stderr JSON.
    Stage {
        /// Stderr JSON.
        stderr: String,
        /// Process exit code.
        exit_code: i32,
    },
    /// Emit-boundary serialize failure.
    Emit(EmitError),
}

/// Usage text for `wyvern extensions --help` / `-h`.
#[must_use]
pub fn extensions_usage_message() -> String {
    concat!(
        "Usage: wyvern extensions [list] [--json]\n",
        "       wyvern extensions show <id> [--json]\n",
        "       wyvern extensions --help\n",
        "\n",
        "Commands:\n",
        "  list         List shipped and project CLI extensions\n",
        "  show <id>    Print one skill (text or --json object)\n",
        "\n",
        "Options:\n",
        "  --json       Print SkillRecord JSON (array for list, object for show)\n",
        "\n",
        "Warning: `.wyvern/extensions.json` is trusted preexec; review it before running wyvern.\n",
        "See also: wyvern --help\n",
    )
    .to_string()
}

/// Run `wyvern extensions …`; returns stdout text on success.
///
/// # Errors
///
/// Returns usage text or structured stderr for invalid argv / registry load.
pub fn run_extensions_command(args: &[String]) -> Result<String, ExtensionsCmdError> {
    if args
        .first()
        .is_some_and(|token| token == "--help" || token == "-h")
    {
        return Ok(extensions_usage_message());
    }
    match args.first().map(String::as_str) {
        None => run_list(args),
        Some("list") => run_list(&args[1..]),
        Some("show") => run_show(&args[1..]),
        Some("--json") => run_list(args),
        Some(other) if other.starts_with('-') => Err(unknown_flag(other)),
        Some(other) => Err(ExtensionsCmdError::Usage {
            kind: UsageErrorKind::UnknownSubcommand {
                domain: BuiltinDomain::Extensions,
                token: other.to_string(),
            },
            message: format!(
                "unknown extensions subcommand '{other}'\n{}",
                extensions_usage_message()
            ),
        }),
    }
}

fn run_list(args: &[String]) -> Result<String, ExtensionsCmdError> {
    if wants_help(args) {
        return Ok(extensions_usage_message());
    }
    let json = parse_list_flags(args)?;
    let registry = ExtensionRegistry::load_default().map_err(map_ext)?;
    let records = build_skill_records(&registry, &PathRequiresProbe);
    if json {
        serialize_records_json(&records)
    } else {
        Ok(format_skill_cards(&records))
    }
}

fn run_show(args: &[String]) -> Result<String, ExtensionsCmdError> {
    if wants_help(args) {
        return Ok(extensions_usage_message());
    }
    let (id, json) = parse_show_args(args)?;
    let registry = ExtensionRegistry::load_default().map_err(map_ext)?;
    let Some(ext) = registry.extensions().iter().find(|ext| ext.id == id) else {
        return Err(unknown_id(id.as_str()));
    };
    let record = build_skill_record(ext, &PathRequiresProbe);
    if json {
        serialize_record_json(&record)
    } else {
        Ok(format_skill_card(&record))
    }
}

fn wants_help(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "--help" || arg == "-h")
}

fn parse_list_flags(args: &[String]) -> Result<bool, ExtensionsCmdError> {
    let mut json = false;
    for arg in args {
        match arg.as_str() {
            "--json" => json = true,
            other => return Err(unknown_flag(other)),
        }
    }
    Ok(json)
}

fn parse_show_args(args: &[String]) -> Result<(ExtensionId, bool), ExtensionsCmdError> {
    let mut json = false;
    let mut id = None;
    for arg in args {
        match arg.as_str() {
            "--json" => json = true,
            other if other.starts_with('-') => return Err(unknown_flag(other)),
            other if id.is_none() => {
                id = Some(ExtensionId::try_from(other.to_string()).map_err(|_| missing_id())?);
            }
            other => return Err(unknown_flag(other)),
        }
    }
    let Some(id) = id else {
        return Err(missing_id());
    };
    Ok((id, json))
}

fn missing_id() -> ExtensionsCmdError {
    ExtensionsCmdError::Usage {
        kind: UsageErrorKind::MissingExtensionId,
        message: format!(
            "extensions show requires an extension id\n{}",
            extensions_usage_message()
        ),
    }
}

/// Format each extension as a [`format_skill_card`] block.
#[must_use]
pub fn format_extensions_list(registry: &ExtensionRegistry) -> String {
    format_skill_cards(&build_skill_records(registry, &PathRequiresProbe))
}

fn format_skill_cards(records: &[SkillRecord]) -> String {
    records
        .iter()
        .map(format_skill_card)
        .collect::<Vec<_>>()
        .join("\n")
}

fn serialize_records_json(records: &[SkillRecord]) -> Result<String, ExtensionsCmdError> {
    match serde_json::to_string_pretty(records) {
        Ok(mut text) => {
            if !text.ends_with('\n') {
                text.push('\n');
            }
            Ok(text)
        }
        Err(err) => Err(ExtensionsCmdError::Emit(EmitError::Serialize(
            SerializeError {
                message: err.to_string(),
            },
        ))),
    }
}

fn serialize_record_json(record: &SkillRecord) -> Result<String, ExtensionsCmdError> {
    match serde_json::to_string_pretty(record) {
        Ok(mut text) => {
            if !text.ends_with('\n') {
                text.push('\n');
            }
            Ok(text)
        }
        Err(err) => Err(ExtensionsCmdError::Emit(EmitError::Serialize(
            SerializeError {
                message: err.to_string(),
            },
        ))),
    }
}

fn unknown_flag(flag: &str) -> ExtensionsCmdError {
    match StderrError::new(ErrorCode::ValidationError, format!("unknown flag '{flag}'"))
        .cause("extensions list/show accept only --json")
        .recovery("Run wyvern extensions list")
        .recovery("Run wyvern extensions list --json")
        .recovery("Run wyvern extensions show <id>")
        .recovery("Run wyvern extensions --help")
        .docs("docs/wyvern/requirements.md (REQ-0132)")
        .to_json_string()
    {
        Ok(stderr) => ExtensionsCmdError::Stage {
            stderr,
            exit_code: ErrorCode::ValidationError.exit_code(),
        },
        Err(err) => ExtensionsCmdError::Emit(EmitError::Serialize(err)),
    }
}

fn unknown_id(id: &str) -> ExtensionsCmdError {
    match StderrError::new(
        ErrorCode::ValidationError,
        format!("unknown extension id '{id}'"),
    )
    .cause("No shipped or project extension has that id")
    .recovery("Run wyvern extensions list")
    .recovery("Run wyvern extensions list --json")
    .recovery("Run wyvern extensions --help")
    .docs("docs/wyvern/requirements.md (REQ-0132)")
    .to_json_string()
    {
        Ok(stderr) => ExtensionsCmdError::Stage {
            stderr,
            exit_code: ErrorCode::ValidationError.exit_code(),
        },
        Err(err) => ExtensionsCmdError::Emit(EmitError::Serialize(err)),
    }
}

fn map_ext(err: ExtensionError) -> ExtensionsCmdError {
    match crate::error::emit_extension_error(&err) {
        Ok(stderr) => ExtensionsCmdError::Stage {
            stderr,
            exit_code: err.exit_code(),
        },
        Err(e) => ExtensionsCmdError::Emit(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_includes_markdown_suffix() {
        let registry = ExtensionRegistry::from_json_str(super::super::SHIPPED_EXTENSIONS_JSON)
            .expect("shipped");
        let text = format_extensions_list(&registry);
        assert!(text.contains("markdown-suffix"), "{text}");
        assert!(text.contains("suffix: .md"), "{text}");
    }

    #[test]
    fn extensions_help_mentions_list_and_show() {
        let text = extensions_usage_message();
        assert!(text.contains("list"), "{text}");
        assert!(text.contains("show"), "{text}");
        assert!(
            text.contains(".wyvern/extensions.json") && text.contains("trusted preexec"),
            "{text}"
        );
    }

    #[test]
    fn bare_extensions_defaults_to_list() {
        let text = run_extensions_command(&[]).expect("list");
        assert!(text.contains("markdown-suffix"), "{text}");
    }

    #[test]
    fn show_without_id_is_missing_extension_id() {
        let err = run_extensions_command(&["show".into()]).expect_err("usage");
        match err {
            ExtensionsCmdError::Usage { kind, message } => {
                assert_eq!(kind, UsageErrorKind::MissingExtensionId);
                assert!(
                    message.contains("extensions show requires an extension id"),
                    "{message}"
                );
            }
            other => panic!("expected Usage, got {other:?}"),
        }
    }

    #[test]
    fn unknown_extensions_subcommand_is_discriminated() {
        let err = run_extensions_command(&["dump".into()]).expect_err("usage");
        match err {
            ExtensionsCmdError::Usage { kind, message } => {
                assert!(matches!(
                    kind,
                    UsageErrorKind::UnknownSubcommand { domain, ref token }
                        if domain == BuiltinDomain::Extensions && token == "dump"
                ));
                assert!(
                    message.contains("unknown extensions subcommand"),
                    "{message}"
                );
            }
            other => panic!("expected Usage, got {other:?}"),
        }
    }

    #[test]
    fn list_prints_requires_availability() {
        let json = r#"{
          "version": 1,
          "extensions": [
            {
              "id": "compose-render",
              "match": { "argv_prefix": ["compose", "render"] },
              "preexec": { "cmd": "sc-compose", "requires": ["sc-compose"] },
              "expand": { "command": { "type": "markdown", "content": "x" } }
            }
          ]
        }"#;
        let registry = ExtensionRegistry::from_json_str(json).expect("parse");
        let text = format_extensions_list(&registry);
        assert!(text.contains("compose-render"), "{text}");
        assert!(text.contains("prefix: compose render"), "{text}");
        assert!(text.contains("sc-compose"), "{text}");
        assert!(
            text.contains("[available]") || text.contains("[missing]"),
            "{text}"
        );
    }

    #[test]
    fn list_unknown_flag_is_validation_error() {
        let err = run_extensions_command(&["list".into(), "--foo".into()]).expect_err("flag");
        match err {
            ExtensionsCmdError::Stage { stderr, exit_code } => {
                assert_eq!(exit_code, 4);
                let value: serde_json::Value = serde_json::from_str(&stderr).expect("json");
                assert_eq!(value["code"], "VALIDATION_ERROR");
                assert!(
                    value["message"].as_str().unwrap_or("").contains("--foo"),
                    "{stderr}"
                );
            }
            other => panic!("expected Stage, got {other:?}"),
        }
    }
}
