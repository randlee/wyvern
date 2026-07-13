//! CLI pipeline: validate → run → emit (load stays in `main` / callers).

use serde_json::Value;

use crate::error::{emit_stdout, emit_validation_error, handle_run_failure};
use crate::observability;

/// Validate `value`, run the window, and return stdout JSON on success.
///
/// # Errors
///
/// On validation or run failure, returns `(stderr_json, exit_code)` with
/// `exit_code != 0`.
pub fn run_from_loaded(value: Value) -> Result<String, (String, i32)> {
    observability::log_command_received(&value);
    let command = match wyvern_schema::validate(&value) {
        Ok(cmd) => {
            observability::log_validation_result(true);
            cmd
        }
        Err(e) => {
            observability::log_validation_result(false);
            observability::log_error("validate", &format!("{e:?}"));
            return Err((emit_validation_error(&e), e.exit_code()));
        }
    };
    observability::log_window_open();
    let result = match wyvern_window::run(command) {
        Ok(r) => {
            observability::log_window_close();
            r
        }
        Err(e) => {
            observability::log_error("run", &format!("{e:?}"));
            return Err(handle_run_failure(&e));
        }
    };
    observability::log_result_emitted();
    Ok(emit_stdout(&result))
}
