//! Wizard HTTP routes — state, navigate, finish (d.1–d.2).

use axum::extract::State;
use axum::Json;
use serde_json::Value;
use tracing::{event, Level};
use wyvern_schema::{
    Command, WizardFinishRequest, WizardNavAction, WizardNavigateRequest, WizardNavigateResponse,
    WizardStateResponse,
};
use wyvern_wizard::WizardError;

use crate::routes::api_error::ApiError;
use crate::session::SessionState;

/// Docs pointer for wizard state route errors.
const WIZARD_STATE_DOCS: &str =
    "docs/plans/phase-C/http-wizard-contract.md (GET /api/wizard/state)";

/// Docs pointer for navigate route errors.
const WIZARD_NAVIGATE_DOCS: &str =
    "docs/plans/phase-C/http-wizard-contract.md (POST /api/wizard/navigate)";

/// Docs pointer for finish route errors.
const WIZARD_FINISH_DOCS: &str =
    "docs/plans/phase-C/http-wizard-contract.md (POST /api/wizard/finish)";

/// Return the current wizard stack snapshot as [`WizardStateResponse`].
pub async fn get_wizard_state(
    State(session): State<SessionState>,
) -> Result<Json<WizardStateResponse>, ApiError> {
    let command = session.command().await;
    let Command::Wizard(wizard_cmd) = command.as_ref() else {
        let session_type = command_type_name(&command);
        event!(
            name: "wizard.state.bad_request",
            Level::WARN,
            route = "/api/wizard/state",
            error_class = "bad_request",
            session_type,
            "GET /api/wizard/state requires an active wizard session"
        );
        return Err(ApiError::bad_request(
            "GET /api/wizard/state requires an active wizard session",
        )
        .cause("the one-shot host session is not a wizard command")
        .recovery("Open a type: wizard command before calling /api/wizard/state")
        .docs(WIZARD_STATE_DOCS));
    };

    let snapshot = session.wizard_snapshot().await.map_err(|err| {
        event!(
            name: "wizard.state.snapshot_failed",
            Level::WARN,
            route = "/api/wizard/state",
            error_class = "internal",
            session_type = "wizard",
            error = %err,
            "wizard snapshot failed"
        );
        ApiError::internal(err.to_string())
            .cause("wizard session was not initialized for this dialog")
            .recovery("Report a bug if a validated wizard command has no session")
            .docs(WIZARD_STATE_DOCS)
    })?;

    event!(
        name: "wizard.state.ok",
        Level::DEBUG,
        route = "/api/wizard/state",
        "wizard state snapshot returned"
    );

    Ok(Json(WizardStateResponse::from_snapshot(
        snapshot.config,
        snapshot.page,
        snapshot.page_data,
        snapshot.stack,
        wizard_cmd.width,
        wizard_cmd.height,
    )))
}

/// `POST /api/wizard/navigate` — non-terminal `next` / `back`.
pub async fn post_wizard_navigate(
    State(session): State<SessionState>,
    Json(body): Json<Value>,
) -> Result<Json<WizardNavigateResponse>, ApiError> {
    require_wizard_session(&session, "/api/wizard/navigate", WIZARD_NAVIGATE_DOCS).await?;

    let request: WizardNavigateRequest = serde_json::from_value(body).map_err(|err| {
        navigate_bad_request(
            format!("invalid navigate body: {err}"),
            "POST /api/wizard/navigate JSON failed to deserialize",
        )
    })?;

    let (_outcome, url) = match request.action {
        WizardNavAction::Next => {
            let next = request.next.ok_or_else(|| {
                navigate_bad_request(
                    "missing field 'next' for action next",
                    "action next requires a WizardPageDescriptor in 'next'",
                )
            })?;
            if let Some(ref page_id) = request.page_id {
                if page_id != &next.id {
                    return Err(navigate_bad_request(
                        "page_id does not match next.id",
                        format!(
                            "page_id '{}' != next.id '{}'",
                            page_id.as_str(),
                            next.id.as_str()
                        ),
                    ));
                }
            }
            session
                .wizard_navigate_next(request.data, next)
                .await
                .map_err(map_wizard_navigate_error)?
        }
        WizardNavAction::Back => session
            .wizard_navigate_back(request.data)
            .await
            .map_err(map_wizard_navigate_error)?,
    };

    event!(
        name: "wizard.navigate.ok",
        Level::DEBUG,
        route = "/api/wizard/navigate",
        "wizard navigate succeeded"
    );

    Ok(Json(WizardNavigateResponse { ok: true, url }))
}

