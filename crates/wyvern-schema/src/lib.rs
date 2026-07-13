//! Wyvern JSON types, validation, and protocol results.
//!
//! Phase A executable surface is [`Command::Chrome`] only. Call [`validate`]
//! on loaded JSON before opening a window.

mod command;
mod error;
mod result;
mod validate;

#[doc(inline)]
pub use command::Command;
#[doc(inline)]
pub use error::ValidationError;
#[doc(inline)]
pub use result::{ChromeResult, CommandResult};
#[doc(inline)]
pub use validate::validate;
