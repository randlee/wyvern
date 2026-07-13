//! Shared helpers for `wyvern-window` integration tests.

use wyvern_schema::{ChromeResult, ChromeTitle, Command, CommandResult};
use wyvern_window::RunError;

/// Opens chrome via [`wyvern_window::run`], auto-dismisses, and returns the
/// dismissed protocol result — the same outcome as an OS chrome close.
pub fn open_blank_window_for_test() -> Result<CommandResult, RunError> {
    // SAFETY: integration test harness runs single-threaded before other work.
    unsafe { std::env::set_var("WYVERN_AUTO_DISMISS", "1") };
    wyvern_window::run(Command::Chrome {
        title: ChromeTitle::new("wyvern-test"),
        status: None,
    })
}

/// Assert helper: dismissed chrome yields the Phase A wire shape.
#[allow(dead_code)]
pub fn assert_dismissed(result: &CommandResult) {
    match result {
        CommandResult::Chrome(ChromeResult { button }) => {
            assert_eq!(button, "dismissed");
        }
    }
}
