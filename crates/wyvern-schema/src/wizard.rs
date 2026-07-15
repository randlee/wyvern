//! Wizard command, stack, and HTTP wire types (Phase D).

use std::fmt;
use std::ops::Deref;

use serde::{Deserialize, Serialize};

use crate::button::ButtonLabel;

/// Error when a wizard page identity field is empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WizardPageFieldError;

impl fmt::Display for WizardPageFieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("wizard page field must be a non-empty string")
    }
}

impl std::error::Error for WizardPageFieldError {}

macro_rules! wizard_page_newtype {
    ($(#[$meta:meta])* $name:ident, $doc:literal) => {
        $(#[$meta])*
        #[doc = $doc]
        ///
        /// Construct via [`Self::try_new`] at the validation boundary so downstream
        /// code can treat the value as already checked non-empty.
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Wrap a validated non-empty string.
            ///
            /// Prefer [`Self::try_new`] at trust boundaries; this constructor is for
            /// already-validated values (e.g. after [`crate::validate`]).
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            /// Construct from a non-empty string.
            ///
            /// # Errors
            ///
            /// Returns [`WizardPageFieldError`] when `value` is empty.
            pub fn try_new(value: impl Into<String>) -> Result<Self, WizardPageFieldError> {
                let value = value.into();
                if value.is_empty() {
                    return Err(WizardPageFieldError);
                }
                Ok(Self(value))
            }

            /// Borrow as a string slice.
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Consume and return the inner string.
            pub fn into_inner(self) -> String {
                self.0
            }
        }

        impl Deref for $name {
            type Target = str;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        // No infallible `From<String>` / `From<&str>` — those bypass `try_new`
        // non-empty validation. Use `try_new` at trust boundaries; `new` only
        // after validation (e.g. `crate::validate`).

        impl PartialEq<str> for $name {
            fn eq(&self, other: &str) -> bool {
                self.0 == other
            }
        }

        impl PartialEq<&str> for $name {
            fn eq(&self, other: &&str) -> bool {
                self.0 == *other
            }
        }
    };
}

wizard_page_newtype!(WizardPageId, "Validated wizard page id (non-empty).");
wizard_page_newtype!(WizardPageTitle, "Validated wizard page title (non-empty).");
wizard_page_newtype!(
    WizardPageHtml,
    "Validated wizard page HTML path relative to `--ui-root` (non-empty)."
);

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
    pub id: WizardPageId,
    /// Display title for the page.
    pub title: WizardPageTitle,
    /// HTML path relative to `--ui-root` (no separate `page_html` field).
    pub html: WizardPageHtml,
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
    /// REQ-0066 / REQ-0097 dismissed shape with empty stack.
    ///
    /// Prefer session-derived dismiss via wizard `finish(Dismissed, …)` (d.2 / d.8)
    /// when a live wizard session exists — that path returns the full visited stack.
    /// This helper remains for degraded fallbacks and non-session call sites.
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

/// Wire: `"next"` | `"back"` only. Cancel/finish/dismissed use `POST /api/wizard/finish`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WizardNavAction {
    /// Advance to an explicit next page descriptor.
    Next,
    /// Move the cursor back without truncating forward history.
    Back,
}

/// Wire DTO for `POST /api/wizard/navigate` (HTTP-TYPES.md).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct WizardNavigateRequest {
    /// `"next"` or `"back"` only — terminal actions use `/finish`.
    pub action: WizardNavAction,
    /// Opaque page payload for the current entry (whole-blob replace).
    #[serde(default)]
    pub data: serde_json::Value,
    /// Optional validation hint; when set on `next`, must equal `next.id`.
    #[serde(default)]
    pub page_id: Option<WizardPageId>,
    /// Required when `action` is `next` — destination page descriptor.
    #[serde(default)]
    pub next: Option<WizardPageDescriptor>,
}

/// Terminal finish buttons accepted by `POST /api/wizard/finish`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WizardTerminalButton {
    /// Complete successfully with full visited stack.
    Finish,
    /// Abort — empty stack and empty data on stdout.
    Cancel,
    /// Viewer dismissed — full visited stack, empty data.
    Dismissed,
}

impl WizardTerminalButton {
    /// Wire name for this button.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Finish => "finish",
            Self::Cancel => "cancel",
            Self::Dismissed => "dismissed",
        }
    }

    /// Convert to the stdout [`ButtonLabel`] shape.
    pub fn to_button_label(self) -> ButtonLabel {
        ButtonLabel::new(self.as_str())
    }
}

/// Successful navigate response — viewer reloads `url`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WizardNavigateResponse {
    /// Always `true` on HTTP 200.
    pub ok: bool,
    /// Absolute URL of the destination wizard page under `/wizard/`.
    pub url: String,
}

/// Wire DTO for `POST /api/wizard/finish` (HTTP-TYPES.md).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct WizardFinishRequest {
    /// Terminal button (`finish` | `cancel` | `dismissed`).
    pub button: WizardTerminalButton,
    /// Opaque final page data (ignored for cancel / dismissed stdout `data`).
    pub data: serde_json::Value,
    /// Client-supplied full visited stack (validated for finish / dismissed).
    pub stack: Vec<WizardStackEntry>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wizard_state_response_wire_shape_first_page() {
        let resp = WizardStateResponse::from_snapshot(
            serde_json::json!({"theme": "dark"}),
            WizardPageDescriptor {
                id: WizardPageId::new("start"),
                title: WizardPageTitle::new("Start"),
                html: WizardPageHtml::new("pages/start.html"),
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
            id: WizardPageId::new("a"),
            title: WizardPageTitle::new("A"),
            html: WizardPageHtml::new("a.html"),
            layout: None,
        };
        let value = serde_json::to_value(&page).expect("serialize");
        assert!(value.get("layout").is_none());
    }

    #[test]
    fn page_id_try_new_rejects_empty() {
        assert_eq!(WizardPageId::try_new(""), Err(WizardPageFieldError));
        assert_eq!(WizardPageId::try_new("ok").unwrap().as_str(), "ok");
    }

    #[test]
    fn navigate_request_deserializes_typed_page_id() {
        let req: WizardNavigateRequest = serde_json::from_value(serde_json::json!({
            "action": "next",
            "data": {},
            "page_id": "step-2",
            "next": {
                "id": "step-2",
                "title": "Step 2",
                "html": "pages/step-2.html"
            }
        }))
        .expect("deserialize");
        assert_eq!(
            req.page_id.as_ref().map(WizardPageId::as_str),
            Some("step-2")
        );
        assert_eq!(req.next.as_ref().unwrap().id.as_str(), "step-2");
    }

    #[test]
    fn terminal_button_wire_round_trip() {
        for (wire, expected) in [
            ("finish", WizardTerminalButton::Finish),
            ("cancel", WizardTerminalButton::Cancel),
            ("dismissed", WizardTerminalButton::Dismissed),
        ] {
            let parsed: WizardTerminalButton =
                serde_json::from_value(serde_json::json!(wire)).expect("parse");
            assert_eq!(parsed, expected);
            assert_eq!(parsed.as_str(), wire);
            assert_eq!(parsed.to_button_label().as_str(), wire);
        }
    }
}