/// `POST /api/wizard/finish` — terminal finish / cancel / dismissed; body = stdout.
pub async fn post_wizard_finish(
    State(session): State<SessionState>,
    Json(body): Json<Value>,
) -> Result<Json<wyvern_schema::WizardResult>, ApiError> {
    require_wizard_session(&session, "/api/wizard/finish", WIZARD_FINISH_DOCS).await?;

    let request: WizardFinishRequest = serde_json::from_value(body).map_err(|err| {
        finish_bad_request(
            format!("invalid finish body: {err}"),
            "POST /api/wizard/finish JSON failed to deserialize",
        )
    })?;

    let result = session
        .wizard_finish(request.button, request.data, request.stack)
        .await
        .map_err(map_wizard_finish_error)?;

    let Some(result) = result else {
        event!(
            name: "wizard.finish.conflict",
            Level::WARN,
            route = "/api/wizard/finish",
            error_class = "conflict",
            "wizard finish rejected; result already submitted"
        );
        return Err(ApiError::conflict("result already submitted")
            .cause("a finish/result was already accepted for this one-shot wizard session")
            .recovery("Do not POST /api/wizard/finish more than once per dialog")
            .docs(WIZARD_FINISH_DOCS));
    };

    event!(
        name: "wizard.finish.ok",
        Level::DEBUG,
        route = "/api/wizard/finish",
        "wizard finish accepted"
    );

    Ok(Json(result))
}

async fn require_wizard_session(
    session: &SessionState,
    route: &'static str,
    docs: &'static str,
) -> Result<(), ApiError> {
    let command = session.command().await;
    if matches!(command.as_ref(), Command::Wizard(_)) {
        return Ok(());
    }
    let session_type = command_type_name(&command);
    event!(
        name: "wizard.route.bad_request",
        Level::WARN,
        route,
        error_class = "bad_request",
        session_type,
        "wizard route requires an active wizard session"
    );
    Err(
        ApiError::bad_request(format!("{route} requires an active wizard session"))
            .cause("the one-shot host session is not a wizard command")
            .recovery("Open a type: wizard command before calling wizard HTTP routes")
            .docs(docs),
    )
}

fn map_wizard_navigate_error(err: WizardError) -> ApiError {
    event!(
        name: "wizard.navigate.error",
        Level::WARN,
        error_class = err.subcode(),
        error = %err,
        "wizard navigate failed"
    );
    match &err {
        WizardError::AtFirstPage => ApiError::bad_request(err.to_string())
            .cause("navigate_back was called while cursor was already 0")
            .recovery("Disable Back on the first page, or ignore the 400 in page JS")
            .docs(WIZARD_NAVIGATE_DOCS),
        WizardError::InvalidCommand { field, reason } => ApiError::bad_request(err.to_string())
            .cause(format!("invalid field '{field}': {reason}"))
            .recovery("POST action next|back with a valid next descriptor when advancing")
            .docs(WIZARD_NAVIGATE_DOCS),
        WizardError::StackDepthExceeded { max } => ApiError::bad_request(err.to_string())
            .cause(format!(
                "navigate_next would exceed max stack depth of {max}"
            ))
            .recovery("Finish the wizard or navigate back before adding more pages")
            .docs(WIZARD_NAVIGATE_DOCS),
        WizardError::SessionNotInitialized => ApiError::internal(err.to_string())
            .cause("wizard session missing for an active wizard command")
            .recovery("Report a bug if a validated wizard command has no session")
            .docs(WIZARD_NAVIGATE_DOCS),
        WizardError::PublicOriginNotSet => ApiError::internal(err.to_string())
            .cause("public origin was not recorded after the HTTP listener bound")
            .recovery("Report a bug — navigate URLs require the bound loopback origin")
            .docs(WIZARD_NAVIGATE_DOCS),
        WizardError::ResultAlreadySubmitted => ApiError::conflict(err.to_string())
            .cause("a finish/result was already accepted for this one-shot wizard session")
            .recovery("Do not POST /api/wizard/navigate after the dialog result is submitted")
            .docs(WIZARD_NAVIGATE_DOCS),
        WizardError::StackMismatch { .. } => ApiError::bad_request(err.to_string())
            .cause("stack mismatch is not expected on navigate")
            .recovery("Use POST /api/wizard/finish for terminal actions")
            .docs(WIZARD_NAVIGATE_DOCS),
    }
}

