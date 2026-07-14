//! `GET /api/dialog` — JSON payload for the active command.

use std::time::Duration;

use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};
use wyvern_schema::{ButtonsPreset, Command, QuestionCard, MARKDOWN_CONTENT_MAX_BYTES};

use crate::routes::api_error::ApiError;
use crate::session::SessionState;

/// Docs pointer for markdown dialog render errors (RBP error-context contract).
const MARKDOWN_DOCS: &str =
    "docs/plans/phase-C/c12-host-markdown.md (GET /api/dialog content_html)";

/// Docs pointer for question preview render errors (RBP error-context contract).
const QUESTION_DOCS: &str =
    "docs/plans/phase-C/c13-host-question.md (GET /api/dialog preview_html)";

/// Max wall time for `pulldown-cmark` + `ammonia` on a `spawn_blocking` worker.
///
/// Large enough for typical docs under [`MARKDOWN_CONTENT_MAX_BYTES`]; short enough
/// that pathological input cannot stall the dialog session indefinitely.
pub(crate) const MARKDOWN_RENDER_TIMEOUT: Duration = Duration::from_secs(5);

/// Serialize active command fields for the packaged UI.
pub async fn get_dialog(State(session): State<SessionState>) -> Result<Json<Value>, ApiError> {
    let command = session.command().await;
    Ok(Json(dialog_payload(&command).await?))
}

/// Attach optional `width` / `height` hints to a `/api/dialog` payload.
fn attach_window_hints(obj: &mut Value, command: &Command) {
    if let Some(w) = command.window_width() {
        obj["width"] = json!(w);
    }
    if let Some(h) = command.window_height() {
        obj["height"] = json!(h);
    }
}

/// Build the `/api/dialog` JSON object for `command`.
pub(crate) async fn dialog_payload(command: &Command) -> Result<Value, ApiError> {
    match command {
        Command::Message {
            title,
            message,
            status,
            buttons,
            custom_buttons,
            default_button,
            level,
            icon,
            image,
            markdown,
            ..
        } => {
            let mut obj = json!({
                "type": "message",
                "title": title.as_str(),
                "message": message,
                "buttons": buttons_wire(*buttons),
                "markdown": markdown,
                "button_list": button_list(*buttons, custom_buttons.as_deref()),
            });
            if let Some(status) = status {
                obj["status"] = json!(status.as_str());
            }
            if let Some(custom) = custom_buttons {
                obj["custom_buttons"] = json!(custom);
            }
            if let Some(idx) = default_button {
                obj["default_button"] = json!(idx);
            }
            if let Some(level) = level {
                obj["level"] = json!(level.as_str());
            }
            if let Some(icon) = icon {
                obj["icon"] = json!(icon.as_str());
            }
            if let Some(image) = image {
                obj["image"] = json!(image.as_str());
            }
            attach_window_hints(&mut obj, command);
            Ok(obj)
        }
        Command::Input {
            title,
            message,
            status,
            icon,
            markdown,
            multiline,
            placeholder,
            default,
            password,
            mode,
            filter,
            multiple,
            start_path,
            buttons,
            ..
        } => {
            let mut obj = json!({
                "type": "input",
                "title": title.as_str(),
                "message": message,
                "markdown": markdown,
                "multiline": multiline,
                "password": password,
                "mode": mode.as_str(),
                "multiple": multiple,
                "buttons": buttons_wire(*buttons),
                "button_list": button_list(*buttons, None),
            });
            if let Some(status) = status {
                obj["status"] = json!(status.as_str());
            }
            if let Some(icon) = icon {
                obj["icon"] = json!(icon.as_str());
            }
            if let Some(placeholder) = placeholder {
                obj["placeholder"] = json!(placeholder);
            }
            if let Some(default) = default {
                obj["default"] = json!(default);
            }
            if let Some(filter) = filter {
                obj["filter"] = json!(filter);
            }
            if let Some(start_path) = start_path {
                obj["start_path"] = json!(start_path);
            }
            attach_window_hints(&mut obj, command);
            Ok(obj)
        }
        Command::Markdown {
            title,
            content,
            status,
            buttons,
            ..
        } => {
            let raw = content.as_deref().unwrap_or("");
            if raw.len() > MARKDOWN_CONTENT_MAX_BYTES {
                return Err(markdown_too_large(raw.len()));
            }
            let content_html = render_content_html_async(raw.to_owned()).await?;
            let mut obj = json!({
                "type": "markdown",
                "title": title
                    .as_ref()
                    .map(|t| t.as_str())
                    .unwrap_or("Markdown"),
                "content": raw,
                "content_html": content_html,
                "buttons": buttons_wire(*buttons),
                "button_list": button_list(*buttons, None),
            });
            if let Some(status) = status {
                obj["status"] = json!(status.as_str());
            }
            attach_window_hints(&mut obj, command);
            Ok(obj)
        }
        Command::Question { questions, .. } => {
            // Defense-in-depth: reject oversized option previews before
            // pulldown-cmark + ammonia (same cap as markdown content_html).
            for card in questions {
                for opt in &card.options {
                    if let Some(preview) = &opt.preview {
                        if preview.len() > MARKDOWN_CONTENT_MAX_BYTES {
                            return Err(preview_too_large(preview.len()));
                        }
                    }
                }
            }
            let questions_payload = render_question_payload_async(questions.clone()).await?;
            let mut obj = json!({
                "type": "question",
                "title": "Question",
                "questions": questions_payload,
            });
            attach_window_hints(&mut obj, command);
            Ok(obj)
        }
        Command::Chrome { title, status, .. } => {
            let mut obj = json!({
                "type": "chrome",
                "title": title.as_str(),
            });
            if let Some(status) = status {
                obj["status"] = json!(status.as_str());
            }
            attach_window_hints(&mut obj, command);
            Ok(obj)
        }
    }
}

