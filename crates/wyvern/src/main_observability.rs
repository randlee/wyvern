//! Binary-only `sc-observability` init (RULE-001: not part of the library).

use std::sync::{Arc, OnceLock};

use sc_observability::{
    ActionName, ConsoleSink, Level, LogEvent, Logger, LoggerConfig, OutcomeLabel, ProcessIdentity,
    SchemaVersion, ServiceName, SinkRegistration, TargetCategory, Timestamp,
    OBSERVATION_ENVELOPE_VERSION,
};
use serde_json::{json, Map, Value};
use tracing_subscriber::EnvFilter;

/// Env var that sets the minimum log level (`off`/`error`/`warn`/`info`/`debug`/`trace`).
pub const WYVERN_LOG_ENV: &str = "WYVERN_LOG";

const SERVICE: &str = "wyvern";
const TARGET: &str = "wyvern.pipeline";
const OBSERVABILITY_DOCS: &str = "docs/observability.md";

static LOGGER: OnceLock<Logger> = OnceLock::new();
static SERVICE_NAME: OnceLock<ServiceName> = OnceLock::new();

/// Structured failure from [`init`] with recovery steps (RBP-F004).
#[derive(Debug, Clone)]
pub struct ObservabilityInitError {
    /// Human-readable failure summary.
    pub message: String,
    /// Underlying cause when available.
    pub cause: Option<String>,
    /// Actionable recovery steps.
    pub recovery: Vec<&'static str>,
}

impl std::fmt::Display for ObservabilityInitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ObservabilityInitError {}

impl ObservabilityInitError {
    fn new(message: impl Into<String>, cause: Option<String>) -> Self {
        Self {
            message: message.into(),
            cause,
            recovery: vec![
                "Unset WYVERN_LOG to disable structured logging, or set WYVERN_LOG=off",
                "Use a valid level: off|error|warn|info|debug|trace",
                "Ensure the process can write under the system temp dir for file sinks",
            ],
        }
    }
}

/// Initialize structured logging from [`WYVERN_LOG_ENV`].
///
/// When unset or `off`, no logger is installed and subsequent `log_*` calls are
/// no-ops. Otherwise a stderr console sink is registered so JSON stdout stays clean,
/// and a `tracing` subscriber is installed so the library pipeline facade emits.
///
/// # Errors
///
/// Returns [`ObservabilityInitError`] when the env value is invalid or logger
/// construction fails.
pub fn init() -> Result<(), ObservabilityInitError> {
    let raw = match std::env::var(WYVERN_LOG_ENV) {
        Ok(value) => value,
        Err(std::env::VarError::NotPresent) => return Ok(()),
        Err(err) => {
            return Err(ObservabilityInitError::new(
                format!("{WYVERN_LOG_ENV} is not valid Unicode"),
                Some(err.to_string()),
            ));
        }
    };

    if raw.eq_ignore_ascii_case("off") {
        return Ok(());
    }

    let service = ServiceName::new(SERVICE).map_err(|e| {
        ObservabilityInitError::new(
            "failed to construct observability ServiceName",
            Some(e.to_string()),
        )
    })?;
    let log_root = std::env::temp_dir().join("wyvern-observability");
    let mut config = LoggerConfig::default_for(service.clone(), log_root);
    apply_level(&mut config, &raw)?;
    // Built-in console sink writes stdout and would pollute JSON protocol output.
    config.enable_console_sink = false;
    config.enable_file_sink = true;

    let mut builder = Logger::builder(config).map_err(|e| {
        ObservabilityInitError::new("failed to build observability Logger", Some(e.to_string()))
    })?;
    builder.register_sink(SinkRegistration::new(Arc::new(ConsoleSink::stderr())));
    let logger = builder.build();

    let _ = SERVICE_NAME.set(service);
    let _ = LOGGER.set(logger);

    // Library pipeline uses the tracing facade; bridge to stderr when enabled.
    let filter = EnvFilter::try_new(format!("wyvern={raw},wyvern_host={raw}"))
        .or_else(|_| EnvFilter::try_new("info"))
        .map_err(|e| ObservabilityInitError::new("invalid tracing filter", Some(e.to_string())))?;
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_target(true)
        .try_init();

    Ok(())
}

/// Emit normative `process_start` after successful [`init`].
pub fn log_process_start() {
    emit("process_start", Level::Info, Some("ok"), Map::new());
}

