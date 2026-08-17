//! Load/validation/run-stage errors and JSON emission helpers.

use wyvern_schema::{ErrorCode, FieldName, SerializeError, StderrError, ValidationError};

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
        /// Built-in family name (`browsers` or `extensions`).
        domain: String,
        /// Offending subcommand token.
        token: String,
    },
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
thread_local! {
    /// Scoped test seam: only the arming thread sees forced stdout emit failures.
    static FORCE_EMIT_STDOUT_FAIL: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// RAII guard that forces [`emit_stdout`] to fail on this thread.
#[cfg(test)]
struct ForceEmitStdoutFailGuard;

#[cfg(test)]
impl ForceEmitStdoutFailGuard {
    fn arm() -> Self {
        FORCE_EMIT_STDOUT_FAIL.with(|f| f.set(true));
        Self
    }
}

#[cfg(test)]
impl Drop for ForceEmitStdoutFailGuard {
    fn drop(&mut self) {
        FORCE_EMIT_STDOUT_FAIL.with(|f| f.set(false));
    }
}

/// Serialize an extension-engine error as stderr JSON.
///
/// # Errors
///
/// Returns [`EmitError::Serialize`] when the envelope cannot be serialized.
pub fn emit_extension_error(err: &crate::extensions::ExtensionError) -> Result<String, EmitError> {
    use crate::extensions::ExtensionError;
    let (code, message, cause, recovery) = match err {
        ExtensionError::InvalidRegistry { message } => (
            ErrorCode::ParseError,
            message.clone(),
            "Extension registry JSON could not be loaded".to_string(),
            vec![
                "Fix share/wyvern/extensions.json or .wyvern/extensions.json".into(),
                "Registry must be version 1 JSON with an extensions array".into(),
            ],
        ),
        ExtensionError::MissingArgs {
            missing,
            extension_id,
            example,
            ..
        } => (
            ErrorCode::ValidationError,
            format!(
                "missing required arguments {} for '{extension_id}'",
                missing.join(", ")
            ),
            format!("'{extension_id}' requires {}", missing.join(" and ")),
            vec![
                format!("Pass {} after the extension prefix", missing.join(" ")),
                format!("Example: {example}"),
                format!("Run wyvern {extension_id} --help"),
                "Run wyvern --help to list skills".into(),
            ],
        ),
        ExtensionError::UnexpectedArg {
            token,
            declared,
            extension_id,
        } => {
            let accepted = if declared.is_empty() {
                format!("Run wyvern {extension_id} --help")
            } else {
                format!(
                    "Accepted flags: {}",
                    declared
                        .iter()
                        .map(|name| format!("--{name}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            };
            (
                ErrorCode::ValidationError,
                format!("unexpected argument after extension match: {token}"),
                format!("'{extension_id}' does not accept leftover token '{token}'"),
                vec![
                    format!("Remove unexpected argument `{token}`"),
                    accepted,
                    "Run wyvern --help to list skills".into(),
                ],
            )
        }
        ExtensionError::PathVarWithoutPath { var } => (
            ErrorCode::ValidationError,
            format!("template {{{var}}} requires a matched file path"),
            "This extension is prefix-only and has no {{path}}".to_string(),
            vec!["Use a suffix or prefix+suffix match when expanding path variables".into()],
        ),
        ExtensionError::Template { kind, message, .. } => {
            let (cause, recovery) = match kind {
                crate::extensions::TemplateErrorKind::UnclosedBrace => (
                    "Template contains an unclosed `{` brace".to_string(),
                    vec!["Close every `{variable}` in the registry expand/preexec templates".into()],
                ),
                crate::extensions::TemplateErrorKind::UnknownVariable => (
                    "Template references an unknown `{variable}`".to_string(),
                    vec!["Use only documented template variables from cli-extensions-contract.md".into()],
                ),
                crate::extensions::TemplateErrorKind::PhaseRestricted => (
                    "Template variable is not allowed in this expansion phase".to_string(),
                    vec!["Move path-only variables to expand phase-1; use phase-2 for preexec stdout vars".into()],
                ),
                crate::extensions::TemplateErrorKind::Unavailable => (
                    "Template variable is not available in this match context".to_string(),
                    vec!["Ensure the match provides path/tmpdir/preexec stdout before using this variable".into()],
                ),
                crate::extensions::TemplateErrorKind::InvalidSpec => (
                    "Expand/preexec spec is incomplete or contradictory".to_string(),
                    vec!["Check command_from_file, preexec cmd/args, and host overrides in the registry".into()],
                ),
            };
            (ErrorCode::ValidationError, message.clone(), cause, recovery)
        }
        ExtensionError::Preexec { kind, message, .. } => {
            use crate::extensions::PreexecFailureKind;
            let (cause, recovery) = match kind {
                Some(PreexecFailureKind::SpawnNotFound { cmd }) => (
                    format!("Could not spawn preexec helper '{cmd}'"),
                    vec![
                        format!("Install '{cmd}' or add it to PATH"),
                        "Run wyvern extensions list to see requires".into(),
                        "Run wyvern --help to list skills".into(),
                    ],
                ),
                Some(PreexecFailureKind::NonZeroExit { stderr_tail, code }) => {
                    let cause = if stderr_tail.is_empty() {
                        format!("Preexec helper exited with status {code}")
                    } else {
                        stderr_tail.clone()
                    };
                    (
                        cause,
                        vec![
                            "Inspect the helper stderr in cause and fix the input path or flags"
                                .into(),
                            "Retry after correcting the file or arguments".into(),
                            "Run wyvern --help to list skills".into(),
                        ],
                    )
                }
                Some(PreexecFailureKind::Timeout { cmd, timeout_secs }) => (
                    format!("Preexec helper '{cmd}' timed out after {timeout_secs}s"),
                    vec![
                        format!(
                            "Increase WYVERN_PREEXEC_TIMEOUT_SECS (current {timeout_secs}) if the helper needs more time"
                        ),
                        "Or fix a hung helper or blocked input path".into(),
                        "Run wyvern --help to list skills".into(),
                    ],
                ),
                None => (
                    "Extension preexec subprocess failed".to_string(),
                    vec![
                        "Inspect the helper output in the error message".into(),
                        "Retry after correcting the input path or flags".into(),
                        "Run wyvern --help to list skills".into(),
                    ],
                ),
            };
            (ErrorCode::IoError, message.clone(), cause, recovery)
        }
        ExtensionError::InvalidCommand { source } => {
            return emit_validation_error(source);
        }
        ExtensionError::Io { message, .. } => (
            ErrorCode::IoError,
            message.clone(),
            "Extension engine filesystem operation failed".to_string(),
            vec!["Check paths in the registry and working directory permissions".into()],
        ),
    };
    let mut envelope = StderrError::new(code, message)
        .cause(cause)
        .docs("docs/plans/phase-F/cli-extensions-contract.md");
    for step in recovery {
        envelope = envelope.recovery(step);
    }
    envelope.to_json_string().map_err(EmitError::Serialize)
}

/// Serialize a usage / unknown-subcommand error as stderr JSON (exit 2).
///
/// # Errors
///
/// Returns [`EmitError::Serialize`] when the envelope cannot be serialized.
pub fn emit_usage_message(message: &str) -> Result<String, EmitError> {
    StderrError::new(ErrorCode::ParseError, message.to_string())
        .cause("CLI argv was not a valid command or extension invocation")
        .recovery("Pass a JSON command, a .json file, or a path handled by an extension")
        .recovery("Run wyvern extensions list to see file-type and prefix extensions")
        .recovery("Run wyvern --help for host flags")
        .docs("docs/wyvern/requirements.md (REQ-0130)")
        .to_json_string()
        .map_err(EmitError::Serialize)
}

/// Serialize [`LoadError::Usage`] as stderr JSON with flag-specific recovery.
///
/// # Errors
///
/// Returns [`EmitError::Serialize`] when the envelope cannot be serialized, or
/// when `err` is not [`LoadError::Usage`] (miswire).
pub fn emit_usage_error(err: &LoadError) -> Result<String, EmitError> {
    let LoadError::Usage { kind, message } = err else {
        debug_assert!(matches!(err, LoadError::Usage { .. }));
        return Err(EmitError::Serialize(SerializeError {
            message: "emit_usage_error: expected Usage".into(),
        }));
    };
    let (cause, recovery, docs) = match kind {
        UsageErrorKind::Generic => (
            "CLI argv was not a valid command or extension invocation".to_string(),
            vec![
                "Pass a JSON command, a .json file, or a path handled by an extension".into(),
                "Run wyvern extensions list to see file-type and prefix extensions".into(),
                "Run wyvern --help for host flags".into(),
            ],
            "docs/wyvern/requirements.md (REQ-0130)",
        ),
        UsageErrorKind::InvalidBind { .. } => (
            "The --bind value is not a valid socket address".to_string(),
            vec![
                "Use host:port form (example: 127.0.0.1:0 for an ephemeral loopback port)".into(),
                "For 0.0.0.0 / LAN binds, also pass --allow-non-loopback".into(),
                "Check the address is a valid IPv4/IPv6 socket address".into(),
            ],
            "docs/plans/phase-F/README.md",
        ),
        UsageErrorKind::MissingFlagValue { flag } => (
            format!("Host flag {flag} requires a value"),
            vec![
                format!("Pass {flag} VALUE on the command line"),
                "Use --bind=ADDR:PORT or --viewer=MODE inline forms when preferred".into(),
            ],
            "docs/plans/phase-F/README.md",
        ),
        UsageErrorKind::InvalidViewer { .. } => (
            "The --viewer value is not a supported viewer mode".to_string(),
            vec![
                "Use one of: embedded, none, system, chrome, safari, edge, firefox".into(),
                "Omit --viewer to use embedded (default)".into(),
                "Set WYVERN_VIEWER=none for headless / CI".into(),
            ],
            "docs/plans/phase-C/http-viewer-contract.md",
        ),
        UsageErrorKind::InvalidWyvernViewerEnv { .. } => (
            "WYVERN_VIEWER is set but not a valid viewer mode".to_string(),
            vec![
                "Use one of: embedded, none, system, chrome, safari, edge, firefox".into(),
                "Unset WYVERN_VIEWER to use embedded (default)".into(),
                "Use WYVERN_VIEWER=none for headless / CI".into(),
            ],
            "docs/plans/phase-C/http-viewer-contract.md",
        ),
        UsageErrorKind::InvalidWyvernViewerUnicode => (
            "WYVERN_VIEWER is not valid Unicode".to_string(),
            vec![
                "Set WYVERN_VIEWER to ASCII viewer mode names only".into(),
                "Unset WYVERN_VIEWER to use embedded (default)".into(),
            ],
            "docs/plans/phase-C/http-viewer-contract.md",
        ),
        UsageErrorKind::UnknownSubcommand { domain, token } => (
            format!("'{token}' is not a valid {domain} subcommand"),
            match domain.as_str() {
                "browsers" => vec![
                    "Use wyvern browsers list or wyvern browsers refresh".into(),
                    "Run wyvern browsers --help".into(),
                ],
                "extensions" => vec![
                    "Use wyvern extensions list or wyvern extensions show <id>".into(),
                    "Run wyvern extensions --help".into(),
                ],
                _ => vec![
                    format!("Run wyvern {domain} --help"),
                    "Run wyvern --help for host flags".into(),
                ],
            },
            "docs/wyvern/requirements.md (REQ-0134)",
        ),
    };
    let mut envelope = StderrError::new(ErrorCode::ParseError, message.clone())
        .cause(cause)
        .docs(docs);
    for step in recovery {
        envelope = envelope.recovery(step);
    }
    envelope.to_json_string().map_err(EmitError::Serialize)
}

/// Serialize a parse load error as stderr JSON.
///
/// # Errors
///
/// Returns [`EmitError::Serialize`] when the envelope cannot be serialized, or
/// when `err` is not [`LoadError::Parse`] (miswire).
pub fn emit_parse_error(err: &LoadError) -> Result<String, EmitError> {
    let LoadError::Parse { message } = err else {
        debug_assert!(matches!(err, LoadError::Parse { .. }));
        return Err(EmitError::Serialize(SerializeError {
            message: "emit_parse_error: expected Parse".into(),
        }));
    };
    StderrError::new(ErrorCode::ParseError, message.clone())
        .cause("Input was not valid JSON")
        .recovery("Ensure input is valid JSON")
        .recovery("Check for trailing commas, unquoted keys, or truncated input")
        .docs("docs/wyvern-schema/requirements.md (REQ-0069)")
        .to_json_string()
        .map_err(EmitError::Serialize)
}

/// Serialize an I/O load error as stderr JSON.
///
/// # Errors
///
/// Returns [`EmitError::Serialize`] when the envelope cannot be serialized, or
/// when `err` is not [`LoadError::Io`] (miswire).
pub fn emit_io_error(err: &LoadError) -> Result<String, EmitError> {
    let LoadError::Io { field, message, .. } = err else {
        debug_assert!(matches!(err, LoadError::Io { .. }));
        return Err(EmitError::Serialize(SerializeError {
            message: "emit_io_error: expected Io".into(),
        }));
    };
    StderrError::new(ErrorCode::IoError, message.clone())
        .field(field.clone())
        .cause(format!("Failed to read input from '{}'", field.as_str()))
        .recovery("Verify the file path exists and is readable")
        .recovery("Pass JSON inline as an argv string or via stdin")
        .docs("docs/wyvern-schema/requirements.md (REQ-0071)")
        .to_json_string()
        .map_err(EmitError::Serialize)
}

/// Serialize a validation/state error as stderr JSON.
///
/// # Errors
///
/// Returns [`EmitError::Serialize`] when the envelope cannot be serialized.
pub fn emit_validation_error(err: &ValidationError) -> Result<String, EmitError> {
    let envelope = match err {
        ValidationError::Validation { field, message } => {
            let mut envelope = StderrError::new(ErrorCode::ValidationError, message.clone())
                .field(field.clone())
                .cause(format!("Command JSON failed schema checks on '{field}'"))
                .docs("docs/wyvern-schema/requirements.md (REQ-0051, REQ-0070)");
            for step in validation_recovery(field.as_str(), message) {
                envelope = envelope.recovery(step);
            }
            envelope
        }
        ValidationError::State { field, message } => {
            StderrError::new(ErrorCode::StateError, message.clone())
                .field(field.clone())
                .cause("Lifecycle action used outside interactive mode")
                .recovery("Run with --interactive to use lifecycle actions (show/hide/exit)")
                .recovery("Omit the action field for one-shot chrome commands")
                .docs("docs/wyvern-schema/requirements.md (REQ-0072)")
        }
    };
    envelope.to_json_string().map_err(EmitError::Serialize)
}

fn validation_recovery(field: &str, message: &str) -> Vec<String> {
    if field == "title" && message.contains("missing required field") {
        return vec![
            "Add required field \"title\" with a string value".into(),
            "Example: {\"type\":\"chrome\",\"title\":\"Foundation\"}".into(),
        ];
    }
    if field == "type" && message.contains("missing required field") {
        return vec![
            "Add required field \"type\" with value \"chrome\"".into(),
            "Example: {\"type\":\"chrome\",\"title\":\"Foundation\"}".into(),
        ];
    }
    if field == "type" && message.contains("expected one of") {
        return vec![
            "Set \"type\" to one of: chrome, message, input, markdown, question, wizard".into(),
            "Example: {\"type\":\"wizard\",\"page\":{\"id\":\"start\",\"title\":\"Start\",\"html\":\"pages/start.html\"}}"
                .into(),
        ];
    }
    if field == "page" && message.contains("missing required field") {
        return vec![
            "Add required object field \"page\" with id, title, and html".into(),
            "Example: {\"type\":\"wizard\",\"page\":{\"id\":\"start\",\"title\":\"Start\",\"html\":\"pages/start.html\"}}"
                .into(),
        ];
    }
    if field == "page" && message.contains("expected object") {
        return vec!["Provide \"page\" as a JSON object with id, title, and html".into()];
    }
    if field == "page.id" {
        return vec![
            "Set \"page.id\" to a non-empty string page identity".into(),
            "Example: \"page\":{\"id\":\"start\",\"title\":\"Start\",\"html\":\"pages/start.html\"}"
                .into(),
        ];
    }
    if field == "page.title" {
        return vec![
            "Set \"page.title\" to a non-empty string display title".into(),
            "Example: \"page\":{\"id\":\"start\",\"title\":\"Start\",\"html\":\"pages/start.html\"}"
                .into(),
        ];
    }
    if field == "page.html" {
        return vec![
            "Set \"page.html\" to a non-empty path relative to --ui-root".into(),
            "Example: \"page\":{\"id\":\"start\",\"title\":\"Start\",\"html\":\"pages/start.html\"}"
                .into(),
        ];
    }
    if field == "page.layout" {
        return vec!["Set \"page.layout\" to one of: dialog, workspace (or omit the field)".into()];
    }
    if field.starts_with("page.") && message.contains("unknown field") {
        return vec![format!(
            "Remove unknown field \"{field}\"; page allows only id, title, html, and layout"
        )];
    }
    if field == "buttons" {
        return vec![
            "Set \"buttons\" to one of: ok, ok_cancel, yes_no, yes_no_cancel, retry_cancel, custom"
                .into(),
        ];
    }
    if field == "level" {
        return vec!["Set \"level\" to one of: info, warning, error, question".into()];
    }
    if field == "custom_buttons" {
        return vec![
            "Provide \"custom_buttons\" as a string array only when \"buttons\" is \"custom\""
                .into(),
        ];
    }
    if field == "default_button" {
        return vec![
            "Set \"default_button\" to a 0-based index within the active button list".into(),
        ];
    }
    if field == "markdown" {
        return vec!["Provide \"markdown\" as a JSON boolean (true or false)".into()];
    }
    if field == "file" && message.contains("exactly one of") {
        return vec![
            "Provide exactly one of \"file\" or \"content\" for markdown commands".into(),
            "Example: {\"type\":\"markdown\",\"file\":\"doc.md\"}".into(),
            "Example: {\"type\":\"markdown\",\"content\":\"# Hello\"}".into(),
        ];
    }
    if message.contains("expected string") {
        return vec![format!("Provide field \"{field}\" as a JSON string")];
    }
    if message.contains("unknown field") {
        return vec![format!(
            "Remove unknown field \"{field}\"; check the schema for this command type"
        )];
    }
    if message.contains("expected JSON object") {
        return vec!["Pass a single JSON object as the command payload".into()];
    }
    vec![format!(
        "Fix field \"{field}\" to match the current phase command schema"
    )]
}

/// Serialize a successful [`wyvern_schema::CommandResult`] for stdout.
///
/// # Errors
///
/// Returns [`EmitError::Serialize`] when `result` cannot be serialized.
pub fn emit_stdout(result: &wyvern_schema::CommandResult) -> Result<String, EmitError> {
    #[cfg(test)]
    {
        if FORCE_EMIT_STDOUT_FAIL.with(std::cell::Cell::get) {
            return Err(EmitError::Serialize(SerializeError {
                message: "forced".into(),
            }));
        }
    }
    serde_json::to_string(result).map_err(|e| {
        EmitError::Serialize(SerializeError {
            message: e.to_string(),
        })
    })
}

/// Serialize a [`wyvern_host::HostError`] as stderr JSON (REQ-0073).
///
/// # Errors
///
/// Returns [`EmitError::Serialize`] when the envelope cannot be serialized.
pub fn emit_host_error(err: &wyvern_host::HostError) -> Result<String, EmitError> {
    use wyvern_host::HostError;
    let (code, message, cause, recovery, docs) = match err {
        HostError::Bind { message, source } => {
            let message = match source {
                Some(err) => format!("{message}: {err}"),
                None => message.clone(),
            };
            (
                ErrorCode::HostBindError,
                message,
                "Failed to bind the dialog HTTP server".to_string(),
                vec![
                    "Check that --bind is a valid address".into(),
                    "Try --bind 127.0.0.1:0 for an ephemeral port".into(),
                ],
                "docs/wyvern-host/requirements.md (REQ-0091)",
            )
        }
        HostError::UiNotFound { path, source } => {
            let message = match source {
                Some(err) => format!("UI not found at '{}': {err}", path.display()),
                None => format!("UI not found at '{}'", path.display()),
            };
            (
                ErrorCode::UiNotFound,
                message,
                "Packaged UI root, dialog template, or wizard page HTML is missing".to_string(),
                vec![
                    "Pass --ui-root pointing at a directory with message/, input/, markdown/, question/, and chrome/ templates".into(),
                    "For wizard commands, ensure page.html exists relative to --ui-root (served under /wizard/**)".into(),
                    "Ensure ui/{message,input,markdown,question,chrome}/ exist in the workspace for development".into(),
                ],
                "docs/wyvern-host/requirements.md (REQ-0093, REQ-0100)",
            )
        }
        HostError::UnsupportedType { type_name } => (
            ErrorCode::UnsupportedType,
            format!("dialog type '{type_name}' is not implemented on the HTTP host yet"),
            "Schema validation passed; host matrix supports chrome, message, input, markdown, question, and wizard".to_string(),
            vec![
                "Use one of: chrome, message, input, markdown, question, wizard".into(),
            ],
            "docs/plans/phase-C/http-dialog-contract.md",
        ),
        HostError::InvalidResult { message } => (
            ErrorCode::HostError,
            message.clone(),
            "POST /api/result body was invalid for the active dialog".to_string(),
            vec!["Submit a body matching the dialog CommandResult wire shape".into()],
            "docs/plans/phase-C/http-post-schema.md",
        ),
        HostError::ViewerNotFound { id, hint } => (
            ErrorCode::HostViewerError,
            format!("viewer '{id}' not found"),
            hint.clone(),
            vec![
                format!("Install {id} or use --viewer system"),
                "Use --viewer none for headless / CI".into(),
            ],
            "docs/plans/phase-C/http-viewer-contract.md",
        ),
        HostError::ViewerUnsupported { mode } => (
            ErrorCode::HostViewerError,
            format!(
                "viewer mode '{}' is not supported by host::run",
                mode.as_str()
            ),
            "Embedded one-shot must use begin + wyvern-viewer spawn (CLI pipeline)".to_string(),
            vec![
                "Omit --viewer or use --viewer embedded (CLI default)".into(),
                "Use --viewer none for headless / CI".into(),
            ],
            "docs/plans/phase-C/http-viewer-contract.md",
        ),
        HostError::Registry { message } => (
            ErrorCode::HostError,
            message.clone(),
            "Browser registry cache read/write failed".to_string(),
            vec![
                "Run `wyvern browsers refresh` to rebuild the cache".into(),
                "Check WYVERN_BROWSERS_FILE path and cache directory permissions".into(),
                "Delete a corrupt browsers.json and retry".into(),
            ],
            "docs/plans/phase-C/http-viewer-contract.md",
        ),
        HostError::Internal { message } => (
            ErrorCode::HostError,
            message.clone(),
            "Internal HTTP host failure".to_string(),
            vec![
                "Retry the command".into(),
                "Report a bug if it persists".into(),
            ],
            "docs/wyvern-host/architecture.md",
        ),
        HostError::Wizard { source } => {
            let subcode = source.subcode();
            (
                ErrorCode::HostError,
                format!("{subcode}: {source}"),
                format!("{subcode}: wizard session failed during host setup or state access"),
                vec![
                    format!("See wizard error sub-code {subcode} for the specific failure"),
                    "Ensure the command is type: wizard with a validated page object".into(),
                    "Retry the command; report a bug if a validated wizard has no session".into(),
                ],
                "docs/plans/phase-C/http-wizard-contract.md",
            )
        }
    };

    let mut envelope = StderrError::new(code, message).cause(cause).docs(docs);
    for step in recovery {
        envelope = envelope.recovery(step);
    }
    envelope.to_json_string().map_err(EmitError::Serialize)
}

/// Emit static internal stderr JSON and exit with code 8 (REQ-0078).
///
/// Uses a hand-built JSON string so a serialize failure cannot recurse.
/// Includes `cause` / `recovery` / `docs` per the stderr contract (RBP-F004).
pub fn emit_fatal_internal(err: &EmitError) -> ! {
    let EmitError::Serialize(e) = err;
    let msg_json =
        serde_json::to_string(&e.message).unwrap_or_else(|_| "\"serialization failed\"".into());
    eprintln!(
        r#"{{"error":"internal","code":"INTERNAL_ERROR","message":{msg_json},"cause":"Stdout or stderr JSON serialization failed at the CLI emit boundary","recovery":["Retry the command","Report a bug if the payload is valid JSON but emit still fails"],"docs":"docs/wyvern-schema/requirements.md (REQ-0078)"}}"#
    );
    std::process::exit(ErrorCode::InternalError.exit_code());
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
        let err = ValidationError::Validation {
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
        let err = ValidationError::Validation {
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
        let err = ValidationError::State {
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
        let _guard = ForceEmitStdoutFailGuard::arm();
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
            ValidationError::Validation {
                field: FieldName::new("title"),
                message: "bad".into(),
            }
            .exit_code(),
            4
        );
        assert_eq!(
            ValidationError::State {
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
                domain: "browsers".into(),
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
                domain: "extensions".into(),
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
    fn emit_missing_args_lists_flags() {
        use crate::extensions::{ExtensionError, ExtensionId};
        let err = ExtensionError::MissingArgs {
            missing: vec!["--root".into(), "--file".into()],
            declared: ["root".into(), "file".into()].into_iter().collect(),
            extension_id: ExtensionId::try_from(String::from("compose-render")).expect("id"),
            example: "wyvern compose render --root DIR --file FILE.j2".into(),
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
        assert!(!text.contains("declare them as {arg:"));
    }

    #[test]
    fn emit_unexpected_arg_is_caller_facing() {
        use crate::extensions::{ExtensionError, ExtensionId};
        let err = ExtensionError::UnexpectedArg {
            token: "--undeclared".into(),
            declared: ["root".into(), "file".into()].into_iter().collect(),
            extension_id: ExtensionId::try_from(String::from("compose-render")).expect("id"),
        };
        let out = emit_extension_error(&err).expect("emit");
        let value: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        assert!(value["cause"].as_str().unwrap().contains("--undeclared"));
        assert!(value["recovery"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s.as_str().unwrap().contains("--root")));
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
