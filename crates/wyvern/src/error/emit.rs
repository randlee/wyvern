//! JSON emission helpers for load, validation, host, and extension errors.

use wyvern_schema::{ErrorCode, SerializeError, StderrError, ValidationError};

use super::{EmitError, LoadError, UsageErrorKind};

#[cfg(test)]
thread_local! {
    /// Scoped test seam: only the arming thread sees forced stdout emit failures.
    static FORCE_EMIT_STDOUT_FAIL: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// RAII guard that forces [`emit_stdout`] to fail on this thread.
#[cfg(test)]
pub(super) struct ForceEmitStdoutFailGuard;

#[cfg(test)]
impl ForceEmitStdoutFailGuard {
    pub(super) fn arm() -> Self {
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
            help_command,
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
                format!("Run {help_command}"),
                "Run wyvern --help to list skills".into(),
            ],
        ),
        ExtensionError::UnexpectedArg {
            token,
            declared,
            extension_id,
            help_command,
        } => {
            let accepted = if declared.is_empty() {
                format!("Run {help_command}")
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
                    format!("Run {help_command}"),
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
            match domain {
                super::BuiltinDomain::Browsers => vec![
                    "Use wyvern browsers list or wyvern browsers refresh".into(),
                    "Run wyvern browsers --help".into(),
                ],
                super::BuiltinDomain::Extensions => vec![
                    "Use wyvern extensions list or wyvern extensions show <id>".into(),
                    "Run wyvern extensions --help".into(),
                ],
                super::BuiltinDomain::Examples => vec![
                    "Use wyvern examples list or wyvern examples list --json".into(),
                    "Run wyvern examples --help".into(),
                ],
                super::BuiltinDomain::Wizard => vec![
                    "Use wyvern wizard lint <path>".into(),
                    "Run wyvern wizard --help".into(),
                ],
            },
            "docs/wyvern/requirements.md (REQ-0134)",
        ),
        UsageErrorKind::MissingExtensionId => (
            "extensions show requires a shipped or project extension id".to_string(),
            vec![
                "Pass an id: wyvern extensions show <id>".into(),
                "Run wyvern extensions list to see available ids".into(),
                "Run wyvern extensions list --json for machine-readable ids".into(),
                "Run wyvern extensions --help".into(),
            ],
            "docs/wyvern/requirements.md (REQ-0132)",
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
            "Set \"type\" to one of: chrome, message, input, markdown, question, wizard, report"
                .into(),
            "Example: {\"type\":\"report\",\"title\":\"Panel\",\"page\":\"pages/view.xhtml\",\"mode\":\"view\"}"
                .into(),
            "Example: {\"type\":\"wizard\",\"page\":{\"id\":\"start\",\"title\":\"Start\",\"html\":\"pages/start.html\"}}"
                .into(),
        ];
    }
    if field == "page" && message.contains("missing required field") {
        return vec![
            "For report commands, set \"page\" to a .html or .xhtml path string relative to --ui-root"
                .into(),
            "Example: {\"type\":\"report\",\"title\":\"Panel\",\"page\":\"pages/view.xhtml\",\"mode\":\"view\"}"
                .into(),
            "For wizard commands, add required object field \"page\" with id, title, and html"
                .into(),
            "Example: {\"type\":\"wizard\",\"page\":{\"id\":\"start\",\"title\":\"Start\",\"html\":\"pages/start.html\"}}"
                .into(),
        ];
    }
    if field == "page"
        && (message.contains("must end with")
            || message.contains("expected string")
            || message.contains("non-empty string"))
    {
        return vec![
            "Set report \"page\" to a non-empty .html or .xhtml path string relative to --ui-root"
                .into(),
            "Example: {\"type\":\"report\",\"title\":\"Panel\",\"page\":\"pages/view.xhtml\",\"mode\":\"view\"}"
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
                "Packaged UI root, dialog template, wizard page HTML, or report page is missing".to_string(),
                vec![
                    "Pass --ui-root pointing at a directory with message/, input/, markdown/, question/, and chrome/ templates".into(),
                    "For wizard commands, ensure page.html exists relative to --ui-root (served under /wizard/**)".into(),
                    "For report commands, ensure page exists relative to --ui-root (served under /report/**)".into(),
                    "Ensure ui/{message,input,markdown,question,chrome}/ exist in the workspace for development".into(),
                ],
                "docs/wyvern-host/requirements.md (REQ-0093, REQ-0100)",
            )
        }
        HostError::UnsupportedType { type_name } => (
            ErrorCode::UnsupportedType,
            format!("dialog type '{type_name}' is not implemented on the HTTP host yet"),
            "Schema validation passed; host matrix supports chrome, message, input, markdown, question, wizard, and report".to_string(),
            vec![
                "Use one of: chrome, message, input, markdown, question, wizard, report".into(),
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
        HostError::SessionTimeout { timeout } => (
            ErrorCode::SessionTimeoutError,
            format!(
                "session idle timeout after {}s: no result posted",
                timeout.as_secs()
            ),
            "Headless blocking dialog was not driven — harness must open WYVERN_DIALOG_URL and click a button (~1s when wired correctly)".to_string(),
            vec![
                "Drive WYVERN_DIALOG_URL in CI (Playwright/curl) and POST /api/result".into(),
                "Use `wyvern examples list` for instant headless smoke without a blocking dialog".into(),
            ],
            "docs/plans/phase-C/c9-testing-headless.md",
        ),
    };

    let mut envelope = StderrError::new(code, message).cause(cause).docs(docs);
    if let HostError::Wizard { source } = err {
        envelope = envelope.subcode(source.subcode());
    }
    if matches!(err, HostError::SessionTimeout { .. }) {
        envelope = envelope.subcode("session_timeout");
    }
    for step in recovery {
        envelope = envelope.recovery(step);
    }
    envelope.to_json_string().map_err(EmitError::Serialize)
}

/// Serialize a wizard lint stage failure as stderr JSON.
///
/// Maps [`WizardLintStageError`] variants to `IoError`, `ParseError`, or
/// `ValidationError` with distinct subcodes and recovery steps (RBP-F002).
///
/// # Errors
///
/// Returns [`EmitError::Serialize`] when the envelope cannot be serialized.
pub fn emit_wizard_lint_stage_error(
    err: &crate::wizard_cmd::WizardLintStageError,
) -> Result<String, EmitError> {
    use crate::wizard_cmd::WizardLintStageError;
    const DOCS: &str =
        ".claude/skills/creating-wyvern-wizard/references/core/validation-and-lint.md";
    let (code, message, cause, recovery, field) = match err {
        WizardLintStageError::Io { path, message } => (
            ErrorCode::IoError,
            message.clone(),
            format!("wyvern wizard lint could not read '{}'", path.display()),
            vec![
                "Verify the path contains wizard.json and all referenced pages exist".into(),
                "Run `wyvern wizard lint --help` for usage".into(),
            ],
            None,
        ),
        WizardLintStageError::Parse { path, message } => (
            ErrorCode::ParseError,
            message.clone(),
            format!("wizard.json at '{}' is not valid JSON", path.display()),
            vec![
                "Ensure wizard.json is valid JSON".into(),
                "Check for trailing commas, unquoted keys, or truncated input".into(),
                "Run `wyvern wizard lint --help` for usage".into(),
            ],
            None,
        ),
        WizardLintStageError::Validation {
            path,
            field,
            message,
        } => (
            ErrorCode::ValidationError,
            message.clone(),
            format!(
                "wizard.json at '{}' failed field checks on '{field}'",
                path.display()
            ),
            vec![
                format!("Fix field '{field}' to a non-empty string"),
                "page.id and page.html must be non-empty".into(),
                "Run `wyvern wizard lint --help` for usage".into(),
            ],
            Some(field.clone()),
        ),
    };
    let mut envelope = StderrError::new(code, message)
        .subcode(err.subcode())
        .cause(cause)
        .docs(DOCS);
    if let Some(field) = field {
        envelope = envelope.field(field);
    }
    for step in recovery {
        envelope = envelope.recovery(step);
    }
    envelope.to_json_string().map_err(EmitError::Serialize)
}

/// Serialize a workflow / chain failure as stderr JSON (`WORKFLOW_ERROR`, exit 9).
///
/// Always uses [`ErrorCode::WorkflowError`] — no hand-built slug.
///
/// # Errors
///
/// Returns [`EmitError::Serialize`] when the envelope cannot be serialized.
pub fn emit_workflow_error(err: &crate::workflow::WorkflowError) -> Result<String, EmitError> {
    let mut envelope = StderrError::new(ErrorCode::WorkflowError, err.to_string())
        .cause(err.cause())
        .subcode(err.subcode())
        .docs("docs/plans/phase-G/wizard-workflow-architecture.md");
    for step in err.recovery() {
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
