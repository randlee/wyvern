//! `wyvern extensions list` subcommand.

use super::{match_kind_summary, ExtensionError, ExtensionRegistry};
use crate::error::EmitError;

/// Failure from the `extensions` built-in.
#[derive(Debug)]
pub enum ExtensionsCmdError {
    /// Bad argv.
    Usage {
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
///
/// Mentions `list` only; `show` is a g.3 deliverable.
#[must_use]
pub fn extensions_usage_message() -> String {
    concat!(
        "Usage: wyvern extensions [list]\n",
        "       wyvern extensions --help\n",
        "\n",
        "Commands:\n",
        "  list    List shipped and project CLI extensions\n",
        "\n",
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
    let sub = args.first().map(String::as_str).unwrap_or("list");
    match sub {
        "list" => list(),
        other => Err(ExtensionsCmdError::Usage {
            message: format!(
                "unknown extensions subcommand '{other}'\n{}",
                extensions_usage_message()
            ),
        }),
    }
}

fn list() -> Result<String, ExtensionsCmdError> {
    let registry = ExtensionRegistry::load_default().map_err(map_ext)?;
    Ok(format_extensions_list(&registry))
}

/// Format each extension as `id  match-kind  [(requires: …)]`.
#[must_use]
pub fn format_extensions_list(registry: &ExtensionRegistry) -> String {
    let mut out = String::new();
    for ext in registry.extensions() {
        out.push_str(ext.id.as_str());
        out.push_str("  ");
        out.push_str(&match_kind_summary(&ext.match_spec));
        if !ext.requires().is_empty() {
            out.push_str("  (requires: ");
            out.push_str(&ext.requires().join(", "));
            out.push(')');
        }
        out.push('\n');
    }
    out
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
    fn extensions_help_mentions_list_only() {
        let text = extensions_usage_message();
        assert!(text.contains("list"), "{text}");
        assert!(
            !text.contains("show"),
            "g.1 must not advertise show: {text}"
        );
    }

    #[test]
    fn list_prints_requires() {
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
        assert!(text.contains("(requires: sc-compose)"), "{text}");
    }
}
