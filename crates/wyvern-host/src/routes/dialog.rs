//! `GET /api/dialog` — JSON payload for the active command.

use std::time::Duration;

use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};
use wyvern_schema::{ButtonsPreset, Command, MARKDOWN_CONTENT_MAX_BYTES};

use crate::error::DialogTypeName;
use crate::routes::api_error::ApiError;
use crate::session::SessionState;

/// Docs pointer for markdown dialog render errors (RBP error-context contract).
const MARKDOWN_DOCS: &str =
    "docs/plans/phase-C/c12-host-markdown.md (GET /api/dialog content_html)";

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
            Ok(obj)
        }
        other => Ok(json!({
            "type": command_type_name(other),
            "error": "unsupported_type",
        })),
    }
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

fn command_type_name(command: &Command) -> &'static str {
    match command {
        Command::Chrome { .. } => DialogTypeName::Chrome.as_str(),
        Command::Message { .. } => DialogTypeName::Message.as_str(),
        Command::Input { .. } => DialogTypeName::Input.as_str(),
        Command::Markdown { .. } => DialogTypeName::Markdown.as_str(),
        Command::Question { .. } => DialogTypeName::Question.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;
    use wyvern_schema::{ButtonsPreset, ChromeTitle};

    fn md(content: impl Into<String>) -> Command {
        Command::Markdown {
            title: Some(ChromeTitle::new("Doc")),
            file: None,
            content: Some(content.into()),
            status: None,
            buttons: ButtonsPreset::Ok,
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
}
