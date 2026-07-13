//! Protocol result types written to stdout on success.

use serde::Serialize;

/// Successful command result for stdout JSON.
///
/// Phase A wire for chrome is `{"button":"dismissed"}` via `#[serde(untagged)]`.
/// Overlapping `{button}` shapes across later variants are intentional.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum CommandResult {
    /// Chrome / message / markdown-style button result.
    Chrome(ChromeResult),
}

/// Chrome command result payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ChromeResult {
    /// Button label selected by the user (or `"dismissed"` on OS close).
    pub button: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_result_chrome_wire_shape() {
        let result = CommandResult::Chrome(ChromeResult {
            button: "dismissed".into(),
        });
        let json = serde_json::to_string(&result).expect("serialize");
        assert_eq!(json, r#"{"button":"dismissed"}"#);
    }
}
