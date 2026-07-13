//! CLI pipeline: validate → run → emit (load stays in `main` / callers).

use serde_json::Value;

use crate::error::{emit_stdout, emit_validation_error, handle_run_failure};

/// Validate `value`, run the window, and return stdout JSON on success.
///
/// # Errors
///
/// On validation or run failure, returns `(stderr_json, exit_code)` with
/// `exit_code != 0`.
pub fn run_from_loaded(value: Value) -> Result<String, (String, i32)> {
    let command = wyvern_schema::validate(&value).map_err(|e| (emit_validation_error(&e), 1))?;
    let result = wyvern_window::run(command).map_err(|e| handle_run_failure(&e))?;
    Ok(emit_stdout(&result))
}
