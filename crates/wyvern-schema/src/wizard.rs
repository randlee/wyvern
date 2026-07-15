//! Wizard command, stack, and HTTP wire types (Phase D).

use serde::{Deserialize, Serialize};

use crate::button::ButtonLabel;

/// Per-page layout hint (`dialog` | `workspace`).
///
/// Validated in d.1; sizing behavior lands in d.6.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WizardPageLayout {
    /// Typical form step (default when omitted).
    Dialog,
    /// HTML page requesting a viewport-sized canvas.
    Workspace,
}

impl WizardPageLayout {
    /// Parse a wire layout name (`dialog`, `workspace`).
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "dialog" => Some(Self::Dialog),
            "workspace" => Some(Self::Workspace),
            _ => None,
        }
    }

    /// All valid wire names (for error messages / suggestions).
    pub fn all_names() -> &'static [&'static str] {
        &["dialog", "workspace"]
    }

    /// Wire name for this layout.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dialog => "dialog",
            Self::Workspace => "workspace",
        }
    }
}

/// Minimal page descriptor (REQ-0026).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WizardPageDescriptor {
    /// Stable page identity.
    pub id: String,
    /// Display title for the page.
    pub title: String,
    /// HTML path relative to `--ui-root` (no separate `page_html` field).
    pub html: String,
    /// Optional per-page layout (`dialog` | `workspace`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<WizardPageLayout>,
}

/// One prior stack entry (REQ-0024).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WizardStackEntry {
    /// Page that was visited.
    pub page: WizardPageDescriptor,
    /// Opaque page data stored for that visit.
    pub data: serde_json::Value,
}

/// Validated wizard ingress after schema validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WizardCommand {
    /// Initial page descriptor.
    pub page: WizardPageDescriptor,
    /// Opaque wizard-wide config (never inspected by the host).
    pub config: serde_json::Value,
    /// Optional viewer width hint.
    pub width: Option<u32>,
    /// Optional viewer height hint.
    pub height: Option<u32>,
}

/// Wizard stdout / finish body shape (REQ-0066).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WizardResult {
    /// Terminal button (`finish` | `cancel` | `dismissed`).
    pub button: ButtonLabel,
    /// Opaque final page data (empty on cancel / dismiss).
    pub data: serde_json::Value,
    /// Visited stack (semantics finalized in d.2).
    pub stack: Vec<WizardStackEntry>,
}

impl WizardResult {
    /// REQ-0066 / REQ-0097 dismissed shape for timeout / viewer exit (d.1 stub).
    ///
    /// Full visited-stack algorithm lands with `finish` in d.2; d.1 dismisses with
    /// empty `data` and an empty prior stack (cursor still on the first page).
    pub fn dismissed() -> Self {
        Self {
            button: ButtonLabel::dismissed(),
            data: serde_json::json!({}),
            stack: Vec::new(),
        }
    }
}

/// Wire DTO for `GET /api/wizard/state` (HTTP-TYPES.md).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WizardStateResponse {
    /// Always `"wizard"`.
    #[serde(rename = "type")]
    pub type_name: &'static str,
    /// Opaque wizard-wide config.
    pub config: serde_json::Value,
    /// Current page descriptor.
    pub page: WizardPageDescriptor,
    /// Opaque data for the current page.
    pub page_data: serde_json::Value,
    /// Prior stack entries only (`entries[0..cursor]`, exclusive of current).
    pub stack: Vec<WizardStackEntry>,
    /// Optional viewer width from the command.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    /// Optional viewer height from the command.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
}

impl WizardStateResponse {
    /// Build a state response from a session snapshot plus optional window hints.
    pub fn from_snapshot(
        config: serde_json::Value,
        page: WizardPageDescriptor,
        page_data: serde_json::Value,
        stack: Vec<WizardStackEntry>,
        width: Option<u32>,
        height: Option<u32>,
    ) -> Self {
        Self {
            type_name: "wizard",
            config,
            page,
            page_data,
            stack,
            width,
            height,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wizard_state_response_wire_shape_first_page() {
        let resp = WizardStateResponse::from_snapshot(
            serde_json::json!({"theme": "dark"}),
            WizardPageDescriptor {
                id: "start".into(),
                title: "Start".into(),
                html: "pages/start.html".into(),
                layout: None,
            },
            serde_json::json!({}),
            Vec::new(),
            Some(640),
            Some(480),
        );
        let value = serde_json::to_value(&resp).expect("serialize");
        assert_eq!(value["type"], "wizard");
        assert_eq!(value["config"]["theme"], "dark");
        assert_eq!(value["page"]["id"], "start");
        assert_eq!(value["page_data"], serde_json::json!({}));
        assert_eq!(value["stack"], serde_json::json!([]));
        assert_eq!(value["width"], 640);
        assert_eq!(value["height"], 480);
    }

    #[test]
    fn page_layout_omitted_when_none() {
        let page = WizardPageDescriptor {
            id: "a".into(),
            title: "A".into(),
            html: "a.html".into(),
            layout: None,
        };
        let value = serde_json::to_value(&page).expect("serialize");
        assert!(value.get("layout").is_none());
    }
}
