//! `GET /api/dialog` — JSON payload for the active command.

use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};
use wyvern_schema::{ButtonsPreset, Command};

use crate::error::DialogTypeName;
use crate::session::SessionState;

/// Serialize active command fields for the packaged UI.
pub async fn get_dialog(State(session): State<SessionState>) -> Json<Value> {
    let command = session.command().await;
    Json(dialog_payload(&command))
}

/// Build the `/api/dialog` JSON object for `command`.
pub(crate) fn dialog_payload(command: &Command) -> Value {
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
            obj
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
            obj
        }
        other => json!({
            "type": command_type_name(other),
            "error": "unsupported_type",
        }),
    }
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
