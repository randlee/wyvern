//! Report command and result types (Phase H / ADR-0025).

use std::fmt;
use std::ops::Deref;

use serde::{Deserialize, Deserializer, Serialize};

/// Maximum review comments length (Unicode scalar values).
///
/// Bound from [xhtml-reporting-contract.md](../../docs/plans/phase-H/xhtml-reporting-contract.md)
/// so finish JSON stays small and textarea input cannot balloon the session payload.
pub const MAX_REVIEW_COMMENTS_CHARS: usize = 32_768;

/// Maximum `panels` entries on a report command (schema `maxItems`).
///
/// Bound from [xhtml-reporting-contract.md](../../docs/plans/phase-H/xhtml-reporting-contract.md)
/// and `review-manifest.schema.json`. Changing this changes the preexec and
/// validate error inventory.
pub const MAX_REPORT_PANELS: usize = 32;

/// Maximum `panels[].label` length (Unicode scalar values).
///
/// Pane headings stay short (basename fallback is typically a filename). 256
/// covers localized titles without letting finish JSON carry an unbounded string.
pub const MAX_PANEL_LABEL_CHARS: usize = 256;

/// Error when a report identity or path field is invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFieldError {
    /// Value is empty.
    Empty,
    /// [`ReportPagePath`] does not end with `.html` or `.xhtml`.
    InvalidPageSuffix,
    /// [`ManifestPanelPath`] does not end with `.xhtml`.
    InvalidPanelSuffix,
    /// [`ReviewComments`] exceeds [`MAX_REVIEW_COMMENTS_CHARS`].
    CommentsTooLong,
    /// [`PanelLabel`] exceeds [`MAX_PANEL_LABEL_CHARS`].
    LabelTooLong,
}

impl fmt::Display for ReportFieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("report field must be a non-empty string"),
            Self::InvalidPageSuffix => {
                f.write_str("report page path must end with .html or .xhtml")
            }
            Self::InvalidPanelSuffix => f.write_str("manifest panel path must end with .xhtml"),
            Self::CommentsTooLong => write!(
                f,
                "review comments must be at most {MAX_REVIEW_COMMENTS_CHARS} characters"
            ),
            Self::LabelTooLong => write!(
                f,
                "panel label must be at most {MAX_PANEL_LABEL_CHARS} characters"
            ),
        }
    }
}

impl std::error::Error for ReportFieldError {}

