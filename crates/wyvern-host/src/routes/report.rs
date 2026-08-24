//! Report HTTP helpers (Phase H / ADR-0025).
//!
//! Static pages are served via `ServeDir` nest at `/report` from session
//! `ui_root` (see [`crate::server::build_router`]). `POST /api/report/finish`
//! is registered only when `mode: "review"`.

use axum::body::Bytes;
use axum::extract::State;
use axum::Json;
use tracing::{event, Level};
use wyvern_schema::{CommandResult, ReportResult};

use crate::report_finish::{
    already_complete, invalid_json_parse, manifest_required, validate_finish_body,
    ReportFinishError,
};
use crate::routes::api_error::ApiError;
use crate::routes::result::ResultAck;
use crate::session::SessionState;

/// Docs pointer for report-route errors.
const REPORT_FINISH_DOCS: &str = "docs/plans/phase-H/xhtml-reporting-contract.md";

/// Review-mode terminal action (Approve / Cancel) → stdout finish JSON.
pub async fn post_report_finish(
    State(session): State<SessionState>,
    body: Bytes,
) -> Result<Json<ResultAck>, ApiError> {
    let parsed: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(value) => value,
        Err(err) => {
            let finish_err = invalid_json_parse(err);
            event!(
                name: "report.finish.bad_request",
                Level::WARN,
                route = "/api/report/finish",
                error_class = "bad_request",
                code = finish_err.code(),
                "POST /api/report/finish body is not JSON"
            );
            return Err(finish_error(&finish_err));
        }
    };

    let Some(manifest) = session.validated_report_manifest().await else {
        event!(
            name: "report.finish.bad_request",
            Level::WARN,
            route = "/api/report/finish",
            error_class = "bad_request",
            code = manifest_required().code(),
            "review finish missing ValidatedReportManifest"
        );
        return Err(finish_error(&manifest_required()));
    };

    let data = match validate_finish_body(&manifest, &parsed) {
        Ok(data) => data,
        Err(err) => {
            event!(
                name: "report.finish.bad_request",
                Level::WARN,
                route = "/api/report/finish",
                error_class = "bad_request",
                code = err.code(),
                error = %err,
                "POST /api/report/finish body failed validation"
            );
            return Err(finish_error(&err));
        }
    };

    if !session
        .complete(CommandResult::Report(ReportResult::finished(data)))
        .await
    {
        let err = already_complete();
        event!(
            name: "report.finish.conflict",
            Level::WARN,
            route = "/api/report/finish",
            error_class = "conflict",
            code = err.code(),
            "finish rejected; session already closed or submitted"
        );
        return Err(finish_error(&err));
    }

    event!(
        name: "report.finish.ok",
        Level::DEBUG,
        route = "/api/report/finish",
        "review finish accepted"
    );
    Ok(Json(ResultAck { ok: true }))
}

fn finish_error(err: &ReportFinishError) -> ApiError {
    let api = if err.kind().is_conflict() {
        ApiError::conflict(err.message())
    } else {
        ApiError::bad_request(err.message())
    };
    api.code(err.code())
        .cause(err.cause())
        .recovery(err.recovery())
        .docs(REPORT_FINISH_DOCS)
}
