//! `POST /api/result` — accept stdout-shaped JSON and complete the session.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use wyvern_schema::{ButtonLabel, Command, CommandResult, MessageResult};

use crate::session::SessionState;

/// Success ack returned to the page after accepting a result.
#[derive(Debug, Serialize, Deserialize)]
pub struct ResultAck {
    /// Always `true` on HTTP 200.
    pub ok: bool,
}

/// Accept a POST body matching the active dialog's [`CommandResult`] wire shape.
pub async fn post_result(
    State(session): State<SessionState>,
    Json(body): Json<Value>,
) -> Result<Json<ResultAck>, (StatusCode, String)> {
    let command = session.command().await;
    let result = parse_result_for_command(&command, &body).map_err(|message| {
        (
            StatusCode::BAD_REQUEST,
            format!("invalid result: {message}"),
        )
    })?;
    if !session.complete(result).await {
        return Err((StatusCode::CONFLICT, "result already submitted".to_string()));
    }
    Ok(Json(ResultAck { ok: true }))
}

fn parse_result_for_command(command: &Command, body: &Value) -> Result<CommandResult, String> {
    match command {
        Command::Message { .. } => {
            let button = body
                .get("button")
                .and_then(Value::as_str)
                .ok_or_else(|| "missing string field 'button'".to_string())?;
            Ok(CommandResult::Message(MessageResult {
                button: ButtonLabel::new(button),
            }))
        }
        _ => Err("active dialog type does not accept results yet".into()),
    }
}
