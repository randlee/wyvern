//! Review-mode finish request validation (REQ-0144 / REQ-HOST-0142).
//!
//! Stable `REPORT_FINISH_*` codes are the machine contract for HTTP 400/409
//! bodies. Posted `panels` must match the session's [`ValidatedReportManifest`].

use serde_json::Value;
use wyvern_schema::{
    ManifestPanelPath, PanelLabel, PanelRole, ReportFinishData, ReportPanelEntry, ReviewComments,
    MAX_PANEL_LABEL_CHARS, MAX_REVIEW_COMMENTS_CHARS,
};

use crate::report_session::ValidatedReportManifest;

/// Allowed top-level finish POST keys (unknown keys → 400).
const FINISH_KEYS: &[&str] = &["approved", "comments", "panels"];

/// Stable finish-validation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReportFinishError {
    kind: ReportFinishErrorKind,
    message: String,
}

/// Finish error classes mapped to `REPORT_FINISH_*` codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReportFinishErrorKind {
    /// Extra top-level key on the POST body.
    UnknownField,
    /// Posted `panels` do not match the validated command list.
    PanelsMismatch,
    /// Posted `panels` JSON is not a well-formed panel array.
    PanelsInvalid,
    /// `comments` exceeds [`MAX_REVIEW_COMMENTS_CHARS`].
    CommentsTooLong,
    /// Body is not an object or required fields have the wrong type.
    InvalidJson,
    /// Session already completed (finish or dismiss).
    AlreadyComplete,
    /// Finish route ran without a review-mode capability token.
    ManifestRequired,
}

impl ReportFinishError {
    fn new(kind: ReportFinishErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    /// Stable machine code (`REPORT_FINISH_*`).
    pub(crate) fn code(&self) -> &'static str {
        self.kind.code()
    }

    /// Human-readable detail for the HTTP envelope.
    pub(crate) fn message(&self) -> &str {
        &self.message
    }

    /// Error class.
    pub(crate) fn kind(&self) -> ReportFinishErrorKind {
        self.kind
    }

    /// Why the request failed (RBP-001).
    pub(crate) fn cause(&self) -> &'static str {
        self.kind.cause()
    }

    /// What the caller should do next (RBP-001).
    pub(crate) fn recovery(&self) -> &'static str {
        self.kind.recovery()
    }
}

impl ReportFinishErrorKind {
    fn code(self) -> &'static str {
        match self {
            Self::UnknownField => "REPORT_FINISH_UNKNOWN_FIELD",
            Self::PanelsMismatch => "REPORT_FINISH_PANELS_MISMATCH",
            Self::PanelsInvalid => "REPORT_FINISH_PANELS_INVALID",
            Self::CommentsTooLong => "REPORT_FINISH_COMMENTS_TOO_LONG",
            Self::InvalidJson => "REPORT_FINISH_INVALID_JSON",
            Self::AlreadyComplete => "REPORT_FINISH_ALREADY_COMPLETE",
            Self::ManifestRequired => "REPORT_FINISH_MANIFEST_REQUIRED",
        }
    }

    fn cause(self) -> &'static str {
        match self {
            Self::UnknownField => "POST /api/report/finish rejects unknown top-level keys",
            Self::PanelsMismatch => {
                "posted panels must echo the authoritative report-command.json list"
            }
            Self::PanelsInvalid => {
                "each posted panel must be an object with path and optional label/role"
            }
            Self::CommentsTooLong => "comments exceeded the 32768-character contract bound",
            Self::InvalidJson => {
                "finish body must be a JSON object with approved, comments, and panels"
            }
            Self::AlreadyComplete => {
                "this one-shot report session already accepted a terminal action"
            }
            Self::ManifestRequired => {
                "finish requires a ValidatedReportManifest from a review-mode command"
            }
        }
    }

    fn recovery(self) -> &'static str {
        match self {
            Self::UnknownField => {
                "POST only approved, comments, and panels (copy panels from #manifest-data)"
            }
            Self::PanelsMismatch => "Resubmit the embedded manifest panels without edits",
            Self::PanelsInvalid => {
                "Fix panels[] shape (path string ending in .xhtml; optional string label/role)"
            }
            Self::CommentsTooLong => "Shorten comments to at most 32768 characters",
            Self::InvalidJson => "POST {\"approved\":true|false,\"comments\":\"…\",\"panels\":[…]}",
            Self::AlreadyComplete => {
                "Do not POST /api/report/finish or /api/result more than once per session"
            }
            Self::ManifestRequired => {
                "Open the report with mode review (wyvern report-xhtml --review)"
            }
        }
    }

    /// HTTP 409 for duplicate terminal actions; HTTP 400 otherwise.
    pub(crate) fn is_conflict(self) -> bool {
        matches!(self, Self::AlreadyComplete)
    }
}

