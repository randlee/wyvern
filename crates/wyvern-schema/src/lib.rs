//! Wyvern JSON types, validation, and protocol results.
//!
//! Executable surface: [`Command::Chrome`], [`Command::Message`],
//! [`Command::Input`], [`Command::Markdown`], [`Command::Question`], and
//! [`Command::Wizard`]. Call [`validate`] on loaded JSON before opening a host
//! session.

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
mod wizard;

#[doc(inline)]
pub use button::ButtonLabel;
#[doc(inline)]
pub use chrome::{ChromeStatus, ChromeTitle};
#[doc(inline)]
pub use command::{
    ButtonsPreset, Command, InputMode, MessageLevel, QuestionCard, QuestionOption, WindowSizeHint,
};
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
#[doc(inline)]
pub use wizard::{
    WizardCommand, WizardPageDescriptor, WizardPageLayout, WizardResult, WizardStackEntry,
    WizardStateResponse,
};
