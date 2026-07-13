//! Shared helpers for `wyvern-window` integration tests.

#![allow(dead_code)]

use wyvern_schema::{
    ButtonLabel, ButtonsPreset, ChromeResult, ChromeTitle, Command, CommandResult, InputMode,
    InputResult, InputValue, MarkdownResult, MessageResult, QuestionCard, QuestionOption,
    QuestionResult,
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

/// Opens chrome via [`wyvern_window::run`], injects IPC, and returns the result.
pub fn open_chrome_with_injected_ipc(ipc_json: &str) -> Result<CommandResult, RunError> {
    // SAFETY: integration test harness runs single-threaded before other work.
    unsafe {
        std::env::remove_var("WYVERN_AUTO_DISMISS");
        std::env::set_var("WYVERN_INJECT_IPC", ipc_json);
    }
    let result = wyvern_window::run(Command::Chrome {
        title: ChromeTitle::new("wyvern-chrome-ipc-test"),
        status: None,
    });
    unsafe { std::env::remove_var("WYVERN_INJECT_IPC") };
    result
}

/// Opens chrome, injects IPC, then auto-dismisses (for non-completing IPC like minimize).
pub fn open_chrome_inject_then_auto_dismiss(ipc_json: &str) -> Result<CommandResult, RunError> {
    // SAFETY: integration test harness runs single-threaded before other work.
    unsafe {
        std::env::set_var("WYVERN_AUTO_DISMISS", "1");
        std::env::set_var("WYVERN_INJECT_IPC", ipc_json);
    }
    let result = wyvern_window::run(Command::Chrome {
        title: ChromeTitle::new("wyvern-chrome-minimize-test"),
        status: None,
    });
    unsafe {
        std::env::remove_var("WYVERN_INJECT_IPC");
        std::env::remove_var("WYVERN_AUTO_DISMISS");
    }
    result
}

/// Opens a message dialog, injects IPC, then auto-dismisses (non-completing IPC).
pub fn open_message_inject_then_auto_dismiss(ipc_json: &str) -> Result<CommandResult, RunError> {
    // SAFETY: integration test harness runs single-threaded before other work.
    unsafe {
        std::env::set_var("WYVERN_AUTO_DISMISS", "1");
        std::env::set_var("WYVERN_INJECT_IPC", ipc_json);
    }
    let result = wyvern_window::run(Command::Message {
        title: ChromeTitle::new("wyvern-message-minimize-test"),
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
    unsafe {
        std::env::remove_var("WYVERN_INJECT_IPC");
        std::env::remove_var("WYVERN_AUTO_DISMISS");
    }
    result
}

/// Opens an input dialog, injects IPC, then auto-dismisses (non-completing IPC).
pub fn open_input_inject_then_auto_dismiss(ipc_json: &str) -> Result<CommandResult, RunError> {
    // SAFETY: integration test harness runs single-threaded before other work.
    unsafe {
        std::env::set_var("WYVERN_AUTO_DISMISS", "1");
        std::env::set_var("WYVERN_INJECT_IPC", ipc_json);
    }
    let result = wyvern_window::run(Command::Input {
        title: ChromeTitle::new("wyvern-input-minimize-test"),
        message: "Enter a value".into(),
        status: None,
        icon: None,
        markdown: false,
        multiline: false,
        placeholder: Some("hint".into()),
        default: Some("prefill".into()),
        mode: InputMode::Text,
        filter: None,
        multiple: false,
        start_path: None,
        buttons: ButtonsPreset::OkCancel,
    });
    unsafe {
        std::env::remove_var("WYVERN_INJECT_IPC");
        std::env::remove_var("WYVERN_AUTO_DISMISS");
    }
    result
}

/// Opens a markdown dialog, injects IPC, then auto-dismisses (non-completing IPC).
pub fn open_markdown_inject_then_auto_dismiss(ipc_json: &str) -> Result<CommandResult, RunError> {
    // SAFETY: integration test harness runs single-threaded before other work.
    unsafe {
        std::env::set_var("WYVERN_AUTO_DISMISS", "1");
        std::env::set_var("WYVERN_INJECT_IPC", ipc_json);
    }
    let result = wyvern_window::run(Command::Markdown {
        title: Some(ChromeTitle::new("wyvern-markdown-minimize-test")),
        file: None,
        content: Some("# Hello\n\nMinimize no-op body.".into()),
        status: None,
        buttons: ButtonsPreset::Ok,
    });
    unsafe {
        std::env::remove_var("WYVERN_INJECT_IPC");
        std::env::remove_var("WYVERN_AUTO_DISMISS");
    }
    result
}

/// Opens a question dialog, injects IPC, then auto-dismisses (non-completing IPC).
pub fn open_question_inject_then_auto_dismiss(ipc_json: &str) -> Result<CommandResult, RunError> {
    // SAFETY: integration test harness runs single-threaded before other work.
    unsafe {
        std::env::set_var("WYVERN_AUTO_DISMISS", "1");
        std::env::set_var("WYVERN_INJECT_IPC", ipc_json);
    }
    let result = wyvern_window::run(Command::Question {
        questions: vec![QuestionCard {
            question: "Output format?".into(),
            header: "Format".into(),
            options: vec![
                QuestionOption {
                    label: "JSON".into(),
                    description: "Structured".into(),
                    preview: None,
                },
                QuestionOption {
                    label: "Plain".into(),
                    description: "Text only".into(),
                    preview: None,
                },
            ],
            multi_select: false,
        }],
        questions_raw: vec![serde_json::json!({
            "question": "Output format?",
            "header": "Format",
            "options": [
                { "label": "JSON", "description": "Structured" },
                { "label": "Plain", "description": "Text only" }
            ],
            "multiSelect": false
        })],
    });
    unsafe {
        std::env::remove_var("WYVERN_INJECT_IPC");
        std::env::remove_var("WYVERN_AUTO_DISMISS");
    }
    result
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
        filter: None,
        multiple: false,
        start_path: None,
        buttons: ButtonsPreset::OkCancel,
    });
    unsafe { std::env::remove_var("WYVERN_INJECT_IPC") };
    result
}

/// Opens a file-mode input dialog, mocks the picker path, and injects OK IPC.
pub fn open_file_picker_with_mock(
    mock_path: &str,
    multiple: bool,
) -> Result<CommandResult, RunError> {
    // SAFETY: integration test harness runs single-threaded before other work.
    unsafe {
        std::env::remove_var("WYVERN_AUTO_DISMISS");
        std::env::set_var("WYVERN_MOCK_PICKER_PATH", mock_path);
        std::env::set_var(
            "WYVERN_INJECT_IPC",
            r#"{"kind":"input_submitted","button":"ok"}"#,
        );
    }
    let result = wyvern_window::run(Command::Input {
        title: ChromeTitle::new("wyvern-file-picker-test"),
        message: "Choose a file".into(),
        status: None,
        icon: None,
        markdown: false,
        multiline: false,
        placeholder: None,
        default: None,
        mode: InputMode::File,
        filter: Some(vec!["*.txt".into()]),
        multiple,
        start_path: Some("/tmp".into()),
        buttons: ButtonsPreset::OkCancel,
    });
    unsafe {
        std::env::remove_var("WYVERN_INJECT_IPC");
        std::env::remove_var("WYVERN_MOCK_PICKER_PATH");
    }
    result
}

/// Opens a folder-mode input dialog, mocks the picker path, and injects OK IPC.
pub fn open_folder_picker_with_mock(mock_path: &str) -> Result<CommandResult, RunError> {
    // SAFETY: integration test harness runs single-threaded before other work.
    unsafe {
        std::env::remove_var("WYVERN_AUTO_DISMISS");
        std::env::set_var("WYVERN_MOCK_PICKER_PATH", mock_path);
        std::env::set_var(
            "WYVERN_INJECT_IPC",
            r#"{"kind":"input_submitted","button":"ok"}"#,
        );
    }
    let result = wyvern_window::run(Command::Input {
        title: ChromeTitle::new("wyvern-folder-picker-test"),
        message: "Choose a folder".into(),
        status: None,
        icon: None,
        markdown: false,
        multiline: false,
        placeholder: None,
        default: None,
        mode: InputMode::Folder,
        filter: None,
        multiple: false,
        start_path: Some("/tmp".into()),
        buttons: ButtonsPreset::OkCancel,
    });
    unsafe {
        std::env::remove_var("WYVERN_INJECT_IPC");
        std::env::remove_var("WYVERN_MOCK_PICKER_PATH");
    }
    result
}

/// Opens a question dialog and injects IPC JSON (test harness hook).
pub fn open_question_with_injected_ipc(ipc_json: &str) -> Result<CommandResult, RunError> {
    open_question_cards_with_injected_ipc(
        vec![QuestionCard {
            question: "Output format?".into(),
            header: "Format".into(),
            options: vec![
                QuestionOption {
                    label: "JSON".into(),
                    description: "Structured".into(),
                    preview: None,
                },
                QuestionOption {
                    label: "Plain".into(),
                    description: "Text only".into(),
                    preview: None,
                },
            ],
            multi_select: false,
        }],
        vec![serde_json::json!({
            "question": "Output format?",
            "header": "Format",
            "options": [
                { "label": "JSON", "description": "Structured" },
                { "label": "Plain", "description": "Text only" }
            ],
            "multiSelect": false
        })],
        ipc_json,
    )
}

/// Opens a multi-select question dialog and injects IPC JSON (test harness hook).
pub fn open_multi_select_question_with_injected_ipc(
    ipc_json: &str,
) -> Result<CommandResult, RunError> {
    open_question_cards_with_injected_ipc(
        vec![QuestionCard {
            question: "Pick tools".into(),
            header: "Tools".into(),
            options: vec![
                QuestionOption {
                    label: "JSON".into(),
                    description: "A".into(),
                    preview: None,
                },
                QuestionOption {
                    label: "Plain".into(),
                    description: "B".into(),
                    preview: None,
                },
            ],
            multi_select: true,
        }],
        vec![serde_json::json!({
            "question": "Pick tools",
            "header": "Tools",
            "options": [
                { "label": "JSON", "description": "A" },
                { "label": "Plain", "description": "B" }
            ],
            "multiSelect": true
        })],
        ipc_json,
    )
}

fn open_question_cards_with_injected_ipc(
    questions: Vec<QuestionCard>,
    questions_raw: Vec<serde_json::Value>,
    ipc_json: &str,
) -> Result<CommandResult, RunError> {
    // SAFETY: integration test harness runs single-threaded before other work.
    unsafe {
        std::env::remove_var("WYVERN_AUTO_DISMISS");
        std::env::set_var("WYVERN_INJECT_IPC", ipc_json);
    }
    let result = wyvern_window::run(Command::Question {
        questions,
        questions_raw,
    });
    unsafe { std::env::remove_var("WYVERN_INJECT_IPC") };
    result
}

/// Assert helper: dismissed chrome/message/markdown/input yields `{ "button": "dismissed" }`.
#[allow(dead_code)]
pub fn assert_dismissed(result: &CommandResult) {
    let button = match result {
        CommandResult::Chrome(ChromeResult { button }) => button,
        CommandResult::Message(MessageResult { button }) => button,
        CommandResult::Markdown(MarkdownResult { button }) => button,
        CommandResult::Input(InputResult { button, .. }) => button,
        CommandResult::Question(QuestionResult { button, .. }) => button
            .as_ref()
            .expect("question dismiss must include button"),
    };
    assert_eq!(button.as_str(), "dismissed");
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

/// Assert helper: multi-select file paths.
#[allow(dead_code)]
pub fn assert_input_paths(result: &CommandResult, expected_button: &str, expected: &[&str]) {
    match result {
        CommandResult::Input(InputResult { button, input }) => {
            assert_eq!(button, &ButtonLabel::new(expected_button));
            match input {
                Some(InputValue::Paths(got)) => {
                    assert_eq!(got.as_slice(), expected);
                }
                other => panic!("expected Paths, got {other:?}"),
            }
        }
        other => panic!("expected Input result, got {other:?}"),
    }
}

/// Assert helper: question submit without button field.
#[allow(dead_code)]
pub fn assert_question_submitted(
    result: &CommandResult,
    expected_answer_key: &str,
    expected: &str,
) {
    match result {
        CommandResult::Question(QuestionResult {
            button,
            answers,
            response,
            ..
        }) => {
            assert!(button.is_none(), "normal completion must omit button");
            assert_eq!(
                answers.get(expected_answer_key).map(String::as_str),
                Some(expected)
            );
            assert_eq!(response, "");
        }
        other => panic!("expected Question result, got {other:?}"),
    }
}

/// Assert helper: question dismiss includes REQ-0068 `button: "dismissed"`.
#[allow(dead_code)]
pub fn assert_question_dismissed(result: &CommandResult) {
    match result {
        CommandResult::Question(QuestionResult {
            button,
            answers,
            response,
            questions,
        }) => {
            assert_eq!(button.as_ref().map(ButtonLabel::as_str), Some("dismissed"));
            assert!(answers.is_empty());
            assert_eq!(response, "");
            assert!(!questions.is_empty());
        }
        other => panic!("expected Question dismissed result, got {other:?}"),
    }
}

/// Opens a question dialog and auto-dismisses (OS-close / REQ-0068 path).
pub fn open_question_auto_dismiss() -> Result<CommandResult, RunError> {
    // SAFETY: integration test harness runs single-threaded before other work.
    unsafe {
        std::env::remove_var("WYVERN_INJECT_IPC");
        std::env::set_var("WYVERN_AUTO_DISMISS", "1");
    }
    let result = wyvern_window::run(Command::Question {
        questions: vec![QuestionCard {
            question: "Output format?".into(),
            header: "Format".into(),
            options: vec![
                QuestionOption {
                    label: "JSON".into(),
                    description: "Structured".into(),
                    preview: Some(r#"<pre>{"ok":true}</pre>"#.into()),
                },
                QuestionOption {
                    label: "Plain".into(),
                    description: "Text only".into(),
                    preview: None,
                },
            ],
            multi_select: false,
        }],
        questions_raw: vec![serde_json::json!({
            "question": "Output format?",
            "header": "Format",
            "options": [
                {
                    "label": "JSON",
                    "description": "Structured",
                    "preview": "<pre>{\"ok\":true}</pre>"
                },
                { "label": "Plain", "description": "Text only" }
            ],
            "multiSelect": false
        })],
    });
    unsafe { std::env::remove_var("WYVERN_AUTO_DISMISS") };
    result
}