fn map_wizard_finish_error(err: WizardError) -> ApiError {
    event!(
        name: "wizard.finish.error",
        Level::WARN,
        error_class = err.subcode(),
        error = %err,
        "wizard finish failed"
    );
    match &err {
        WizardError::StackMismatch {
            index,
            expected_page_id,
            got_page_id,
            reason,
        } => {
            let mut cause = format!(
                "client stack does not equal session-derived full visited stack ({reason})"
            );
            if let Some(i) = index {
                cause.push_str(&format!("; index={i}"));
            }
            if let (Some(expected), Some(got)) = (expected_page_id, got_page_id) {
                cause.push_str(&format!(
                    "; expected_page_id={}; got_page_id={}",
                    expected.as_str(),
                    got.as_str()
                ));
            }
            ApiError::bad_request(err.to_string())
                .cause(cause)
                .recovery(
                    "Build stack as window.wyvern.stack plus { page, data } for the current page",
                )
                .docs(WIZARD_FINISH_DOCS)
        }
        WizardError::InvalidCommand { field, reason } => ApiError::bad_request(err.to_string())
            .cause(format!("invalid field '{field}': {reason}"))
            .recovery("Use button finish|cancel|dismissed only on /api/wizard/finish")
            .docs(WIZARD_FINISH_DOCS),
        WizardError::AtFirstPage => ApiError::bad_request(err.to_string())
            .cause("AtFirstPage is not expected on finish")
            .recovery("Report a bug")
            .docs(WIZARD_FINISH_DOCS),
        WizardError::StackDepthExceeded { .. } => ApiError::bad_request(err.to_string())
            .cause("StackDepthExceeded is not expected on finish")
            .recovery("Report a bug")
            .docs(WIZARD_FINISH_DOCS),
        WizardError::SessionNotInitialized => ApiError::internal(err.to_string())
            .cause("wizard session missing for an active wizard command")
            .recovery("Report a bug if a validated wizard command has no session")
            .docs(WIZARD_FINISH_DOCS),
        WizardError::PublicOriginNotSet => ApiError::internal(err.to_string())
            .cause("PublicOriginNotSet is not expected on finish")
            .recovery("Report a bug")
            .docs(WIZARD_FINISH_DOCS),
        WizardError::ResultAlreadySubmitted => ApiError::conflict(err.to_string())
            .cause("a finish/result was already accepted for this one-shot wizard session")
            .recovery("Do not POST /api/wizard/finish more than once per dialog")
            .docs(WIZARD_FINISH_DOCS),
    }
}

fn navigate_bad_request(message: impl Into<String>, cause: impl Into<String>) -> ApiError {
    ApiError::bad_request(message)
        .cause(cause)
        .recovery("POST { action: \"next\"|\"back\", data, next? } — cancel uses /finish")
        .docs(WIZARD_NAVIGATE_DOCS)
}

fn finish_bad_request(message: impl Into<String>, cause: impl Into<String>) -> ApiError {
    ApiError::bad_request(message)
        .cause(cause)
        .recovery("POST { button: \"finish\"|\"cancel\"|\"dismissed\", data, stack }")
        .docs(WIZARD_FINISH_DOCS)
}

fn command_type_name(command: &Command) -> &'static str {
    match command {
        Command::Chrome { .. } => "chrome",
        Command::Message { .. } => "message",
        Command::Input { .. } => "input",
        Command::Markdown { .. } => "markdown",
        Command::Question { .. } => "question",
        Command::Wizard(_) => "wizard",
    }
}
