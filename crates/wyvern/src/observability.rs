//! Pipeline observability hooks via the `tracing` facade.
//!
//! Library code must not import `sc_observability` (RULE-001). The binary
//! entrypoint installs a subscriber / backend; these helpers only emit
//! structured `tracing` events.

use serde_json::Value;

/// Emit normative `command_received` with a redacted command shape summary.
pub fn log_command_received(value: &Value) {
    let command_type = value.get("type").and_then(Value::as_str);
    tracing::info!(
        action = "command_received",
        command_type,
        "command_received"
    );
}

/// Emit normative `validation_result` for pass/fail.
pub fn log_validation_result(ok: bool) {
    let outcome = if ok { "ok" } else { "error" };
    tracing::info!(
        action = "validation_result",
        ok,
        outcome,
        "validation_result"
    );
}

/// Emit normative `host_start` before dialog delivery.
pub fn log_host_start(command_type: &str) {
    tracing::info!(action = "host_start", command_type, "host_start");
}

/// Emit normative `host_result` after the host returns.
pub fn log_host_result(ok: bool) {
    let outcome = if ok { "ok" } else { "error" };
    tracing::info!(action = "host_result", ok, outcome, "host_result");
}

/// Emit normative `error` for a pipeline stage failure.
pub fn log_error(stage: &str, detail: &str) {
    tracing::error!(action = "error", stage, detail, "error");
}