impl std::fmt::Display for ReportFinishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

/// Parse and validate a finish POST body against the session capability token.
///
/// # Errors
///
/// Returns a [`ReportFinishError`] when the JSON is malformed, contains unknown
/// keys, comments are too long, or `panels` do not match `manifest`.
pub(crate) fn validate_finish_body(
    manifest: &ValidatedReportManifest,
    body: &Value,
) -> Result<ReportFinishData, ReportFinishError> {
    let obj = body.as_object().ok_or_else(|| {
        ReportFinishError::new(
            ReportFinishErrorKind::InvalidJson,
            "finish body must be a JSON object",
        )
    })?;
    for key in obj.keys() {
        if !FINISH_KEYS.contains(&key.as_str()) {
            return Err(ReportFinishError::new(
                ReportFinishErrorKind::UnknownField,
                format!("unknown field '{key}'"),
            ));
        }
    }

    let approved = match obj.get("approved") {
        Some(Value::Bool(flag)) => *flag,
        Some(other) => {
            return Err(ReportFinishError::new(
                ReportFinishErrorKind::InvalidJson,
                format!(
                    "field 'approved' expected boolean, got {}",
                    json_type_name(other)
                ),
            ));
        }
        None => {
            return Err(ReportFinishError::new(
                ReportFinishErrorKind::InvalidJson,
                "missing required field 'approved'",
            ));
        }
    };

    let comments = match obj.get("comments") {
        None | Some(Value::Null) => ReviewComments::new(""),
        Some(Value::String(text)) => ReviewComments::try_new(text.clone()).map_err(|_| {
            ReportFinishError::new(
                ReportFinishErrorKind::CommentsTooLong,
                format!("comments must be at most {MAX_REVIEW_COMMENTS_CHARS} characters"),
            )
        })?,
        Some(other) => {
            return Err(ReportFinishError::new(
                ReportFinishErrorKind::InvalidJson,
                format!(
                    "field 'comments' expected string, got {}",
                    json_type_name(other)
                ),
            ));
        }
    };

    let posted = match obj.get("panels") {
        Some(Value::Array(items)) => parse_posted_panels(items)?,
        Some(other) => {
            return Err(ReportFinishError::new(
                ReportFinishErrorKind::InvalidJson,
                format!(
                    "field 'panels' expected array, got {}",
                    json_type_name(other)
                ),
            ));
        }
        None => {
            return Err(ReportFinishError::new(
                ReportFinishErrorKind::InvalidJson,
                "missing required field 'panels'",
            ));
        }
    };

    if posted != manifest.panels() {
        return Err(ReportFinishError::new(
            ReportFinishErrorKind::PanelsMismatch,
            "posted panels do not match the authoritative report command",
        ));
    }

    Ok(ReportFinishData {
        approved,
        comments,
        panels: manifest.panels().to_vec(),
    })
}

fn parse_posted_panels(items: &[Value]) -> Result<Vec<ReportPanelEntry>, ReportFinishError> {
    let mut panels = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        panels.push(parse_posted_panel(index, item)?);
    }
    Ok(panels)
}