macro_rules! report_newtype {
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

report_newtype!(
    ReportPagePath,
    "Validated report page path relative to `--ui-root` (`.html` or `.xhtml`)."
);
report_newtype!(ReportTitle, "Validated report window title (non-empty).");
report_newtype!(
    ManifestPanelPath,
    "Validated manifest panel path (non-empty `.xhtml` relative path)."
);

impl ReportTitle {
    /// Construct from a non-empty string.
    ///
    /// # Errors
    ///
    /// Returns [`ReportFieldError::Empty`] when `value` is empty.
    pub fn try_new(value: impl Into<String>) -> Result<Self, ReportFieldError> {
        let value = value.into();
        if value.is_empty() {
            return Err(ReportFieldError::Empty);
        }
        Ok(Self(value))
    }
}

impl ReportPagePath {
    /// Construct from a non-empty `.html` or `.xhtml` path.
    ///
    /// # Errors
    ///
    /// Returns [`ReportFieldError::Empty`] when `value` is empty, or
    /// [`ReportFieldError::InvalidPageSuffix`] when the path does not end with
    /// `.html` or `.xhtml` (ASCII case-insensitive).
    pub fn try_new(value: impl Into<String>) -> Result<Self, ReportFieldError> {
        let value = value.into();
        if value.is_empty() {
            return Err(ReportFieldError::Empty);
        }
        if !has_html_or_xhtml_suffix(&value) {
            return Err(ReportFieldError::InvalidPageSuffix);
        }
        Ok(Self(value))
    }
}

impl ManifestPanelPath {
    /// Construct from a non-empty `.xhtml` relative path.
    ///
    /// # Errors
    ///
    /// Returns [`ReportFieldError::Empty`] when `value` is empty, or
    /// [`ReportFieldError::InvalidPanelSuffix`] when the path does not end with
    /// `.xhtml` (ASCII case-insensitive).
    pub fn try_new(value: impl Into<String>) -> Result<Self, ReportFieldError> {
        let value = value.into();
        if value.is_empty() {
            return Err(ReportFieldError::Empty);
        }
        if !has_xhtml_suffix(&value) {
            return Err(ReportFieldError::InvalidPanelSuffix);
        }
        Ok(Self(value))
    }
}

fn has_html_or_xhtml_suffix(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.ends_with(".html") || lower.ends_with(".xhtml")
}

fn has_xhtml_suffix(value: &str) -> bool {
    value.to_ascii_lowercase().ends_with(".xhtml")
}

/// Bounded review-finish comments (empty allowed, max 32 KiB scalars).
///
/// Construct via [`Self::try_new`] at HTTP/JSON trust boundaries so
/// [`ReportFinishData`] cannot carry an unbounded `String`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct ReviewComments(String);

impl ReviewComments {
    /// Wrap already-validated comments, including the empty string.
    ///
    /// Prefer [`Self::try_new`] at trust boundaries.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Construct comments that fit [`MAX_REVIEW_COMMENTS_CHARS`].
    ///
    /// Empty comments are allowed. Length is counted in Unicode scalar values.
    ///
    /// # Errors
    ///
    /// Returns [`ReportFieldError::CommentsTooLong`] when `value` exceeds the bound.
    pub fn try_new(value: impl Into<String>) -> Result<Self, ReportFieldError> {
        let value = value.into();
        if value.chars().count() > MAX_REVIEW_COMMENTS_CHARS {
            return Err(ReportFieldError::CommentsTooLong);
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

impl Deref for ReviewComments {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for ReviewComments {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for ReviewComments {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl PartialEq<str> for ReviewComments {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for ReviewComments {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl<'de> Deserialize<'de> for ReviewComments {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Self::try_new(value).map_err(serde::de::Error::custom)
    }
}

/// Bounded pane heading for a manifest panel (non-empty, max 256 scalars).
///
/// Construct via [`Self::try_new`] at the validation boundary so
/// [`ReportPanelEntry::label`] cannot carry an unbounded `String`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct PanelLabel(String);

impl PanelLabel {
    /// Wrap an already-validated pane heading.
    ///
    /// Prefer [`Self::try_new`] at trust boundaries.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Construct a non-empty label that fits [`MAX_PANEL_LABEL_CHARS`].
    ///
    /// # Errors
    ///
    /// Returns [`ReportFieldError::Empty`] when `value` is empty, or
    /// [`ReportFieldError::LabelTooLong`] when it exceeds the bound.
    pub fn try_new(value: impl Into<String>) -> Result<Self, ReportFieldError> {
        let value = value.into();
        if value.is_empty() {
            return Err(ReportFieldError::Empty);
        }
        if value.chars().count() > MAX_PANEL_LABEL_CHARS {
            return Err(ReportFieldError::LabelTooLong);
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

impl Deref for PanelLabel {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for PanelLabel {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for PanelLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<String> for PanelLabel {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for PanelLabel {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl PartialEq<str> for PanelLabel {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for PanelLabel {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl<'de> Deserialize<'de> for PanelLabel {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Self::try_new(value).map_err(serde::de::Error::custom)
    }
}

/// Report session mode (`view` | `review`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReportMode {
    /// Static document; OS-close / `/api/result` dismiss (h.1).
    View,
    /// Static document plus terminal Approve/Cancel (h.3).
    Review,
}

impl ReportMode {
    /// Parse a wire mode name (`view`, `review`).
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "view" => Some(Self::View),
            "review" => Some(Self::Review),
            _ => None,
        }
    }

    /// All valid wire names (for error messages / suggestions).
    pub fn all_names() -> &'static [&'static str] {
        &["view", "review"]
    }

    /// Wire name for this mode.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::View => "view",
            Self::Review => "review",
        }
    }
}

/// CSS role on a stitched pane (`failure` | `proposal` | `info`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PanelRole {
    /// Failed-benchmark / error pane.
    Failure,
    /// Proposed-fix pane.
    Proposal,
    /// Informational pane.
    Info,
}

impl PanelRole {
    /// Parse a wire role name.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "failure" => Some(Self::Failure),
            "proposal" => Some(Self::Proposal),
            "info" => Some(Self::Info),
            _ => None,
        }
    }

    /// All valid wire names (for error messages / suggestions).
    pub fn all_names() -> &'static [&'static str] {
        &["failure", "proposal", "info"]
    }

    /// Wire name for this role.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Failure => "failure",
            Self::Proposal => "proposal",
            Self::Info => "info",
        }
    }
}

/// One manifest panel entry on a report command (required when `mode` is review).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportPanelEntry {
    /// `.xhtml` path relative to the manifest / `ui_root`.
    pub path: ManifestPanelPath,
    /// Optional pane heading (defaults to basename at stitch time).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<PanelLabel>,
    /// Optional CSS role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<PanelRole>,
}

/// Validated report ingress after schema validation (REQ-0140).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportCommand {
    /// Window / viewer title.
    pub title: ReportTitle,
    /// Page path relative to `--ui-root` (`.html` or `.xhtml`).
    pub page: ReportPagePath,
    /// `view` (default) or `review`.
    pub mode: ReportMode,
    /// Manifest panel entries; required when [`ReportMode::Review`].
    pub panels: Option<Vec<ReportPanelEntry>>,
    /// Optional viewer width hint.
    pub width: Option<u32>,
    /// Optional viewer height hint.
    pub height: Option<u32>,
}

/// Terminal buttons accepted on report stdout (view dismiss + review finish).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReportTerminalButton {
    /// Viewer dismissed / OS-close (no `data`).
    Dismissed,
    /// Review-mode Approve/Cancel finish (h.3).
    Finish,
}

impl ReportTerminalButton {
    /// Wire name for this button.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dismissed => "dismissed",
            Self::Finish => "finish",
        }
    }
}

