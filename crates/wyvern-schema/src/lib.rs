//! Wyvern JSON types, validation, and protocol results.
//!
//! Phase B executable surface (through b.5): [`Command::Chrome`],
//! [`Command::Message`], [`Command::Input`] (text / file / folder), and
//! [`Command::Markdown`] (file path). Call [`validate`] on loaded JSON before
//! opening a window.

mod button;
mod chrome;
mod command;
mod error;
mod error_code;
mod field_name;
mod result;
mod stderr;
mod validate;

#[doc(inline)]
pub use button::ButtonLabel;
#[doc(inline)]
pub use chrome::{ChromeStatus, ChromeTitle};
#[doc(inline)]
pub use command::{ButtonsPreset, Command, InputMode, MessageLevel};
#[doc(inline)]
pub use error::ValidationError;
#[doc(inline)]
pub use error_code::ErrorCode;
#[doc(inline)]
pub use field_name::FieldName;
#[doc(inline)]
pub use result::{
    ChromeResult, CommandResult, InputResult, InputValue, MarkdownResult, MessageResult,
};
#[doc(inline)]
pub use stderr::StderrError;
#[doc(inline)]
pub use validate::validate;
