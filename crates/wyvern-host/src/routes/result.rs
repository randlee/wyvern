//! `POST /api/result` — accept stdout-shaped JSON and complete the session.

use std::collections::HashMap;

use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use wyvern_schema::{
    ButtonLabel, ChromeResult, Command, CommandResult, InputResult, InputValue, MarkdownResult,
    MessageResult, QuestionCard, QuestionResult,
};

use crate::error::HostError;
use crate::routes::api_error::ApiError;
use crate::session::SessionState;

/// Docs pointer for result route errors (RBP error-context contract).
const RESULT_DOCS: &str = "docs/plans/phase-C/http-post-schema.md (POST /api/result)";

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
        // for clients; CLI emit_host_error maps the same variant) with RBP
        // cause/recovery/docs (c.11 picker contract parity).
        result_bad_request(
            err.to_string(),
            format!("POST /api/result body failed validation for the active dialog: {err}"),
        )
    })?;
    if !session.complete(result).await {
        return Err(ApiError::conflict("result already submitted")
            .cause("a result was already accepted for this one-shot dialog session")
            .recovery("Do not POST /api/result more than once per dialog")
            .docs(RESULT_DOCS));
    }
    Ok(Json(ResultAck { ok: true }))
}

fn result_bad_request(message: impl Into<String>, cause: impl Into<String>) -> ApiError {
    ApiError::bad_request(message)
        .cause(cause)
        .recovery("POST a JSON body matching the active dialog's result wire shape")
        .recovery(
            "For markdown/message results include a string 'button' field (e.g. \"ok\" or \"dismissed\")",
        )
        .recovery(
            "For question submit omit 'button' and include questions, non-empty answers, and response",
        )
        .recovery(
            "Question answer keys must match the active dialog's question prompts",
        )
        .docs(RESULT_DOCS)
}

fn parse_result_for_command(command: &Command, body: &Value) -> Result<CommandResult, HostError> {
    match command {
        Command::Chrome { .. } => {
            let button = body.get("button").and_then(Value::as_str).ok_or_else(|| {
                HostError::InvalidResult {
                    message: "missing string field 'button'".into(),
                }
            })?;
            Ok(CommandResult::Chrome(ChromeResult {
                button: ButtonLabel::new(button),
            }))
        }
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
        Command::Markdown { .. } => {
            let button = body.get("button").and_then(Value::as_str).ok_or_else(|| {
                HostError::InvalidResult {
                    message: "missing string field 'button'".into(),
                }
            })?;
            Ok(CommandResult::Markdown(MarkdownResult {
                button: ButtonLabel::new(button),
            }))
        }
        Command::Input { .. } => parse_input_result(body),
        Command::Question {
            questions,
            questions_raw,
        } => parse_question_result(questions, questions_raw, body),
    }
}

/// Allowed top-level keys for `question` `POST /api/result` (http-post-schema.md).
const QUESTION_RESULT_KEYS: &[&str] = &["questions", "answers", "response", "button"];

/// Parse question POST body per http-post-schema.md (REQ-0067 / REQ-0068).
///
/// - Normal submit: omit `button`, non-empty `answers`, `response` present.
/// - Presence of `button`, or empty `answers` without submit → fail-safe dismiss.
/// - Stdout `questions` always echo the validated command `questions_raw`.
/// - Answer map keys must be prompts from the active [`QuestionCard`] set.
/// - Unknown top-level keys → 400 (Extra fields convention).
fn parse_question_result(
    questions: &[QuestionCard],
    questions_raw: &[Value],
    body: &Value,
) -> Result<CommandResult, HostError> {
    reject_unknown_keys(body, QUESTION_RESULT_KEYS)?;
    if !body.get("questions").map(Value::is_array).unwrap_or(false) {
        return Err(HostError::InvalidResult {
            message: "missing array field 'questions'".into(),
        });
    }
    if body.get("response").and_then(Value::as_str).is_none() {
        return Err(HostError::InvalidResult {
            message: "missing string field 'response'".into(),
        });
    }
    let answers = ValidatedAnswers::parse(questions, body)?;
    let has_button = body.get("button").is_some();

    // Fail-safe dismiss: any `button` field, or empty answers on "submit".
    if has_button || answers.is_empty() {
        return Ok(CommandResult::Question(QuestionResult::dismissed(
            questions_raw.to_vec(),
        )));
    }

    Ok(CommandResult::Question(QuestionResult::submitted(
        questions_raw.to_vec(),
        answers.into_map(),
    )))
}

/// Answer map whose keys were validated against active question prompts at parse time (RBP-F006).
#[derive(Debug, Clone)]
struct ValidatedAnswers(HashMap<String, String>);

impl ValidatedAnswers {
    fn parse(questions: &[QuestionCard], body: &Value) -> Result<Self, HostError> {
        let answers = parse_question_answers(body)?;
        validate_answer_keys(questions, &answers)?;
        Ok(Self(answers))
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn into_map(self) -> HashMap<String, String> {
        self.0
    }
}

/// Reject unknown top-level object keys (http-post-schema.md Extra fields → 400).
fn reject_unknown_keys(body: &Value, allowed: &[&str]) -> Result<(), HostError> {
    let Some(obj) = body.as_object() else {
        return Err(HostError::InvalidResult {
            message: "result body must be a JSON object".into(),
        });
    };
    for key in obj.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(HostError::InvalidResult {
                message: format!("unknown field '{key}'"),
            });
        }
    }
    Ok(())
}

/// Reject answer keys that are not prompts on the active question cards.
fn validate_answer_keys(
    questions: &[QuestionCard],
    answers: &HashMap<String, String>,
) -> Result<(), HostError> {
    if answers.is_empty() {
        return Ok(());
    }
    let allowed: std::collections::HashSet<&str> = questions
        .iter()
        .map(|card| card.question.as_str())
        .collect();
    for key in answers.keys() {
        if !allowed.contains(key.as_str()) {
            return Err(HostError::InvalidResult {
                message: format!(
                    "answers key {key:?} is not a question prompt on the active dialog"
                ),
            });
        }
    }
    Ok(())
}

fn parse_question_answers(body: &Value) -> Result<HashMap<String, String>, HostError> {
    let Some(obj) = body.get("answers").and_then(Value::as_object) else {
        return Err(HostError::InvalidResult {
            message: "missing object field 'answers'".into(),
        });
    };
    let mut answers = HashMap::with_capacity(obj.len());
    for (key, value) in obj {
        match value {
            Value::String(s) => {
                answers.insert(key.clone(), s.clone());
            }
            other => {
                return Err(HostError::InvalidResult {
                    message: format!(
                        "answers[{key:?}] expected string, got {}",
                        json_type_name(other)
                    ),
                });
            }
        }
    }
    Ok(answers)
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