/// Build question cards with server-side `preview_html` off the async executor.
async fn render_question_payload_async(questions: Vec<QuestionCard>) -> Result<Value, ApiError> {
    let join = tokio::task::spawn_blocking(move || question_payload_value(&questions));
    match tokio::time::timeout(markdown_render_timeout(), join).await {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(join_err)) => Err(question_internal(format!(
            "question preview render task failed: {join_err}"
        ))),
        Err(_elapsed) => Err(question_timeout(format!(
            "question preview render exceeded {}s",
            markdown_render_timeout().as_secs()
        ))),
    }
}

fn question_payload_value(questions: &[QuestionCard]) -> Value {
    let cards: Vec<Value> = questions
        .iter()
        .map(|card| {
            let options: Vec<Value> = card
                .options
                .iter()
                .map(|opt| {
                    let mut obj = json!({
                        "label": opt.label,
                        "description": opt.description,
                    });
                    if let Some(preview) = &opt.preview {
                        obj["preview"] = json!(preview);
                        obj["preview_html"] = json!(crate::question::render_preview_html(preview));
                    }
                    obj
                })
                .collect();
            json!({
                "question": card.question,
                "header": card.header,
                "options": options,
                "multiSelect": card.multi_select,
            })
        })
        .collect();
    Value::Array(cards)
}

fn preview_too_large(got_bytes: usize) -> ApiError {
    ApiError::bad_request(format!(
        "question option preview exceeds maximum of {MARKDOWN_CONTENT_MAX_BYTES} bytes (got {got_bytes} bytes)"
    ))
    .cause("active question dialog option preview is larger than the host render limit")
    .recovery("Reduce option preview size before opening the dialog")
    .recovery(format!(
        "Keep each preview at or below {MARKDOWN_CONTENT_MAX_BYTES} UTF-8 bytes"
    ))
    .docs(QUESTION_DOCS)
}

fn question_timeout(message: impl Into<String>) -> ApiError {
    ApiError::gateway_timeout(message)
        .cause("pulldown-cmark + ammonia did not finish within the question preview render timeout")
        .recovery("Simplify or shorten option preview fields and retry GET /api/dialog")
        .recovery("Report a bug if small previews consistently time out")
        .docs(QUESTION_DOCS)
}

fn question_internal(message: impl Into<String>) -> ApiError {
    ApiError::internal(message)
        .cause("spawn_blocking question preview render task joined with an error")
        .recovery("Retry GET /api/dialog")
        .recovery("Report a bug if the failure persists for valid preview markdown")
        .docs(QUESTION_DOCS)
}

/// Run CPU-bound markdown render off the async executor (picker `spawn_blocking` pattern).
async fn render_content_html_async(source: String) -> Result<String, ApiError> {
    let join = tokio::task::spawn_blocking(move || crate::markdown::render_content_html(&source));
    match tokio::time::timeout(markdown_render_timeout(), join).await {
        Ok(Ok(html)) => Ok(html),
        Ok(Err(join_err)) => Err(markdown_internal(format!(
            "markdown render task failed: {join_err}"
        ))),
        Err(_elapsed) => Err(markdown_timeout(format!(
            "markdown render exceeded {}s",
            markdown_render_timeout().as_secs()
        ))),
    }
}

fn markdown_render_timeout() -> Duration {
    #[cfg(test)]
    {
        if let Some(ms) = crate::markdown::test_render_timeout_ms() {
            return Duration::from_millis(ms);
        }
    }
    MARKDOWN_RENDER_TIMEOUT
}

fn markdown_too_large(got_bytes: usize) -> ApiError {
    ApiError::bad_request(format!(
        "markdown content exceeds maximum of {MARKDOWN_CONTENT_MAX_BYTES} bytes (got {got_bytes} bytes)"
    ))
    .cause("active markdown dialog content is larger than the host render limit")
    .recovery("Reduce markdown content size before opening the dialog")
    .recovery(format!(
        "Keep content at or below {MARKDOWN_CONTENT_MAX_BYTES} UTF-8 bytes"
    ))
    .docs(MARKDOWN_DOCS)
}

