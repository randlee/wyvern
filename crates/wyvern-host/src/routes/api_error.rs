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
    cause: Option<String>,
    recovery: Vec<String>,
    docs: Option<String>,
}

impl ApiError {
    /// Build an API error with an explicit status and message.
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
            cause: None,
            recovery: Vec::new(),
            docs: None,
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

    /// Attach a cause string (RBP error-context contract).
    pub fn cause(mut self, cause: impl Into<String>) -> Self {
        self.cause = Some(cause.into());
        self
    }

    /// Append one recovery step.
    pub fn recovery(mut self, step: impl Into<String>) -> Self {
        self.recovery.push(step.into());
        self
    }

    /// Attach a docs pointer (path or URL).
    pub fn docs(mut self, docs: impl Into<String>) -> Self {
        self.docs = Some(docs.into());
        self
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
    /// Why the failure occurred (when known).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cause: Option<String>,
    /// Actionable recovery steps.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub recovery: Vec<String>,
    /// Pointer to contract / requirements docs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs: Option<String>,
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
            cause: self.cause,
            recovery: self.recovery,
            docs: self.docs,
        };
        (status, Json(body)).into_response()
    }
}
