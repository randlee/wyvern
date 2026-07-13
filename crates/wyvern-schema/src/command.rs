//! Typed command surface for the current phase.

use crate::chrome::{ChromeStatus, ChromeTitle};

/// Standard button preset for dialog types (REQ Phase B).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonsPreset {
    /// Single OK button.
    Ok,
    /// OK + Cancel.
    OkCancel,
    /// Yes + No.
    YesNo,
    /// Yes + No + Cancel.
    YesNoCancel,
    /// Retry + Cancel.
    RetryCancel,
    /// Caller-supplied labels via `custom_buttons`.
    Custom,
}

impl ButtonsPreset {
    /// Parse a wire preset name (`ok`, `ok_cancel`, …).
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "ok" => Some(Self::Ok),
            "ok_cancel" => Some(Self::OkCancel),
            "yes_no" => Some(Self::YesNo),
            "yes_no_cancel" => Some(Self::YesNoCancel),
            "retry_cancel" => Some(Self::RetryCancel),
            "custom" => Some(Self::Custom),
            _ => None,
        }
    }

    /// All valid wire names (for error messages / suggestions).
    pub fn all_names() -> &'static [&'static str] {
        &[
            "ok",
            "ok_cancel",
            "yes_no",
            "yes_no_cancel",
            "retry_cancel",
            "custom",
        ]
    }

    /// Display labels shown in the HTML button bar (ipc-dialog-contract).
    pub fn display_labels(self, custom_buttons: Option<&[String]>) -> Vec<String> {
        match self {
            Self::Ok => vec!["OK".into()],
            Self::OkCancel => vec!["OK".into(), "Cancel".into()],
            Self::YesNo => vec!["Yes".into(), "No".into()],
            Self::YesNoCancel => vec!["Yes".into(), "No".into(), "Cancel".into()],
            Self::RetryCancel => vec!["Retry".into(), "Cancel".into()],
            Self::Custom => custom_buttons.unwrap_or(&[]).to_vec(),
        }
    }

    /// Stdout / IPC wire labels corresponding 1:1 with [`Self::display_labels`].
    pub fn wire_labels(self, custom_buttons: Option<&[String]>) -> Vec<String> {
        match self {
            Self::Ok => vec!["ok".into()],
            Self::OkCancel => vec!["ok".into(), "cancel".into()],
            Self::YesNo => vec!["yes".into(), "no".into()],
            Self::YesNoCancel => vec!["yes".into(), "no".into(), "cancel".into()],
            Self::RetryCancel => vec!["retry".into(), "cancel".into()],
            Self::Custom => custom_buttons.unwrap_or(&[]).to_vec(),
        }
    }

    /// Number of buttons for the active preset (or custom list).
    pub fn button_count(self, custom_buttons: Option<&[String]>) -> usize {
        self.wire_labels(custom_buttons).len()
    }
}

/// Semantic severity for a message dialog (REQ-0012).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageLevel {
    /// Informational notice.
    Info,
    /// Caution / non-fatal problem.
    Warning,
    /// Error condition.
    Error,
    /// Prompt requiring a decision.
    Question,
}

impl MessageLevel {
    /// Parse a wire level name (`info`, `warning`, …).
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "info" => Some(Self::Info),
            "warning" => Some(Self::Warning),
            "error" => Some(Self::Error),
            "question" => Some(Self::Question),
            _ => None,
        }
    }

    /// All valid wire names (for error messages / suggestions).
    pub fn all_names() -> &'static [&'static str] {
        &["info", "warning", "error", "question"]
    }

    /// Wire / asset name for this level.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Question => "question",
        }
    }
}

/// Input dialog mode (REQ-0014).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    /// Free-text field (default when `mode` is omitted).
    Text,
    /// Native file picker via `rfd` in `wyvern-window` (sprint b.4).
    File,
    /// Native folder picker via `rfd` in `wyvern-window` (sprint b.4).
    Folder,
}

impl InputMode {
    /// Parse a wire mode name (`text`, `file`, `folder`).
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "text" => Some(Self::Text),
            "file" => Some(Self::File),
            "folder" => Some(Self::Folder),
            _ => None,
        }
    }

    /// All valid wire names (for error messages / suggestions).
    pub fn all_names() -> &'static [&'static str] {
        &["text", "file", "folder"]
    }

    /// Wire name for this mode.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::File => "file",
            Self::Folder => "folder",
        }
    }
}

/// Executable command after successful validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Foundation chrome frame: required `title`, optional `status`.
    Chrome {
        title: ChromeTitle,
        status: Option<ChromeStatus>,
    },
    /// Modal message dialog (Phase B sprint b.1 / b.2).
    Message {
        title: ChromeTitle,
        message: String,
        status: Option<ChromeStatus>,
        buttons: ButtonsPreset,
        custom_buttons: Option<Vec<String>>,
        default_button: Option<u32>,
        level: Option<MessageLevel>,
        icon: Option<String>,
        image: Option<String>,
        markdown: bool,
    },
    /// Modal input dialog — text / file / folder (REQ-0013 / REQ-0015).
    Input {
        title: ChromeTitle,
        message: String,
        status: Option<ChromeStatus>,
        icon: Option<String>,
        markdown: bool,
        multiline: bool,
        placeholder: Option<String>,
        default: Option<String>,
        mode: InputMode,
        /// Extension patterns (`*.json`, …); file mode only (REQ-0015 / REQ-0059).
        filter: Option<Vec<String>>,
        /// Multi-file selection; file mode only (REQ-0015 / REQ-0059).
        multiple: bool,
        /// Initial picker directory; file or folder mode only (REQ-0059).
        start_path: Option<String>,
        buttons: ButtonsPreset,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_label_mapping_table() {
        assert_eq!(ButtonsPreset::Ok.display_labels(None), ["OK"]);
        assert_eq!(ButtonsPreset::Ok.wire_labels(None), ["ok"]);

        assert_eq!(
            ButtonsPreset::OkCancel.display_labels(None),
            ["OK", "Cancel"]
        );
        assert_eq!(ButtonsPreset::OkCancel.wire_labels(None), ["ok", "cancel"]);

        assert_eq!(ButtonsPreset::YesNo.display_labels(None), ["Yes", "No"]);
        assert_eq!(ButtonsPreset::YesNo.wire_labels(None), ["yes", "no"]);

        assert_eq!(
            ButtonsPreset::YesNoCancel.display_labels(None),
            ["Yes", "No", "Cancel"]
        );
        assert_eq!(
            ButtonsPreset::YesNoCancel.wire_labels(None),
            ["yes", "no", "cancel"]
        );

        assert_eq!(
            ButtonsPreset::RetryCancel.display_labels(None),
            ["Retry", "Cancel"]
        );
        assert_eq!(
            ButtonsPreset::RetryCancel.wire_labels(None),
            ["retry", "cancel"]
        );
    }

    #[test]
    fn custom_labels_are_verbatim() {
        let custom = vec!["Save".into(), "Discard".into()];
        assert_eq!(ButtonsPreset::Custom.display_labels(Some(&custom)), custom);
        assert_eq!(ButtonsPreset::Custom.wire_labels(Some(&custom)), custom);
    }
}