fn parse_posted_panel(index: usize, value: &Value) -> Result<ReportPanelEntry, ReportFinishError> {
    let obj = value.as_object().ok_or_else(|| {
        ReportFinishError::new(
            ReportFinishErrorKind::PanelsInvalid,
            format!("panels[{index}] must be an object"),
        )
    })?;
    for key in obj.keys() {
        if !matches!(key.as_str(), "path" | "label" | "role") {
            return Err(ReportFinishError::new(
                ReportFinishErrorKind::PanelsInvalid,
                format!("panels[{index}] unknown field '{key}'"),
            ));
        }
    }
    let path = match obj.get("path") {
        Some(Value::String(s)) => ManifestPanelPath::try_new(s.clone()).map_err(|_| {
            ReportFinishError::new(
                ReportFinishErrorKind::PanelsInvalid,
                format!("panels[{index}].path is not a valid .xhtml path"),
            )
        })?,
        _ => {
            return Err(ReportFinishError::new(
                ReportFinishErrorKind::PanelsInvalid,
                format!("panels[{index}].path must be a string"),
            ));
        }
    };
    let label = match obj.get("label") {
        None | Some(Value::Null) => None,
        Some(Value::String(s)) => Some(PanelLabel::try_new(s.clone()).map_err(|_| {
            ReportFinishError::new(
                ReportFinishErrorKind::PanelsInvalid,
                format!(
                    "panels[{index}].label must be a non-empty string of at most {MAX_PANEL_LABEL_CHARS} characters"
                ),
            )
        })?),
        Some(_) => {
            return Err(ReportFinishError::new(
                ReportFinishErrorKind::PanelsInvalid,
                format!("panels[{index}].label must be a string"),
            ));
        }
    };
    let role = match obj.get("role") {
        None | Some(Value::Null) => None,
        Some(Value::String(s)) => Some(PanelRole::parse(s).ok_or_else(|| {
            ReportFinishError::new(
                ReportFinishErrorKind::PanelsInvalid,
                format!("panels[{index}].role is not a known panel role"),
            )
        })?),
        Some(_) => {
            return Err(ReportFinishError::new(
                ReportFinishErrorKind::PanelsInvalid,
                format!("panels[{index}].role must be a string"),
            ));
        }
    };
    Ok(ReportPanelEntry { path, label, role })
}

fn json_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Error used when the finish route is reached without a review token.
pub(crate) fn manifest_required() -> ReportFinishError {
    ReportFinishError::new(
        ReportFinishErrorKind::ManifestRequired,
        "review finish requires validated command panels",
    )
}

/// Error used when `SessionState::complete` loses the result token.
pub(crate) fn already_complete() -> ReportFinishError {
    ReportFinishError::new(
        ReportFinishErrorKind::AlreadyComplete,
        "result already submitted",
    )
}

