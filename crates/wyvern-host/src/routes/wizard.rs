//! Wizard HTTP routes — `GET /api/wizard/state` (d.1).

use axum::extract::State;
use axum::Json;
use wyvern_schema::{Command, WizardStateResponse};

use crate::routes::api_error::ApiError;
use crate::session::SessionState;

/// Docs pointer for wizard state route errors.
const WIZARD_STATE_DOCS: &str =
    "docs/plans/phase-C/http-wizard-contract.md (GET /api/wizard/state)";

/// Return the current wizard stack snapshot as [`WizardStateResponse`].
pub async fn get_wizard_state(
    State(session): State<SessionState>,
) -> Result<Json<WizardStateResponse>, ApiError> {
    let command = session.command().await;
    let Command::Wizard(ref wizard_cmd) = command else {
        return Err(ApiError::bad_request(
            "GET /api/wizard/state requires an active wizard session",
        )
        .cause("the one-shot host session is not a wizard command")
        .recovery("Open a type: wizard command before calling /api/wizard/state")
        .docs(WIZARD_STATE_DOCS));
    };

    let snapshot = session.wizard_snapshot().await.map_err(|message| {
        ApiError::internal(message)
            .cause("wizard session was not initialized for this dialog")
            .recovery("Report a bug if a validated wizard command has no session")
            .docs(WIZARD_STATE_DOCS)
    })?;

    Ok(Json(WizardStateResponse::from_snapshot(
        snapshot.config,
        snapshot.page,
        snapshot.page_data,
        snapshot.stack,
        wizard_cmd.width,
        wizard_cmd.height,
    )))
}
