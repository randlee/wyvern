//! Load/validation/run-stage errors and JSON emission helpers.

mod emit;

use wyvern_schema::{ErrorCode, FieldName, SerializeError};

#[doc(inline)]
pub use emit::{
    emit_extension_error, emit_fatal_internal, emit_host_error, emit_io_error, emit_parse_error,
    emit_stdout, emit_usage_error, emit_usage_message, emit_validation_error, emit_workflow_error,
};

/// Built-in CLI family that owns subcommands (`browsers`, `extensions`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinDomain {
    /// `wyvern browsers …`
    Browsers,
    /// `wyvern extensions …`
    Extensions,
}

impl BuiltinDomain {
    /// Stable CLI family name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Browsers => "browsers",
            Self::Extensions => "extensions",
        }
    }
}

impl std::fmt::Display for BuiltinDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Host-flag / env usage failure kind for structured stderr recovery (RBP-F009).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UsageErrorKind {
    /// Generic argv/input usage (empty stdin, too many args, etc.).
    Generic,
    /// `--bind` value failed to parse as a socket address.
    InvalidBind {
        /// Raw `--bind` token from argv.
        value: String,
    },
    /// A host flag was given without a following value.
    MissingFlagValue {
        /// Flag name (e.g. `--bind`).
        flag: String,
    },
    /// `--viewer` value was not a known viewer mode.
    InvalidViewer {
        /// Raw `--viewer` token from argv.
        value: String,
    },
    /// `WYVERN_VIEWER` is set but not a valid viewer mode (RSH-010).
    InvalidWyvernViewerEnv {
        /// Raw env var value.
        value: String,
    },
    /// `WYVERN_VIEWER` is not valid Unicode.
    InvalidWyvernViewerUnicode,
    /// Unknown subcommand on a built-in family (`browsers`, `extensions`).
    UnknownSubcommand {
        /// Built-in family that rejected the token.
        domain: BuiltinDomain,
        /// Offending subcommand token.
        token: String,
    },
    /// `extensions show` was invoked without a usable extension id.
    MissingExtensionId,
}

/// Failure while loading command input from argv or stdin.
#[derive(Debug)]
pub enum LoadError {
    /// JSON text could not be parsed.
    Parse { message: String },
    /// A file or stdin read failed.
    Io {
        field: FieldName,
        message: String,
        /// Original I/O error if available.
        source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
    },
    /// Invalid argv shape or host-flag value.
    Usage {
        kind: UsageErrorKind,
        message: String,
    },
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse { message } => write!(f, "parse error: {message}"),
            Self::Io { field, message, .. } => write!(f, "io error ({field}): {message}"),
            Self::Usage { message, .. } => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for LoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => source.as_deref().map(|e| e as _),
            _ => None,
        }
    }
}

impl LoadError {
    /// Stable exit code for this load failure.
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Parse { .. } => ErrorCode::ParseError.exit_code(),
            Self::Io { .. } => ErrorCode::IoError.exit_code(),
            Self::Usage { .. } => ErrorCode::ParseError.exit_code(),
        }
    }
}

/// Failure serializing stdout or structured stderr JSON at the CLI emit boundary.
#[derive(Debug)]
pub enum EmitError {
    /// `serde_json` could not serialize the envelope or result.
    Serialize(SerializeError),
}

