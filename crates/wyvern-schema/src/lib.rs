//! Wyvern JSON types, validation, and protocol results.
//!
//! Phase B executable surface (through b.7): [`Command::Chrome`],
//! [`Command::Message`], [`Command::Input`] (text / file / folder),
//! [`Command::Markdown`] (file path or inline `content`), and
//! [`Command::Question`] (card radio/checkbox). Call [`validate`] on
//! loaded JSON before opening a window.

#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable,
        clippy::todo,
        clippy::unimplemented
    )
)]

mod button;
mod chrome;
mod command;
mod error;
mod error_code;
mod field_name;
mod media;
mod result;
mod stderr;
mod validate;

#[doc(inline)]
pub use button::ButtonLabel;
#[doc(inline)]
pub use chrome::{ChromeStatus, ChromeTitle};
#[doc(inline)]
pub use command::{ButtonsPreset, Command, InputMode, MessageLevel, QuestionCard, QuestionOption};
#[doc(inline)]
pub use error::ValidationError;
#[doc(inline)]
pub use error_code::ErrorCode;
#[doc(inline)]
pub use field_name::{FieldName, FieldNameError};
#[doc(inline)]
pub use media::MediaRef;
#[doc(inline)]
pub use result::{
    ChromeResult, CommandResult, InputResult, InputValue, MarkdownResult, MessageResult,
    QuestionResult,
};
#[doc(inline)]
pub use stderr::{SerializeError, StderrError};
#[doc(inline)]
pub use validate::{validate, MARKDOWN_CONTENT_MAX_BYTES};
