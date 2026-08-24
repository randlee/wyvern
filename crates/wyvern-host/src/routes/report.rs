//! Report HTTP helpers (Phase H / ADR-0025).
//!
//! Static pages are served via `ServeDir` nest at `/report` from session
//! `ui_root` (see [`crate::server::build_router`]). `POST /api/report/finish`
//! is registered only when `mode: "review"`; payload validation lands in h.3.

use axum::extract::State;
use axum::Json;

use crate::routes::api_error::ApiError;
use crate::routes::result::ResultAck;
use crate::session::SessionState;

/// Docs pointer for report-route errors.
const REPORT_FINISH_DOCS: &str = "docs/plans/phase-H/xhtml-reporting-contract.md";

/// Review-mode terminal action. Strict finish validation is sprint h.3 (REQ-0144).
pub async fn post_report_finish(
    State(_session): State<SessionState>,
    Json(_body): Json<serde_json::Value>,
) -> Result<Json<ResultAck>, ApiError> {
    Err(ApiError::bad_request(
        "POST /api/report/finish validation lands in sprint h.3",
    )
    .cause("review finish is registered in review mode but payload checks are not implemented yet")
    .recovery("Use view-mode dismiss: close the window or POST /api/result {\"button\":\"dismissed\"}")
    .docs(REPORT_FINISH_DOCS))
}
