//! `wyvern examples list` — discover bundled examples from README frontmatter.

use crate::error::{BuiltinDomain, EmitError, UsageErrorKind};
use crate::examples::{discover_examples, format_examples_list, ExampleRecord, ExamplesDiscoverError};
use crate::extensions::resolve_wyvern_share;
use wyvern_schema::{ErrorCode, SerializeError, StderrError};

/// Usage text for `wyvern examples --help` / `-h`.
#[must_use]
pub fn examples_usage_message() -> String {
    concat!(
        "Usage: wyvern examples [list] [--json]\n",
        "       wyvern examples --help\n",
        "\n",
        "Commands:\n",
        "  list         List bundled examples discovered from README frontmatter\n",
        "\n",
        "Options:\n",
        "  --json       Print ExampleRecord JSON array\n",
        "\n",
        "Each example README under {wyvern_share}/examples/ must begin with:\n",
        "  ---\n",
        "  name: Example title\n",
        "  description: One-line summary\n",
        "  ---\n",
        "\n",
        "See also: wyvern guide, wyvern --help\n",
    )
    .to_string()
}

/// Run `wyvern examples …`; returns stdout text on success.
///
/// # Errors
///
/// Returns usage text or structured stderr for invalid argv / discovery I/O.
pub fn run_examples_command(args: &[String]) -> Result<String, ExamplesCmdError> {
    if args
        .first()
        .is_some_and(|token| token == "--help" || token == "-h")
    {
        return Ok(examples_usage_message());
    }
    match args.first().map(String::as_str) {
        None => run_list(args),
        Some("list") => run_list(&args[1..]),
        Some("--json") => run_list(args),
        Some(other) if other.starts_with('-') => Err(unknown_flag(other)),
        Some(other) => Err(ExamplesCmdError::Usage {
            kind: UsageErrorKind::UnknownSubcommand {
                domain: BuiltinDomain::Examples,
                token: other.to_string(),
            },
            message: format!(
                "unknown examples subcommand '{other}'\n{}",
                examples_usage_message()
            ),
        }),
    }
}

fn run_list(args: &[String]) -> Result<String, ExamplesCmdError> {
    if wants_help(args) {
        return Ok(examples_usage_message());
    }
    let json = parse_list_flags(args)?;
    let share_root = resolve_wyvern_share();
    let records = discover_examples(&share_root).map_err(map_discover)?;
    if json {
        serialize_records_json(&records)
    } else {
        Ok(format_examples_list(&records))
    }
}

fn wants_help(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "--help" || arg == "-h")
}

fn parse_list_flags(args: &[String]) -> Result<bool, ExamplesCmdError> {
    let mut json = false;
    for arg in args {
        match arg.as_str() {
            "--json" => json = true,
            other => return Err(unknown_flag(other)),
        }
    }
    Ok(json)
}

fn serialize_records_json(records: &[ExampleRecord]) -> Result<String, ExamplesCmdError> {
    match serde_json::to_string_pretty(records) {
        Ok(mut text) => {
            if !text.ends_with('\n') {
                text.push('\n');
            }
            Ok(text)
        }
        Err(err) => Err(ExamplesCmdError::Emit(EmitError::Serialize(
            SerializeError {
                message: err.to_string(),
            },
        ))),
    }
}

fn unknown_flag(flag: &str) -> ExamplesCmdError {
    match StderrError::new(ErrorCode::ValidationError, format!("unknown flag '{flag}'"))
        .cause("examples list accepts only --json")
        .recovery("Run wyvern examples list")
        .recovery("Run wyvern examples list --json")
        .recovery("Run wyvern examples --help")
        .docs("docs/wyvern/requirements.md")
        .to_json_string()
    {
        Ok(stderr) => ExamplesCmdError::Stage {
            stderr,
            exit_code: ErrorCode::ValidationError.exit_code(),
        },
        Err(err) => ExamplesCmdError::Emit(EmitError::Serialize(err)),
    }
}

fn map_discover(err: ExamplesDiscoverError) -> ExamplesCmdError {
    match StderrError::new(ErrorCode::ValidationError, err.to_string())
        .cause("Could not scan bundled example README files")
        .recovery("Run wyvern examples list")
        .recovery("Ensure {wyvern_share}/examples exists and README files are readable")
        .docs("docs/wyvern/requirements.md")
        .to_json_string()
    {
        Ok(stderr) => ExamplesCmdError::Stage {
            stderr,
            exit_code: ErrorCode::ValidationError.exit_code(),
        },
        Err(e) => ExamplesCmdError::Emit(EmitError::Serialize(e)),
    }
}

/// CLI `examples` subcommand failure.
#[derive(Debug)]
pub enum ExamplesCmdError {
    /// Bad argv.
    Usage {
        /// Discriminated usage class for structured stderr recovery.
        kind: UsageErrorKind,
        /// Plain-text usage.
        message: String,
    },
    /// Discovery or emit failure with stderr JSON.
    Stage {
        /// Stderr JSON.
        stderr: String,
        /// Process exit code.
        exit_code: i32,
    },
    /// Emit-boundary serialize failure.
    Emit(EmitError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn examples_help_mentions_list_and_frontmatter() {
        let text = examples_usage_message();
        assert!(text.contains("list"), "{text}");
        assert!(text.contains("name:"), "{text}");
        assert!(text.contains("description:"), "{text}");
    }

    #[test]
    fn bare_examples_defaults_to_list() {
        let text = run_examples_command(&[]).expect("list");
        assert!(text.contains("README:") || text.is_empty(), "{text}");
    }
}
