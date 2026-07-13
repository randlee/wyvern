//! Protocol result types written to stdout on success.

use serde::Serialize;

use crate::button::ButtonLabel;

/// Successful command result for stdout JSON.
///
/// Overlapping `{button}` shapes across chrome/message/markdown/input are intentional:
/// `#[serde(untagged)]` keeps the wire shape `{ "button": "<label>" }` (and
/// optional `input` for text/file results per REQ-0065).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum CommandResult {
    /// Chrome frame result (Phase A).
    Chrome(ChromeResult),
    /// Message dialog result (Phase B / REQ-0064).
    Message(MessageResult),
    /// Markdown viewer result (Phase B / REQ-0064).
    Markdown(MarkdownResult),
    /// Input dialog result (Phase B / REQ-0065).
    Input(InputResult),
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

/// Markdown viewer result payload (REQ-0064).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MarkdownResult {
    /// Button label selected by the user (or dismissed on OS close).
    pub button: ButtonLabel,
}

/// Value carried in [`InputResult::input`] (REQ-0065).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum InputValue {
    /// Text mode value, or a single file/folder path.
    Text(String),
    /// Multi-select file paths (`multiple: true`, sprint b.4).
    Paths(Vec<String>),
}

/// Input dialog result payload (REQ-0065).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InputResult {
    /// Button label selected by the user (or dismissed on OS close).
    pub button: ButtonLabel,
    /// Submitted value; omitted on cancel / dismiss.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<InputValue>,
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

    #[test]
    fn command_result_markdown_wire_shape() {
        let result = CommandResult::Markdown(MarkdownResult {
            button: ButtonLabel::new("ok"),
        });
        let json = serde_json::to_string(&result).expect("serialize");
        assert_eq!(json, r#"{"button":"ok"}"#);
    }

    #[test]
    fn command_result_input_ok_with_text() {
        let result = CommandResult::Input(InputResult {
            button: ButtonLabel::new("ok"),
            input: Some(InputValue::Text("Ada Lovelace".into())),
        });
        let json = serde_json::to_string(&result).expect("serialize");
        assert_eq!(json, r#"{"button":"ok","input":"Ada Lovelace"}"#);
    }

    #[test]
    fn command_result_input_cancel_omits_input() {
        let result = CommandResult::Input(InputResult {
            button: ButtonLabel::new("cancel"),
            input: None,
        });
        let json = serde_json::to_string(&result).expect("serialize");
        assert_eq!(json, r#"{"button":"cancel"}"#);
    }

    #[test]
    fn command_result_input_dismissed_omits_input() {
        let result = CommandResult::Input(InputResult {
            button: ButtonLabel::dismissed(),
            input: None,
        });
        let json = serde_json::to_string(&result).expect("serialize");
        assert_eq!(json, r#"{"button":"dismissed"}"#);
    }

    #[test]
    fn command_result_input_ok_with_paths_array() {
        let result = CommandResult::Input(InputResult {
            button: ButtonLabel::new("ok"),
            input: Some(InputValue::Paths(vec![
                "/tmp/a.json".into(),
                "/tmp/b.json".into(),
            ])),
        });
        let json = serde_json::to_string(&result).expect("serialize");
        assert_eq!(
            json,
            r#"{"button":"ok","input":["/tmp/a.json","/tmp/b.json"]}"#
        );
    }
}
