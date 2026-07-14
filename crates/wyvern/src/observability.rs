//! Thin `sc-observability` wrapper for CLI pipeline events.
//!
//! Logging stays in the `wyvern` binary crate only. Normative event keys are
//! emitted as `action` names; see `docs/observability.md`.

use std::sync::{Arc, OnceLock};

use sc_observability::{
    ActionName, ConsoleSink, Level, LogEvent, Logger, LoggerConfig, OutcomeLabel, ProcessIdentity,
    SchemaVersion, ServiceName, SinkRegistration, TargetCategory, Timestamp,
    OBSERVATION_ENVELOPE_VERSION,
};
use serde_json::{json, Map, Value};

/// Env var that sets the minimum log level (`off`/`error`/`warn`/`info`/`debug`/`trace`).
pub const WYVERN_LOG_ENV: &str = "WYVERN_LOG";

const SERVICE: &str = "wyvern";
const TARGET: &str = "wyvern.pipeline";

static LOGGER: OnceLock<Logger> = OnceLock::new();
static SERVICE_NAME: OnceLock<ServiceName> = OnceLock::new();

/// Initialize structured logging from [`WYVERN_LOG_ENV`].
///
/// When unset or `off`, no logger is installed and subsequent `log_*` calls are
/// no-ops. Otherwise a stderr console sink is registered so JSON stdout stays clean.
///
/// # Errors
///
/// Returns an error string when the env value is invalid or logger construction fails.
pub fn init() -> Result<(), String> {
    let raw = match std::env::var(WYVERN_LOG_ENV) {
        Ok(value) => value,
        Err(std::env::VarError::NotPresent) => return Ok(()),
        Err(err) => return Err(format!("{WYVERN_LOG_ENV} is not valid Unicode: {err}")),
    };

    if raw.eq_ignore_ascii_case("off") {
        return Ok(());
    }

    let service = ServiceName::new(SERVICE).map_err(|e| e.to_string())?;
    let log_root = std::env::temp_dir().join("wyvern-observability");
    let mut config = LoggerConfig::default_for(service.clone(), log_root);
    apply_level(&mut config, &raw)?;
    // Built-in console sink writes stdout and would pollute JSON protocol output.
    config.enable_console_sink = false;
    config.enable_file_sink = true;

    let mut builder = Logger::builder(config).map_err(|e| e.to_string())?;
    builder.register_sink(SinkRegistration::new(Arc::new(ConsoleSink::stderr())));
    let logger = builder.build();

    let _ = SERVICE_NAME.set(service);
    let _ = LOGGER.set(logger);
    Ok(())
}

/// Emit normative `process_start` after successful [`init`].
pub fn log_process_start() {
    emit("process_start", Level::Info, Some("ok"), Map::new());
}

/// Emit normative `command_received` with a redacted command shape summary.
pub fn log_command_received(value: &Value) {
    let mut fields = Map::new();
    if let Some(type_name) = value.get("type").and_then(Value::as_str) {
        fields.insert("command_type".to_string(), json!(type_name));
    }
    emit("command_received", Level::Info, Some("ok"), fields);
}

/// Emit normative `validation_result` for pass/fail.
pub fn log_validation_result(ok: bool) {
    let mut fields = Map::new();
    fields.insert("ok".to_string(), json!(ok));
    let outcome = if ok { "ok" } else { "error" };
    emit("validation_result", Level::Info, Some(outcome), fields);
}

/// Emit normative `window_open` before dialog delivery (host from c.10).
pub fn log_window_open() {
    emit("window_open", Level::Info, Some("ok"), Map::new());
}

/// Emit normative `window_close` after a successful window run.
pub fn log_window_close() {
    emit("window_close", Level::Info, Some("ok"), Map::new());
}

/// Emit normative `result_emitted` before writing stdout JSON.
pub fn log_result_emitted() {
    emit("result_emitted", Level::Info, Some("ok"), Map::new());
}

/// Emit normative `error` for a pipeline stage failure.
pub fn log_error(stage: &str, detail: &str) {
    let mut fields = Map::new();
    fields.insert("stage".to_string(), json!(stage));
    fields.insert("detail".to_string(), json!(detail));
    emit("error", Level::Error, Some("error"), fields);
}

fn apply_level(config: &mut LoggerConfig, raw: &str) -> Result<(), String> {
    // `LevelFilter` is not re-exported by `sc-observability`; deserialize into the field.
    config.level = match raw.to_ascii_lowercase().as_str() {
        "trace" => serde_json::from_value(json!("Trace")),
        "debug" => serde_json::from_value(json!("Debug")),
        "info" => serde_json::from_value(json!("Info")),
        "warn" | "warning" => serde_json::from_value(json!("Warn")),
        "error" => serde_json::from_value(json!("Error")),
        other => {
            return Err(format!(
                "invalid {WYVERN_LOG_ENV}={other:?}; expected off|error|warn|info|debug|trace"
            ));
        }
    }
    .map_err(|e| format!("failed to parse log level: {e}"))?;
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