/// Review-mode finish payload (h.3). Absent on view dismiss.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportFinishData {
    /// `true` = Approve; `false` = Cancel.
    pub approved: bool,
    /// Free-text comments (may be empty; bounded by [`MAX_REVIEW_COMMENTS_CHARS`]).
    pub comments: ReviewComments,
    /// Echo of authoritative manifest panel entries.
    pub panels: Vec<ReportPanelEntry>,
}

/// Report stdout / dismiss-or-finish body (REQ-0143 / REQ-0144).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportResult {
    /// Terminal button (`dismissed` | `finish`).
    pub button: ReportTerminalButton,
    /// Review finish payload; omitted on view dismiss.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<ReportFinishData>,
}

impl ReportResult {
    /// View-mode / OS-close dismiss with no finish `data` (REQ-0143).
    pub fn dismissed() -> Self {
        Self {
            button: ReportTerminalButton::Dismissed,
            data: None,
        }
    }

    /// Review-mode Approve/Cancel finish with validated `data` (REQ-0144).
    pub fn finished(data: ReportFinishData) -> Self {
        Self {
            button: ReportTerminalButton::Finish,
            data: Some(data),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn review_comments_try_new_enforces_bound() {
        assert_eq!(ReviewComments::try_new("").unwrap().as_str(), "");
        assert_eq!(
            ReviewComments::try_new("x".repeat(MAX_REVIEW_COMMENTS_CHARS + 1)),
            Err(ReportFieldError::CommentsTooLong)
        );
        assert!(ReviewComments::try_new("x".repeat(MAX_REVIEW_COMMENTS_CHARS)).is_ok());
    }

    #[test]
    fn panel_label_try_new_rejects_empty_and_enforces_bound() {
        assert_eq!(PanelLabel::try_new(""), Err(ReportFieldError::Empty));
        assert_eq!(
            PanelLabel::try_new("x".repeat(MAX_PANEL_LABEL_CHARS + 1)),
            Err(ReportFieldError::LabelTooLong)
        );
        assert_eq!(PanelLabel::try_new("Fail 1").unwrap().as_str(), "Fail 1");
        assert!(PanelLabel::try_new("x".repeat(MAX_PANEL_LABEL_CHARS)).is_ok());
    }

    #[test]
    fn report_title_try_new_rejects_empty() {
        assert_eq!(ReportTitle::try_new(""), Err(ReportFieldError::Empty));
        assert_eq!(ReportTitle::try_new("ok").unwrap().as_str(), "ok");
    }

    #[test]
    fn report_page_path_try_new_rejects_empty_and_bad_suffix() {
        assert_eq!(ReportPagePath::try_new(""), Err(ReportFieldError::Empty));
        assert_eq!(
            ReportPagePath::try_new("pages/view.txt"),
            Err(ReportFieldError::InvalidPageSuffix)
        );
        assert_eq!(
            ReportPagePath::try_new("pages/view.xhtml")
                .unwrap()
                .as_str(),
            "pages/view.xhtml"
        );
        assert_eq!(
            ReportPagePath::try_new("pages/view.HTML").unwrap().as_str(),
            "pages/view.HTML"
        );
    }

    #[test]
    fn manifest_panel_path_try_new_requires_xhtml_suffix() {
        assert_eq!(ManifestPanelPath::try_new(""), Err(ReportFieldError::Empty));
        assert_eq!(
            ManifestPanelPath::try_new("panels/fail.html"),
            Err(ReportFieldError::InvalidPanelSuffix)
        );
        assert_eq!(
            ManifestPanelPath::try_new("panels/fail-1.xhtml")
                .unwrap()
                .as_str(),
            "panels/fail-1.xhtml"
        );
        assert_eq!(
            ManifestPanelPath::try_new("panels/fail-1.XHTML")
                .unwrap()
                .as_str(),
            "panels/fail-1.XHTML"
        );
    }

    #[test]
    fn report_mode_parse_round_trip() {
        for (wire, expected) in [("view", ReportMode::View), ("review", ReportMode::Review)] {
            assert_eq!(ReportMode::parse(wire), Some(expected));
            assert_eq!(expected.as_str(), wire);
        }
        assert!(ReportMode::parse("wizard").is_none());
    }

    #[test]
    fn report_result_dismissed_omits_data() {
        let json = serde_json::to_string(&ReportResult::dismissed()).expect("serialize");
        assert_eq!(json, r#"{"button":"dismissed"}"#);
    }

    #[test]
    fn report_result_finished_includes_data() {
        let result = ReportResult::finished(ReportFinishData {
            approved: false,
            comments: ReviewComments::new(""),
            panels: vec![ReportPanelEntry {
                path: ManifestPanelPath::new("panels/fail.xhtml"),
                label: Some("Fail 1".into()),
                role: Some(PanelRole::Failure),
            }],
        });
        let value: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&result).expect("serialize"))
                .expect("json");
        assert_eq!(value["button"], "finish");
        assert_eq!(value["data"]["approved"], false);
        assert_eq!(value["data"]["comments"], "");
        assert_eq!(value["data"]["panels"][0]["path"], "panels/fail.xhtml");
    }

    #[test]
    fn report_terminal_button_wire_names() {
        assert_eq!(ReportTerminalButton::Dismissed.as_str(), "dismissed");
        assert_eq!(ReportTerminalButton::Finish.as_str(), "finish");
    }
}
