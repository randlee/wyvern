//! Protocol result types written to stdout on success.

use serde::Serialize;

use crate::button::ButtonLabel;

/// Successful command result for stdout JSON.
///
/// Overlapping `{button}` shapes across chrome/message are intentional:
/// `#[serde(untagged)]` keeps the wire shape `{ "button": "<label>" }`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum CommandResult {
    /// Chrome frame result (Phase A).
    Chrome(ChromeResult),
    /// Message dialog result (Phase B / REQ-0064).
    Message(MessageResult),
}

/// Chrome command result payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ChromeResult {
    /// Button label selected by the user (or dismissed on OS close).
    pub button: ButtonLabel,
}

/// Message dialog result payload (REQ-0064).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MessageResult {
    /// Button label selected by the user (or dismissed on OS close).
    pub button: ButtonLabel,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_result_chrome_wire_shape() {
        let result = CommandResult::Chrome(ChromeResult {
            button: ButtonLabel::dismissed(),
        });
        let json = serde_json::to_string(&result).expect("serialize");
        assert_eq!(json, r#"{"button":"dismissed"}"#);
    }

    #[test]
    fn command_result_message_wire_shape() {
        let result = CommandResult::Message(MessageResult {
            button: ButtonLabel::new("ok"),
        });
        let json = serde_json::to_string(&result).expect("serialize");
        assert_eq!(json, r#"{"button":"ok"}"#);
    }
}
