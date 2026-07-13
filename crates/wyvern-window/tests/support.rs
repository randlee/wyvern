//! Shared helpers for `wyvern-window` integration tests.

#![allow(dead_code)]

use wyvern_schema::{
    ButtonLabel, ButtonsPreset, ChromeResult, ChromeTitle, Command, CommandResult, InputMode,
    InputResult, InputValue, MessageResult,
};
use wyvern_window::RunError;

/// Opens chrome via [`wyvern_window::run`], auto-dismisses, and returns the
/// dismissed protocol result — the same outcome as an OS chrome close.
pub fn open_blank_window_for_test() -> Result<CommandResult, RunError> {
    // SAFETY: integration test harness runs single-threaded before other work.
    unsafe { std::env::set_var("WYVERN_AUTO_DISMISS", "1") };
    unsafe { std::env::remove_var("WYVERN_INJECT_IPC") };
    wyvern_window::run(Command::Chrome {
        title: ChromeTitle::new("wyvern-test"),
        status: None,
    })
}

/// Opens a message dialog and injects IPC JSON (test harness hook).
pub fn open_message_with_injected_ipc(ipc_json: &str) -> Result<CommandResult, RunError> {
    // SAFETY: integration test harness runs single-threaded before other work.
    unsafe {
        std::env::remove_var("WYVERN_AUTO_DISMISS");
        std::env::set_var("WYVERN_INJECT_IPC", ipc_json);
    }
    let result = wyvern_window::run(Command::Message {
        title: ChromeTitle::new("wyvern-message-test"),
        message: "Test body".into(),
        status: None,
        buttons: ButtonsPreset::OkCancel,
        custom_buttons: None,
        default_button: Some(0),
        level: None,
        icon: None,
        image: None,
        markdown: false,
    });
    unsafe { std::env::remove_var("WYVERN_INJECT_IPC") };
    result
}

/// Opens an input dialog and injects IPC JSON (test harness hook).
pub fn open_input_with_injected_ipc(ipc_json: &str) -> Result<CommandResult, RunError> {
    // SAFETY: integration test harness runs single-threaded before other work.
    unsafe {
        std::env::remove_var("WYVERN_AUTO_DISMISS");
        std::env::set_var("WYVERN_INJECT_IPC", ipc_json);
    }
    let result = wyvern_window::run(Command::Input {
        title: ChromeTitle::new("wyvern-input-test"),
        message: "Enter a value".into(),
        status: None,
        icon: None,
        markdown: false,
        multiline: false,
        placeholder: Some("hint".into()),
        default: Some("prefill".into()),
        mode: InputMode::Text,
        buttons: ButtonsPreset::OkCancel,
    });
    unsafe { std::env::remove_var("WYVERN_INJECT_IPC") };
    result
}

/// Assert helper: dismissed chrome/message/input yields `{ "button": "dismissed" }`.
#[allow(dead_code)]
pub fn assert_dismissed(result: &CommandResult) {
    match result {
        CommandResult::Chrome(ChromeResult { button })
        | CommandResult::Message(MessageResult { button })
        | CommandResult::Input(InputResult { button, .. }) => {
            assert_eq!(button.as_str(), "dismissed");
        }
    }
}

/// Assert helper: message result button wire label.
#[allow(dead_code)]
pub fn assert_message_button(result: &CommandResult, expected: &str) {
    match result {
        CommandResult::Message(MessageResult { button }) => {
            assert_eq!(button, &ButtonLabel::new(expected));
        }
        other => panic!("expected Message result, got {other:?}"),
    }
}

/// Assert helper: input result wire shape.
#[allow(dead_code)]
pub fn assert_input_result(
    result: &CommandResult,
    expected_button: &str,
    expected_input: Option<&str>,
) {
    match result {
        CommandResult::Input(InputResult { button, input }) => {
            assert_eq!(button, &ButtonLabel::new(expected_button));
            match (input, expected_input) {
                (None, None) => {}
                (Some(InputValue::Text(got)), Some(want)) => assert_eq!(got, want),
                (got, want) => panic!("input mismatch: got={got:?} want={want:?}"),
            }
        }
        other => panic!("expected Input result, got {other:?}"),
    }
}