fn markdown_timeout(message: impl Into<String>) -> ApiError {
    ApiError::gateway_timeout(message)
        .cause("pulldown-cmark + ammonia did not finish within the markdown render timeout")
        .recovery("Simplify or shorten the markdown document and retry GET /api/dialog")
        .recovery("Report a bug if small documents consistently time out")
        .docs(MARKDOWN_DOCS)
}

fn markdown_internal(message: impl Into<String>) -> ApiError {
    ApiError::internal(message)
        .cause("spawn_blocking markdown render task joined with an error")
        .recovery("Retry GET /api/dialog")
        .recovery("Report a bug if the failure persists for valid markdown")
        .docs(MARKDOWN_DOCS)
}

fn buttons_wire(preset: ButtonsPreset) -> &'static str {
    match preset {
        ButtonsPreset::Ok => "ok",
        ButtonsPreset::OkCancel => "ok_cancel",
        ButtonsPreset::YesNo => "yes_no",
        ButtonsPreset::YesNoCancel => "yes_no_cancel",
        ButtonsPreset::RetryCancel => "retry_cancel",
        ButtonsPreset::Custom => "custom",
    }
}

fn button_list(preset: ButtonsPreset, custom: Option<&[String]>) -> Vec<Value> {
    let wire = preset.wire_labels(custom);
    let display = preset.display_labels(custom);
    wire.into_iter()
        .zip(display)
        .map(|(id, label)| json!({ "id": id, "label": label }))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;
    use wyvern_schema::{ButtonsPreset, ChromeTitle, QuestionCard, QuestionOption};

    fn md(content: impl Into<String>) -> Command {
        Command::Markdown {
            title: Some(ChromeTitle::new("Doc")),
            file: None,
            content: Some(content.into()),
            status: None,
            buttons: ButtonsPreset::Ok,
            width: None,
            height: None,
        }
    }

    fn question_with_preview(preview: impl Into<String>) -> Command {
        let preview = preview.into();
        Command::Question {
            questions: vec![QuestionCard {
                question: "Q?".into(),
                header: "Hdr".into(),
                options: vec![
                    QuestionOption {
                        label: "A".into(),
                        description: "a".into(),
                        preview: Some(preview.clone()),
                    },
                    QuestionOption {
                        label: "B".into(),
                        description: "b".into(),
                        preview: None,
                    },
                ],
                multi_select: false,
            }],
            questions_raw: vec![json!({
                "question": "Q?",
                "header": "Hdr",
                "options": [
                    { "label": "A", "description": "a", "preview": preview },
                    { "label": "B", "description": "b" }
                ],
                "multiSelect": false
            })],
            width: None,
            height: None,
        }
    }

    #[tokio::test]
    async fn dialog_payload_rejects_oversized_markdown() {
        let _guard = crate::markdown::lock_hooks().await;
        let oversized = "x".repeat(MARKDOWN_CONTENT_MAX_BYTES + 1);
        let err = dialog_payload(&md(oversized)).await.expect_err("oversized");
        let response = err.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn dialog_payload_times_out_slow_render() {
        let _guard = crate::markdown::lock_hooks().await;
        crate::markdown::set_render_timeout_ms(50);
        crate::markdown::set_render_delay_ms(500);
        let err = dialog_payload(&md("# slow\n")).await.expect_err("timeout");
        crate::markdown::set_render_delay_ms(0);
        crate::markdown::set_render_timeout_ms(0);
        let response = err.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::GATEWAY_TIMEOUT);
    }

    #[tokio::test]
    async fn dialog_payload_renders_normal_markdown() {
        let _guard = crate::markdown::lock_hooks().await;
        crate::markdown::set_render_delay_ms(0);
        crate::markdown::set_render_timeout_ms(0);
        let value = dialog_payload(&md("# Hello\n")).await.expect("render");
        assert_eq!(value["type"], "markdown");
        assert!(value["content_html"]
            .as_str()
            .is_some_and(|s| s.contains("<h1>Hello</h1>")));
    }

    #[tokio::test]
    async fn dialog_payload_rejects_oversized_question_preview() {
        let _guard = crate::markdown::lock_hooks().await;
        crate::markdown::set_render_delay_ms(0);
        crate::markdown::set_render_timeout_ms(0);
        let oversized = "x".repeat(MARKDOWN_CONTENT_MAX_BYTES + 1);
        let err = dialog_payload(&question_with_preview(oversized))
            .await
            .expect_err("oversized preview");
        let response = err.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn dialog_payload_renders_normal_question_preview() {
        let _guard = crate::markdown::lock_hooks().await;
        crate::markdown::set_render_delay_ms(0);
        crate::markdown::set_render_timeout_ms(0);
        let value = dialog_payload(&question_with_preview("**bold**"))
            .await
            .expect("render");
        assert_eq!(value["type"], "question");
        assert!(value["questions"][0]["options"][0]["preview_html"]
            .as_str()
            .is_some_and(|s| s.contains("<strong>bold</strong>")));
        assert!(value.get("buttons").is_none());
    }
}
