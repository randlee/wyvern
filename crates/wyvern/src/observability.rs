//! Pipeline observability hooks via the `tracing` facade.
//!
//! Library code must not import `sc_observability` (RULE-001). The binary
//! entrypoint installs a subscriber / backend; these helpers only emit
//! structured `tracing` events.

use std::sync::OnceLock;

use serde_json::Value;

static PIPELINE_CORRELATION_ID: OnceLock<String> = OnceLock::new();

/// Bind the session correlation id for pipeline `tracing` events (binary calls after init).
pub fn set_pipeline_correlation_id(id: impl Into<String>) {
    let _ = PIPELINE_CORRELATION_ID.set(id.into());
}

fn pipeline_correlation_id() -> Option<&'static str> {
    PIPELINE_CORRELATION_ID.get().map(String::as_str)
}

macro_rules! pipeline_event {
    ($level:ident, $($field:tt)*) => {
        if let Some(correlation_id) = pipeline_correlation_id() {
            tracing::$level!(correlation_id, $($field)*);
        } else {
            tracing::$level!($($field)*);
        }
    };
}

/// Emit normative `command_received` with a redacted command shape summary.
pub fn log_command_received(value: &Value) {
    let command_type = value.get("type").and_then(Value::as_str);
    pipeline_event!(
        info,
        action = "command_received",
        command_type,
        "command_received"
    );
}

/// Emit normative `validation_result` for pass/fail.
pub fn log_validation_result(ok: bool) {
    let outcome = if ok { "ok" } else { "error" };
    pipeline_event!(
        info,
        action = "validation_result",
        ok,
        outcome,
        "validation_result"
    );
}

/// Emit normative `host_start` before dialog delivery.
pub fn log_host_start(command_type: &str) {
    pipeline_event!(info, action = "host_start", command_type, "host_start");
}

/// Emit normative `host_result` after the host returns.
pub fn log_host_result(ok: bool) {
    let outcome = if ok { "ok" } else { "error" };
    pipeline_event!(info, action = "host_result", ok, outcome, "host_result");
}

/// Emit normative `error` for a pipeline stage failure.
pub fn log_error(stage: &str, detail: &str) {
    pipeline_event!(error, action = "error", stage, detail, "error");
}