impl std::fmt::Display for EmitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Serialize(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for EmitError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Serialize(e) => Some(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wyvern_schema::{ButtonLabel, ChromeResult, CommandResult, FieldName, MessageResult};

    #[test]
    fn emit_parse_error_with_quotes_is_valid_json() {
        let err = LoadError::Parse {
            message: r#"expected value at line 1: "bad""#.to_string(),
        };
        let out = emit_parse_error(&err).expect("emit");
        let value: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        assert_eq!(value["error"], "parse");
        assert_eq!(value["code"], "PARSE_ERROR");
        assert!(value["message"].as_str().unwrap().contains('"'));
        assert!(!value["recovery"].as_array().unwrap().is_empty());
        assert!(value.get("cause").is_some());
    }

    #[test]
    fn emit_io_error_with_quotes_is_valid_json() {
        let err = LoadError::Io {
            field: FieldName::new("file"),
            message: r#"could not read path 'say "hi".json'"#.to_string(),
            source: None,
        };
        let out = emit_io_error(&err).expect("emit");
        let value: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        assert_eq!(value["error"], "io");
        assert_eq!(value["code"], "IO_ERROR");
        assert_eq!(value["field"], "file");
        assert!(value["message"].as_str().unwrap().contains('"'));
        assert!(!value["recovery"].as_array().unwrap().is_empty());
    }

    #[test]
    fn emit_validation_error_message_with_quotes_is_valid_json() {
        let err = wyvern_schema::ValidationError::Validation {
            field: FieldName::new("title"),
            message: r#"field 'title' expected string, got "oops""#.to_string(),
        };
        let out = emit_validation_error(&err).expect("emit");
        let value: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        assert_eq!(value["error"], "validation");
        assert_eq!(value["code"], "VALIDATION_ERROR");
        assert_eq!(value["field"], "title");
        assert!(value["message"].as_str().unwrap().contains('"'));
        assert!(!value["recovery"].as_array().unwrap().is_empty());
    }

    #[test]
    fn emit_validation_error_missing_title_has_actionable_recovery() {
        let err = wyvern_schema::ValidationError::Validation {
            field: FieldName::new("title"),
            message: "missing required field 'title'".to_string(),
        };
        let out = emit_validation_error(&err).expect("emit");
        let value: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        let recovery = value["recovery"].as_array().unwrap();
        assert!(recovery
            .iter()
            .any(|s| s.as_str().unwrap().contains("title")));
    }

    #[test]
    fn emit_validation_error_state() {
        let err = wyvern_schema::ValidationError::State {
            field: FieldName::new("action"),
            message: "show is only valid in --interactive mode".to_string(),
        };
        let out = emit_validation_error(&err).expect("emit");
        let value: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        assert_eq!(value["error"], "state");
        assert_eq!(value["code"], "STATE_ERROR");
        assert_eq!(value["field"], "action");
        assert!(!value["recovery"].as_array().unwrap().is_empty());
    }

    #[test]
    fn emit_stdout_chrome_wire_shape() {
        let result = CommandResult::Chrome(ChromeResult {
            button: ButtonLabel::dismissed(),
        });
        assert_eq!(
            emit_stdout(&result).expect("emit"),
            r#"{"button":"dismissed"}"#
        );
    }

    #[test]
    fn emit_stdout_message_wire_shape() {
        let result = CommandResult::Message(MessageResult {
            button: ButtonLabel::new("ok"),
        });
        assert_eq!(emit_stdout(&result).expect("emit"), r#"{"button":"ok"}"#);
    }

    #[test]
    fn emit_stdout_forced_fail() {
        let _guard = emit::ForceEmitStdoutFailGuard::arm();
        let result = CommandResult::Message(MessageResult {
            button: ButtonLabel::new("ok"),
        });
        assert!(emit_stdout(&result).is_err());
    }

    #[test]
    fn load_error_io_preserves_source_chain() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let err = LoadError::Io {
            field: FieldName::new("file"),
            message: "could not read path".into(),
            source: Some(Box::new(io_err)),
        };
        assert_eq!(err.to_string(), "io error (file): could not read path");
        let source = std::error::Error::source(&err).expect("source chain");
        assert!(source.to_string().contains("missing"));
        assert!(std::error::Error::source(&LoadError::Parse {
            message: "x".into()
        })
        .is_none());
    }

    #[test]
    fn load_error_exit_codes() {
        assert_eq!(
            LoadError::Parse {
                message: "x".into()
            }
            .exit_code(),
            2
        );
        assert_eq!(
            LoadError::Io {
                field: FieldName::new("file"),
                message: "x".into(),
                source: None,
            }
            .exit_code(),
            3
        );
        assert_eq!(
            LoadError::Usage {
                kind: UsageErrorKind::Generic,
                message: "usage".into()
            }
            .exit_code(),
            2
        );
    }

    #[test]
    fn validation_error_exit_codes() {
        assert_eq!(
            wyvern_schema::ValidationError::Validation {
                field: FieldName::new("title"),
                message: "bad".into(),
            }
            .exit_code(),
            4
        );
        assert_eq!(
            wyvern_schema::ValidationError::State {
                field: FieldName::new("action"),
                message: "bad".into(),
            }
            .exit_code(),
            5
        );
    }

    #[test]
    fn emit_usage_error_is_structured_json() {
        let err = LoadError::Usage {
            kind: UsageErrorKind::Generic,
            message: "unknown subcommand".into(),
        };
        let out = emit_usage_error(&err).expect("emit");
        let value: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        assert_eq!(value["error"], "parse");
        assert_eq!(value["code"], "PARSE_ERROR");
        assert!(value["message"].as_str().unwrap().contains("unknown"));
        assert!(!value["recovery"].as_array().unwrap().is_empty());
    }

    #[test]
    fn emit_invalid_bind_has_flag_specific_recovery() {
        let err = LoadError::Usage {
            kind: UsageErrorKind::InvalidBind {
                value: "not-an-addr".into(),
            },
            message: "invalid --bind 'not-an-addr': invalid socket address".into(),
        };
        let out = emit_usage_error(&err).expect("emit");
        let value: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        let recovery = value["recovery"].as_array().unwrap();
        assert!(recovery
            .iter()
            .any(|s| s.as_str().unwrap().contains("--allow-non-loopback")));
        assert!(!value["message"].as_str().unwrap().contains("Recovery:"));
    }

    #[test]
    fn emit_invalid_wyvern_viewer_env_has_flag_specific_recovery() {
        let err = LoadError::Usage {
            kind: UsageErrorKind::InvalidWyvernViewerEnv {
                value: "not-a-viewer-mode".into(),
            },
            message: "invalid WYVERN_VIEWER=\"not-a-viewer-mode\"; expected embedded, none, system, or a named viewer path".into(),
        };
        let out = emit_usage_error(&err).expect("emit");
        let value: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        assert!(value["cause"].as_str().unwrap().contains("WYVERN_VIEWER"));
        let recovery = value["recovery"].as_array().unwrap();
        assert!(recovery
            .iter()
            .any(|s| s.as_str().unwrap().contains("Unset WYVERN_VIEWER")));
    }

    #[test]
    fn emit_unknown_subcommand_has_domain_specific_recovery() {
        let browsers = LoadError::Usage {
            kind: UsageErrorKind::UnknownSubcommand {
                domain: BuiltinDomain::Browsers,
                token: "nope".into(),
            },
            message: "unknown browsers subcommand 'nope'".into(),
        };
        let out = emit_usage_error(&browsers).expect("emit");
        let value: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        assert!(value["cause"].as_str().unwrap().contains("nope"));
        let recovery = value["recovery"].as_array().unwrap();
        assert!(recovery
            .iter()
            .any(|s| s.as_str().unwrap().contains("browsers list")));

        let extensions = LoadError::Usage {
            kind: UsageErrorKind::UnknownSubcommand {
                domain: BuiltinDomain::Extensions,
                token: "show".into(),
            },
            message: "unknown extensions subcommand 'show'".into(),
        };
        let out = emit_usage_error(&extensions).expect("emit");
        let value: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        let recovery = value["recovery"].as_array().unwrap();
        assert!(recovery
            .iter()
            .any(|s| s.as_str().unwrap().contains("extensions list")));
    }

    #[test]
    fn emit_missing_extension_id_has_show_recovery() {
        let err = LoadError::Usage {
            kind: UsageErrorKind::MissingExtensionId,
            message: "extensions show requires an extension id".into(),
        };
        let out = emit_usage_error(&err).expect("emit");
        let value: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        assert!(value["cause"].as_str().unwrap().contains("extension id"));
        let recovery = value["recovery"].as_array().unwrap();
        assert!(recovery
            .iter()
            .any(|s| s.as_str().unwrap().contains("extensions show <id>")));
        assert!(recovery
            .iter()
            .any(|s| s.as_str().unwrap().contains("extensions list")));
    }

    #[test]
    fn emit_missing_args_lists_flags() {
        use crate::extensions::{ExtensionError, ExtensionId};
        let err = ExtensionError::MissingArgs {
            missing: vec!["--root".into(), "--file".into()],
            declared: ["root".into(), "file".into()].into_iter().collect(),
            extension_id: ExtensionId::try_from(String::from("compose-render")).expect("id"),
            example: "wyvern compose render --root DIR --file FILE.j2".into(),
            help_command: "wyvern compose render --help".into(),
        };
        let out = emit_extension_error(&err).expect("emit");
        let value: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        let text = out.to_ascii_lowercase();
        assert!(value["message"].as_str().unwrap().contains("--root"));
        assert!(value["message"].as_str().unwrap().contains("--file"));
        assert!(value["recovery"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s.as_str().unwrap().contains("--root")));
        assert!(value["recovery"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s.as_str() == Some("Run wyvern compose render --help")));
        assert!(
            !out.contains("wyvern compose-render --help"),
            "recovery must use the invocation prefix, not the extension id: {out}"
        );
        assert!(!text.contains("declare them as {arg:"));
    }

    #[test]
    fn emit_unexpected_arg_is_caller_facing() {
        use crate::extensions::{ExtensionError, ExtensionId};
        let err = ExtensionError::UnexpectedArg {
            token: "--undeclared".into(),
            declared: ["root".into(), "file".into()].into_iter().collect(),
            extension_id: ExtensionId::try_from(String::from("compose-render")).expect("id"),
            help_command: "wyvern compose render --help".into(),
        };
        let out = emit_extension_error(&err).expect("emit");
        let value: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        assert!(value["cause"].as_str().unwrap().contains("--undeclared"));
        assert!(value["recovery"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s.as_str().unwrap().contains("--root")));
        assert!(value["recovery"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s.as_str() == Some("Run wyvern compose render --help")));
        assert!(!out.contains("wyvern compose-render --help"), "{out}");
        assert!(
            !out.contains("declare them as {arg:name}") && !out.contains("{arg:"),
            "{out}"
        );
    }

    #[test]
    fn emit_preexec_timeout_mentions_env_var() {
        use crate::extensions::{ExtensionError, PreexecFailureKind};
        let err = ExtensionError::Preexec {
            kind: Some(PreexecFailureKind::Timeout {
                cmd: "slow".into(),
                timeout_secs: 30,
            }),
            message: "slow timed out after 30s".into(),
            source: None,
        };
        let out = emit_extension_error(&err).expect("emit");
        assert!(
            out.contains("WYVERN_PREEXEC_TIMEOUT_SECS"),
            "timeout recovery must name the env var: {out}"
        );
        assert!(out.contains("30"), "{out}");
    }
}
