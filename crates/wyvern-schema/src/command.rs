//! Typed command surface for the current phase.

use crate::chrome::{ChromeStatus, ChromeTitle};

/// Executable command after successful validation.
///
/// Phase A accepts only [`Command::Chrome`]. Later phases add variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Foundation chrome frame: required `title`, optional `status`.
    Chrome {
        title: ChromeTitle,
        status: Option<ChromeStatus>,
    },
}