/// Emit a structured stderr envelope for an [`ObservabilityInitError`] (best-effort).
pub fn emit_init_error(err: &ObservabilityInitError) {
    use wyvern_schema::{ErrorCode, StderrError};

    let mut envelope = StderrError::new(ErrorCode::InternalError, err.message.clone())
        .docs(OBSERVABILITY_DOCS.to_string());
    if let Some(cause) = &err.cause {
        envelope = envelope.cause(cause.clone());
    }
    for step in &err.recovery {
        envelope = envelope.recovery((*step).to_string());
    }
    match envelope.to_json_string() {
        Ok(json) => eprintln!("{json}"),
        Err(_) => eprintln!("wyvern: {}", err.message),
    }
}

fn apply_level(config: &mut LoggerConfig, raw: &str) -> Result<(), ObservabilityInitError> {
    // `LevelFilter` is not re-exported by `sc-observability`; deserialize into the field.
    config.level = match raw.to_ascii_lowercase().as_str() {
        "trace" => serde_json::from_value(json!("Trace")),
        "debug" => serde_json::from_value(json!("Debug")),
        "info" => serde_json::from_value(json!("Info")),
        "warn" | "warning" => serde_json::from_value(json!("Warn")),
        "error" => serde_json::from_value(json!("Error")),
        other => {
            return Err(ObservabilityInitError::new(
                format!(
                    "invalid {WYVERN_LOG_ENV}={other:?}; expected off|error|warn|info|debug|trace"
                ),
                None,
            ));
        }
    }
    .map_err(|e| ObservabilityInitError::new("failed to parse log level", Some(e.to_string())))?;
    Ok(())
}

fn emit(action: &str, level: Level, outcome: Option<&str>, fields: Map<String, Value>) {
    let Some(logger) = LOGGER.get() else {
        return;
    };
    let Some(service) = SERVICE_NAME.get() else {
        return;
    };

    let event = match build_event(service.clone(), action, level, outcome, fields) {
        Ok(event) => event,
        Err(err) => {
            eprintln!("wyvern: observability event build failed: {err}");
            return;
        }
    };
    let _ = logger.try_log(event);
    // CLI exits quickly; flush so stderr console events are visible before teardown.
    let _ = logger.flush();
}

fn build_event(
    service: ServiceName,
    action: &str,
    level: Level,
    outcome: Option<&str>,
    fields: Map<String, Value>,
) -> Result<LogEvent, String> {
    Ok(LogEvent {
        version: SchemaVersion::new(OBSERVATION_ENVELOPE_VERSION).map_err(|e| e.to_string())?,
        timestamp: Timestamp::now_utc(),
        level,
        service,
        target: TargetCategory::new(TARGET).map_err(|e| e.to_string())?,
        action: ActionName::new(action).map_err(|e| e.to_string())?,
        message: Some(action.to_string()),
        identity: ProcessIdentity::default(),
        trace: None,
        request_id: None,
        correlation_id: None,
        outcome: match outcome {
            Some(label) => Some(OutcomeLabel::new(label).map_err(|e| e.to_string())?),
            None => None,
        },
        diagnostic: None,
        state_transition: None,
        fields,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_level_accepts_debug() {
        let service = ServiceName::new("wyvern-test").expect("service");
        let mut config =
            LoggerConfig::default_for(service, std::env::temp_dir().join("wyvern-test-logs"));
        apply_level(&mut config, "debug").expect("debug level");
        let encoded = serde_json::to_value(config.level).expect("serialize level");
        assert_eq!(encoded, json!("Debug"));
    }

    #[test]
    fn apply_level_rejects_unknown_with_recovery() {
        let service = ServiceName::new("wyvern-test").expect("service");
        let mut config =
            LoggerConfig::default_for(service, std::env::temp_dir().join("wyvern-test-logs"));
        let err = apply_level(&mut config, "nope").expect_err("level");
        assert!(!err.recovery.is_empty());
        assert!(err.message.contains("WYVERN_LOG"));
    }

    #[test]
    fn build_event_uses_normative_action() {
        let service = ServiceName::new("wyvern-test").expect("service");
        let event = build_event(
            service,
            "process_start",
            Level::Info,
            Some("ok"),
            Map::new(),
        )
        .expect("event");
        assert_eq!(event.action.as_str(), "process_start");
        assert_eq!(event.target.as_str(), TARGET);
    }
}
