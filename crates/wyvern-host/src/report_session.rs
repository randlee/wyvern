//! Report-session capability tokens (Phase H / REQ-HOST-0142).
//!
//! [`ValidatedReportManifest`] is issued only for review-mode commands that
//! already passed schema validation with a non-empty `panels` list. The finish
//! handler requires this token so posted `panels` are checked against the
//! command JSON, not free-form client input (RBP-010).

use wyvern_schema::{ReportCommand, ReportMode, ReportPanelEntry};

/// Proof that this session's report command has validated review panels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ValidatedReportManifest {
    panels: Vec<ReportPanelEntry>,
}

impl ValidatedReportManifest {
    /// Issue the token when `command` is review mode with validated panels.
    pub(crate) fn from_review_command(command: &ReportCommand) -> Option<Self> {
        if command.mode != ReportMode::Review {
            return None;
        }
        let panels = command.panels.as_ref()?.clone();
        if panels.is_empty() {
            return None;
        }
        Some(Self { panels })
    }

    /// Authoritative panel list from the validated command JSON.
    pub(crate) fn panels(&self) -> &[ReportPanelEntry] {
        &self.panels
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wyvern_schema::{ManifestPanelPath, ReportPagePath, ReportTitle};

    fn review_command(panels: Option<Vec<ReportPanelEntry>>) -> ReportCommand {
        ReportCommand {
            title: ReportTitle::new("review"),
            page: ReportPagePath::new("pages/view.xhtml"),
            mode: ReportMode::Review,
            panels,
            width: None,
            height: None,
        }
    }

    #[test]
    fn token_issued_only_for_review_with_panels() {
        let panel = ReportPanelEntry {
            path: ManifestPanelPath::new("panels/fail.xhtml"),
            label: Some("Fail 1".into()),
            role: None,
        };
        let token =
            ValidatedReportManifest::from_review_command(&review_command(Some(
                vec![panel.clone()],
            )))
            .expect("review+panels");
        assert_eq!(token.panels(), &[panel]);

        let mut view = review_command(Some(vec![ReportPanelEntry {
            path: ManifestPanelPath::new("panels/fail.xhtml"),
            label: None,
            role: None,
        }]));
        view.mode = ReportMode::View;
        assert!(ValidatedReportManifest::from_review_command(&view).is_none());
        assert!(ValidatedReportManifest::from_review_command(&review_command(None)).is_none());
        assert!(
            ValidatedReportManifest::from_review_command(&review_command(Some(Vec::new())))
                .is_none()
        );
    }
}
