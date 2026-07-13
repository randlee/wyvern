//! Typed command surface for the current phase.

/// Executable command after successful validation.
///
/// Phase A accepts only [`Command::Chrome`]. Later phases add variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Foundation chrome frame: required `title`, optional `status`.
    Chrome {
        title: String,
        status: Option<String>,
    },
}
