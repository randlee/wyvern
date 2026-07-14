//! `POST /api/result` — accept stdout-shaped JSON and complete the session.

use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use wyvern_schema::{ButtonLabel, Command, CommandResult, InputResult, InputValue, MessageResult};

use crate::error::HostError;
use crate::routes::api_error::ApiError;
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
) -> Result<Json<ResultAck>, ApiError> {
    let command = session.command().await;
    let result = parse_result_for_command(&command, &body).map_err(|err| {
        // Surface the structured HostError::InvalidResult path (stable message
        // for clients; CLI emit_host_error maps the same variant).
        ApiError::bad_request(err.to_string())
    })?;
    if !session.complete(result).await {
        return Err(ApiError::conflict("result already submitted"));
    }
    Ok(Json(ResultAck { ok: true }))
}

fn parse_result_for_command(command: &Command, body: &Value) -> Result<CommandResult, HostError> {
    match command {
        Command::Message { .. } => {
            let button = body.get("button").and_then(Value::as_str).ok_or_else(|| {
                HostError::InvalidResult {
                    message: "missing string field 'button'".into(),
                }
            })?;
            Ok(CommandResult::Message(MessageResult {
                button: ButtonLabel::new(button),
            }))
        }
        Command::Input { .. } => parse_input_result(body),
        _ => Err(HostError::InvalidResult {
            message: "active dialog type does not accept results yet".into(),
        }),
    }
}

fn parse_input_result(body: &Value) -> Result<CommandResult, HostError> {
    let button =
        body.get("button")
            .and_then(Value::as_str)
            .ok_or_else(|| HostError::InvalidResult {
                message: "missing string field 'button'".into(),
            })?;
    let input = match body.get("input") {
        None | Some(Value::Null) => None,
        Some(Value::String(s)) => Some(InputValue::Text(s.clone())),
        Some(Value::Array(items)) => {
            let mut paths = Vec::with_capacity(items.len());
            for (i, item) in items.iter().enumerate() {
                match item {
                    Value::String(s) => paths.push(s.clone()),
                    other => {
                        return Err(HostError::InvalidResult {
                            message: format!(
                                "input[{i}] expected string, got {}",
                                json_type_name(other)
                            ),
                        });
                    }
                }
            }
            Some(InputValue::Paths(paths))
        }
        Some(other) => {
            return Err(HostError::InvalidResult {
                message: format!(
                    "field 'input' expected string or array, got {}",
                    json_type_name(other)
                ),
            });
        }
    };
    Ok(CommandResult::Input(InputResult {
        button: ButtonLabel::new(button),
        input,
    }))
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
