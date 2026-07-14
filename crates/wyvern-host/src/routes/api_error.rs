//! Structured JSON error envelope for HTTP API routes.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

/// Opaque HTTP failure returned as a JSON [`ApiErrorBody`] (not bare `String`).
#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    /// Build an API error with an explicit status and message.
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    /// HTTP 400 — invalid request for the active dialog.
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }

    /// HTTP 409 — result already submitted.
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(StatusCode::CONFLICT, message)
    }

    /// HTTP 503 — session closed / picker unavailable.
    pub fn service_unavailable(message: impl Into<String>) -> Self {
        Self::new(StatusCode::SERVICE_UNAVAILABLE, message)
    }

    /// HTTP 500 — internal picker / server fault.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
    }

    /// HTTP 504 — native picker timed out.
    pub fn gateway_timeout(message: impl Into<String>) -> Self {
        Self::new(StatusCode::GATEWAY_TIMEOUT, message)
    }
}

/// Wire body for failed `/api/*` responses.
#[derive(Debug, Serialize)]
pub struct ApiErrorBody {
    /// Always `false` on error responses.
    pub ok: bool,
    /// Stable machine-oriented error class derived from the HTTP status.
    pub error: &'static str,
    /// Human-readable detail.
    pub message: String,
}

fn error_class(status: StatusCode) -> &'static str {
    match status.as_u16() {
        400 => "bad_request",
        409 => "conflict",
        503 => "service_unavailable",
        504 => "gateway_timeout",
        500 => "internal",
        _ => "error",
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status;
        let body = ApiErrorBody {
            ok: false,
            error: error_class(status),
            message: self.message,
        };
        (status, Json(body)).into_response()
    }
}
