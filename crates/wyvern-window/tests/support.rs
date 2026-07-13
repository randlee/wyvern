//! Shared helpers for `wyvern-window` integration tests.

use wyvern_window::RunError;

/// Opens a blank native window, dismisses it via the API close path, and
/// returns [`Ok(())`] — the same outcome as an OS chrome dismiss.
#[cfg(test)]
pub fn open_blank_window_for_test() -> Result<(), RunError> {
    wyvern_window::open_blank_window(true)
}
