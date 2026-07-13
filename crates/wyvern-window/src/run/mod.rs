//! Public `run` entry: open chrome/message/input/markdown/question windows and return protocol results.

mod chrome;
mod input;
mod markdown;
mod message;

use wyvern_schema::{Command, CommandResult};

use crate::error::RunError;
use crate::question::run_question;

use chrome::run_chrome;
use input::{run_input, InputRunArgs};
use markdown::{run_markdown, MarkdownRunArgs};
use message::{run_message, MessageRunArgs};

/// Env var that auto-dismisses the window after successful creation.
///
/// Used by CI and crate tests so GUI paths do not block on interactive close.
pub(super) const AUTO_DISMISS_ENV: &str = "WYVERN_AUTO_DISMISS";

/// Env var that injects a raw IPC JSON body after the window opens (tests).
///
/// Example: `{"kind":"button_pressed","label":"ok"}`.
pub(super) const INJECT_IPC_ENV: &str = "WYVERN_INJECT_IPC";

/// User events from IPC handler / test inject / auto-dismiss.
#[derive(Debug)]
pub(super) enum DialogEvent {
    Ipc(String),
    AutoDismiss,
}

/// Open a native window for `command` and return the protocol result.
///
/// # Errors
///
/// Returns [`RunError::EventLoop`] if the event loop cannot start, or
/// [`RunError::WindowCreate`] if the native window / webview fails to build.
pub fn run(command: Command) -> Result<CommandResult, RunError> {
    match command {
        Command::Chrome { title, status } => run_chrome(title, status),
        Command::Message {
            title,
            message,
            status,
            buttons,
            custom_buttons,
            default_button,
            level,
            icon,
            image,
            markdown,
        } => run_message(MessageRunArgs {
            title,
            message,
            status,
            buttons,
            custom_buttons,
            default_button,
            level,
            icon,
            image,
            markdown,
        }),
        Command::Input {
            title,
            message,
            status,
            icon,
            markdown,
            multiline,
            placeholder,
            default,
            mode,
            filter,
            multiple,
            start_path,
            buttons,
        } => run_input(InputRunArgs {
            title,
            message,
            status,
            icon,
            markdown,
            multiline,
            placeholder,
            default,
            mode,
            filter,
            multiple,
            start_path,
            buttons,
        }),
        Command::Markdown {
            title,
            file: _,
            content,
            status,
            buttons,
        } => {
            let Some(source) = content else {
                return Err(RunError::WindowCreate {
                    message: "markdown content was not loaded before run".into(),
                });
            };
            run_markdown(MarkdownRunArgs {
                title,
                source,
                status,
                buttons,
            })
        }
        Command::Question {
            questions,
            questions_raw,
        } => run_question(questions, questions_raw),
    }
}