/// Error used when the POST body is not JSON.
pub(crate) fn invalid_json_parse(err: serde_json::Error) -> ReportFinishError {
    ReportFinishError::new(
        ReportFinishErrorKind::InvalidJson,
        format!("malformed finish JSON: {err}"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use wyvern_schema::{ReportCommand, ReportMode, ReportPagePath, ReportTitle};

    fn manifest() -> ValidatedReportManifest {
        let command = ReportCommand {
            title: ReportTitle::new("review"),
            page: ReportPagePath::new("pages/view.xhtml"),
            mode: ReportMode::Review,
            panels: Some(vec![ReportPanelEntry {
                path: ManifestPanelPath::new("panels/fail.xhtml"),
                label: Some("Fail 1".into()),
                role: Some(PanelRole::Failure),
            }]),
            width: None,
            height: None,
        };
        ValidatedReportManifest::from_review_command(&command).expect("token")
    }

    fn valid_body() -> Value {
        serde_json::json!({
            "approved": true,
            "comments": "looks good",
            "panels": [{
                "path": "panels/fail.xhtml",
                "label": "Fail 1",
                "role": "failure"
            }]
        })
    }

    #[test]
    fn accepts_matching_finish_and_echoes_authoritative_panels() {
        let data = validate_finish_body(&manifest(), &valid_body()).expect("ok");
        assert!(data.approved);
        assert_eq!(data.comments, "looks good");
        assert_eq!(data.panels[0].path.as_str(), "panels/fail.xhtml");
    }

    #[test]
    fn empty_comments_are_allowed() {
        let mut body = valid_body();
        body["comments"] = serde_json::json!("");
        let data = validate_finish_body(&manifest(), &body).expect("ok");
        assert_eq!(data.comments, "");
        body.as_object_mut().expect("obj").remove("comments");
        let data = validate_finish_body(&manifest(), &body).expect("ok");
        assert_eq!(data.comments, "");
    }

    #[test]
    fn unknown_top_level_key_is_rejected() {
        let mut body = valid_body();
        body["extra"] = serde_json::json!(1);
        let err = validate_finish_body(&manifest(), &body).expect_err("unknown");
        assert_eq!(err.kind(), ReportFinishErrorKind::UnknownField);
        assert_eq!(err.code(), "REPORT_FINISH_UNKNOWN_FIELD");
    }

    #[test]
    fn comments_bound_is_enforced() {
        let mut body = valid_body();
        body["comments"] = Value::String("x".repeat(MAX_REVIEW_COMMENTS_CHARS + 1));
        let err = validate_finish_body(&manifest(), &body).expect_err("long");
        assert_eq!(err.kind(), ReportFinishErrorKind::CommentsTooLong);
        assert_eq!(err.code(), "REPORT_FINISH_COMMENTS_TOO_LONG");
    }

    #[test]
    fn panels_mismatch_is_rejected() {
        let mut body = valid_body();
        body["panels"] = serde_json::json!([{
            "path": "panels/other.xhtml",
            "label": "Fail 1",
            "role": "failure"
        }]);
        let err = validate_finish_body(&manifest(), &body).expect_err("mismatch");
        assert_eq!(err.kind(), ReportFinishErrorKind::PanelsMismatch);
        assert_eq!(err.code(), "REPORT_FINISH_PANELS_MISMATCH");
        assert_eq!(
            err.recovery(),
            "Resubmit the embedded manifest panels without edits"
        );
    }

    #[test]
    fn panel_shape_errors_use_panels_invalid_not_mismatch() {
        let mut body = valid_body();
        body["panels"] = serde_json::json!(["not-an-object"]);
        let err = validate_finish_body(&manifest(), &body).expect_err("shape");
        assert_eq!(err.kind(), ReportFinishErrorKind::PanelsInvalid);
        assert_eq!(err.code(), "REPORT_FINISH_PANELS_INVALID");
        assert_ne!(
            err.recovery(),
            "Resubmit the embedded manifest panels without edits"
        );

        body["panels"] = serde_json::json!([{ "path": 1 }]);
        let err = validate_finish_body(&manifest(), &body).expect_err("path type");
        assert_eq!(err.kind(), ReportFinishErrorKind::PanelsInvalid);

        body["panels"] = serde_json::json!([{ "path": "panels/fail.html" }]);
        let err = validate_finish_body(&manifest(), &body).expect_err("suffix");
        assert_eq!(err.kind(), ReportFinishErrorKind::PanelsInvalid);
    }

    #[test]
    fn manifest_required_has_dedicated_code() {
        let err = manifest_required();
        assert_eq!(err.kind(), ReportFinishErrorKind::ManifestRequired);
        assert_eq!(err.code(), "REPORT_FINISH_MANIFEST_REQUIRED");
        assert_ne!(err.code(), "REPORT_FINISH_INVALID_JSON");
    }
}
