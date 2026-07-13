//! Validate JSON input against the Phase B executable surface
//! (`chrome`, `message`, `input`, `markdown`, `question`).

use std::path::Path;

use serde_json::{Map, Value};

use crate::chrome::{ChromeStatus, ChromeTitle};
use crate::command::{
    ButtonsPreset, Command, InputMode, MessageLevel, QuestionCard, QuestionOption,
};
use crate::error::ValidationError;
use crate::field_name::FieldName;
use crate::icons;

/// Known lifecycle actions that require `--interactive` (REQ-0060).
const LIFECYCLE_ACTIONS: &[&str] = &["show", "hide", "exit"];

/// Allowed fields on a `chrome` command object.
const CHROME_FIELDS: &[&str] = &["type", "title", "status"];

/// Allowed fields on a `message` command object (b.2 full surface).
const MESSAGE_FIELDS: &[&str] = &[
    "type",
    "title",
    "message",
    "status",
    "buttons",
    "custom_buttons",
    "default_button",
    "level",
    "icon",
    "image",
    "markdown",
];

/// Allowed fields on an `input` command object (b.4 full surface).
const INPUT_FIELDS: &[&str] = &[
    "type",
    "title",
    "message",
    "status",
    "icon",
    "markdown",
    "multiline",
    "placeholder",
    "default",
    "mode",
    "filter",
    "multiple",
    "start_path",
    "buttons",
];

/// Allowed fields on a `markdown` command object (b.5 file subset).
const MARKDOWN_FIELDS: &[&str] = &["type", "title", "file", "content", "status", "buttons"];

/// Allowed fields on a `question` command object (b.7).
const QUESTION_FIELDS: &[&str] = &["type", "questions"];

/// Allowed fields on each question card.
const QUESTION_CARD_FIELDS: &[&str] = &["question", "header", "options", "multiSelect"];

/// Allowed fields on each question option.
const QUESTION_OPTION_FIELDS: &[&str] = &["label", "description", "preview"];

/// Max characters for `questions[].header` (REQ-0062).
const QUESTION_HEADER_MAX_CHARS: usize = 12;

/// Phase B executable `type` values (through b.7).
const VALID_TYPES: &[&str] = &["chrome", "message", "input", "markdown", "question"];

/// Validate `value` as a Phase B command.
///
/// # Errors
///
/// Returns [`ValidationError::Validation`] for schema/field failures and
/// [`ValidationError::State`] when a lifecycle `action` is used outside
/// `--interactive`.
pub fn validate(value: &Value) -> Result<Command, ValidationError> {
    let obj = match value {
        Value::Object(map) => map,
        _ => {
            return Err(ValidationError::validation("type", "expected JSON object"));
        }
    };

    if let Some(err) = check_lifecycle_action(obj) {
        return Err(err);
    }

    let type_value = match obj.get("type") {
        None => {
            return Err(ValidationError::validation(
                "type",
                "missing required field 'type'",
            ));
        }
        Some(v) => v,
    };

    let type_str = match type_value {
        Value::String(s) => s.as_str(),
        other => {
            return Err(ValidationError::validation(
                "type",
                format!(
                    "field 'type' expected string, got {}",
                    json_type_name(other)
                ),
            ));
        }
    };

    match type_str {
        "chrome" => validate_chrome(obj),
        "message" => validate_message(obj),
        "input" => validate_input(obj),
        "markdown" => validate_markdown(obj),
        "question" => validate_question(obj),
        other => Err(unknown_type_error(other)),
    }
}

fn check_lifecycle_action(obj: &Map<String, Value>) -> Option<ValidationError> {
    let action = obj.get("action")?;
    let action_str = action.as_str()?;
    if LIFECYCLE_ACTIONS.contains(&action_str) {
        Some(ValidationError::state(
            "action",
            format!("{action_str} is only valid in --interactive mode"),
        ))
    } else {
        None
    }
}

fn validate_chrome(obj: &Map<String, Value>) -> Result<Command, ValidationError> {
    for key in obj.keys() {
        if !CHROME_FIELDS.contains(&key.as_str()) {
            return Err(ValidationError::validation(
                FieldName::new(key.as_str()),
                format!("unknown field '{key}'"),
            ));
        }
    }

    let title = require_string_field(obj, "title")?;
    let status = optional_string_field(obj, "status")?;

    Ok(Command::Chrome {
        title: ChromeTitle::new(title),
        status: status.map(ChromeStatus::new),
    })
}

fn validate_message(obj: &Map<String, Value>) -> Result<Command, ValidationError> {
    for key in obj.keys() {
        let key_str = key.as_str();
        if !MESSAGE_FIELDS.contains(&key_str) {
            return Err(ValidationError::validation(
                FieldName::new(key_str),
                format!("unknown field '{key_str}'"),
            ));
        }
    }

    let title = require_string_field(obj, "title")?;
    let message = require_string_field(obj, "message")?;
    let status = optional_string_field(obj, "status")?;

    let buttons = match obj.get("buttons") {
        None => {
            return Err(ValidationError::validation(
                "buttons",
                "missing required field 'buttons'",
            ));
        }
        Some(Value::String(s)) => match ButtonsPreset::parse(s) {
            Some(preset) => preset,
            None => {
                let options = ButtonsPreset::all_names().join(", ");
                let mut msg = format!("got '{s}', expected one of: {options}");
                if let Some(suggestion) = closest_match(s, ButtonsPreset::all_names()) {
                    msg.push_str(&format!("; did you mean '{suggestion}'?"));
                }
                return Err(ValidationError::validation("buttons", msg));
            }
        },
        Some(other) => {
            return Err(ValidationError::validation(
                "buttons",
                format!(
                    "field 'buttons' expected string, got {}",
                    json_type_name(other)
                ),
            ));
        }
    };

    let custom_buttons = match obj.get("custom_buttons") {
        None => None,
        Some(Value::Array(items)) => {
            let mut labels = Vec::with_capacity(items.len());
            for (i, item) in items.iter().enumerate() {
                match item {
                    Value::String(s) => labels.push(s.clone()),
                    other => {
                        return Err(ValidationError::validation(
                            "custom_buttons",
                            format!(
                                "custom_buttons[{i}] expected string, got {}",
                                json_type_name(other)
                            ),
                        ));
                    }
                }
            }
            Some(labels)
        }
        Some(other) => {
            return Err(ValidationError::validation(
                "custom_buttons",
                format!(
                    "field 'custom_buttons' expected array, got {}",
                    json_type_name(other)
                ),
            ));
        }
    };

    // REQ-0055: buttons: custom without custom_buttons → error
    if buttons == ButtonsPreset::Custom && custom_buttons.is_none() {
        return Err(ValidationError::validation(
            "custom_buttons",
            "buttons: custom requires a custom_buttons array",
        ));
    }

    // REQ-0056: custom_buttons with non-custom buttons → error
    if custom_buttons.is_some() && buttons != ButtonsPreset::Custom {
        return Err(ValidationError::validation(
            "custom_buttons",
            "custom_buttons is only valid when buttons is 'custom'",
        ));
    }

    let default_button = match obj.get("default_button") {
        None => None,
        Some(Value::Number(n)) => {
            let Some(idx) = n.as_u64() else {
                return Err(ValidationError::validation(
                    "default_button",
                    "field 'default_button' expected non-negative integer",
                ));
            };
            if idx > u64::from(u32::MAX) {
                return Err(ValidationError::validation(
                    "default_button",
                    "field 'default_button' exceeds u32 range",
                ));
            }
            Some(idx as u32)
        }
        Some(other) => {
            return Err(ValidationError::validation(
                "default_button",
                format!(
                    "field 'default_button' expected number, got {}",
                    json_type_name(other)
                ),
            ));
        }
    };

    if let Some(idx) = default_button {
        let count = buttons.button_count(custom_buttons.as_deref());
        if count == 0 || (idx as usize) >= count {
            return Err(ValidationError::validation(
                "default_button",
                format!("default_button index {idx} is out of range for {count} button(s)"),
            ));
        }
    }

    let level = match obj.get("level") {
        None => None,
        Some(Value::String(s)) => match MessageLevel::parse(s) {
            Some(level) => Some(level),
            None => {
                let options = MessageLevel::all_names().join(", ");
                let mut msg = format!("got '{s}', expected one of: {options}");
                if let Some(suggestion) = closest_match(s, MessageLevel::all_names()) {
                    msg.push_str(&format!("; did you mean '{suggestion}'?"));
                }
                return Err(ValidationError::validation("level", msg));
            }
        },
        Some(other) => {
            return Err(ValidationError::validation(
                "level",
                format!(
                    "field 'level' expected string, got {}",
                    json_type_name(other)
                ),
            ));
        }
    };

    let icon = optional_string_field(obj, "icon")?;
    let image = optional_string_field(obj, "image")?;
    if let Some(spec) = icon.as_deref() {
        if is_named_icon_spec(spec) {
            validate_named_icon("icon", spec)?;
        }
    }
    if let Some(spec) = image.as_deref() {
        if is_named_icon_spec(spec) {
            validate_named_icon("image", spec)?;
        }
    }
    let markdown = optional_bool_field(obj, "markdown")?.unwrap_or(false);

    Ok(Command::Message {
        title: ChromeTitle::new(title),
        message,
        status: status.map(ChromeStatus::new),
        buttons,
        custom_buttons,
        default_button,
        level,
        icon,
        image,
        markdown,
    })
}

fn validate_input(obj: &Map<String, Value>) -> Result<Command, ValidationError> {
    for key in obj.keys() {
        let key_str = key.as_str();
        if !INPUT_FIELDS.contains(&key_str) {
            return Err(ValidationError::validation(
                FieldName::new(key_str),
                format!("unknown field '{key_str}'"),
            ));
        }
    }

    let title = require_string_field(obj, "title")?;
    let message = require_string_field(obj, "message")?;
    let status = optional_string_field(obj, "status")?;
    let icon = optional_string_field(obj, "icon")?;
    if let Some(spec) = icon.as_deref() {
        if is_named_icon_spec(spec) {
            validate_named_icon("icon", spec)?;
        }
    }
    let markdown = optional_bool_field(obj, "markdown")?.unwrap_or(false);
    let multiline = optional_bool_field(obj, "multiline")?.unwrap_or(false);
    let placeholder = optional_string_field(obj, "placeholder")?;
    let default = optional_string_field(obj, "default")?;

    let mode = match obj.get("mode") {
        None => InputMode::Text,
        Some(Value::String(s)) => match InputMode::parse(s) {
            Some(mode) => mode,
            None => {
                let options = InputMode::all_names().join(", ");
                let mut msg = format!("got '{s}', expected one of: {options}");
                if let Some(suggestion) = closest_match(s, InputMode::all_names()) {
                    msg.push_str(&format!("; did you mean '{suggestion}'?"));
                }
                return Err(ValidationError::validation("mode", msg));
            }
        },
        Some(other) => {
            return Err(ValidationError::validation(
                "mode",
                format!(
                    "field 'mode' expected string, got {}",
                    json_type_name(other)
                ),
            ));
        }
    };

    // REQ-0057 — multiline is text-mode only.
    if multiline && matches!(mode, InputMode::File | InputMode::Folder) {
        return Err(ValidationError::validation(
            "multiline",
            "multiline is only valid when mode is 'text' (or omitted)",
        ));
    }

    // REQ-0059 — placeholder / default only for text mode.
    if placeholder.is_some() && matches!(mode, InputMode::File | InputMode::Folder) {
        return Err(ValidationError::validation(
            "placeholder",
            "placeholder is only valid when mode is 'text' (or omitted)",
        ));
    }
    if default.is_some() && matches!(mode, InputMode::File | InputMode::Folder) {
        return Err(ValidationError::validation(
            "default",
            "default is only valid when mode is 'text' (or omitted)",
        ));
    }

    // REQ-0059 — filter / multiple only for file mode.
    let filter = match obj.get("filter") {
        None => None,
        Some(_) if mode != InputMode::File => {
            return Err(ValidationError::validation(
                "filter",
                "filter is only valid when mode is 'file'",
            ));
        }
        Some(Value::Array(items)) => {
            let mut patterns = Vec::with_capacity(items.len());
            for (i, item) in items.iter().enumerate() {
                match item {
                    Value::String(s) => patterns.push(s.clone()),
                    other => {
                        return Err(ValidationError::validation(
                            "filter",
                            format!("filter[{i}] expected string, got {}", json_type_name(other)),
                        ));
                    }
                }
            }
            Some(patterns)
        }
        Some(other) => {
            return Err(ValidationError::validation(
                "filter",
                format!(
                    "field 'filter' expected array, got {}",
                    json_type_name(other)
                ),
            ));
        }
    };

    let multiple = match obj.get("multiple") {
        None => false,
        Some(_) if mode != InputMode::File => {
            return Err(ValidationError::validation(
                "multiple",
                "multiple is only valid when mode is 'file'",
            ));
        }
        Some(Value::Bool(b)) => *b,
        Some(other) => {
            return Err(ValidationError::validation(
                "multiple",
                format!(
                    "field 'multiple' expected boolean, got {}",
                    json_type_name(other)
                ),
            ));
        }
    };

    // REQ-0059 — start_path only for file or folder mode.
    let start_path = match obj.get("start_path") {
        None => None,
        Some(_) if matches!(mode, InputMode::Text) => {
            return Err(ValidationError::validation(
                "start_path",
                "start_path is only valid when mode is 'file' or 'folder'",
            ));
        }
        Some(Value::String(s)) => Some(s.clone()),
        Some(other) => {
            return Err(ValidationError::validation(
                "start_path",
                format!(
                    "field 'start_path' expected string, got {}",
                    json_type_name(other)
                ),
            ));
        }
    };

    let buttons = match obj.get("buttons") {
        None => ButtonsPreset::OkCancel,
        Some(Value::String(s)) => match ButtonsPreset::parse(s) {
            Some(preset) => preset,
            None => {
                let options = ButtonsPreset::all_names().join(", ");
                let mut msg = format!("got '{s}', expected one of: {options}");
                if let Some(suggestion) = closest_match(s, ButtonsPreset::all_names()) {
                    msg.push_str(&format!("; did you mean '{suggestion}'?"));
                }
                return Err(ValidationError::validation("buttons", msg));
            }
        },
        Some(other) => {
            return Err(ValidationError::validation(
                "buttons",
                format!(
                    "field 'buttons' expected string, got {}",
                    json_type_name(other)
                ),
            ));
        }
    };

    if buttons == ButtonsPreset::Custom {
        return Err(ValidationError::validation(
            "buttons",
            "buttons: custom is not supported for input in sprint b.4",
        ));
    }

    Ok(Command::Input {
        title: ChromeTitle::new(title),
        message,
        status: status.map(ChromeStatus::new),
        icon,
        markdown,
        multiline,
        placeholder,
        default,
        mode,
        filter,
        multiple,
        start_path,
        buttons,
    })
}

fn validate_markdown(obj: &Map<String, Value>) -> Result<Command, ValidationError> {
    for key in obj.keys() {
        let key_str = key.as_str();
        if !MARKDOWN_FIELDS.contains(&key_str) {
            return Err(ValidationError::validation(
                FieldName::new(key_str),
                format!("unknown field '{key_str}'"),
            ));
        }
    }

    let file = optional_string_field(obj, "file")?;
    let content = optional_string_field(obj, "content")?;

    // REQ-0058 — exactly one of file or content.
    match (file.as_ref(), content.as_ref()) {
        (None, None) | (Some(_), Some(_)) => {
            return Err(ValidationError::validation(
                "file",
                "markdown requires exactly one of 'file' or 'content'",
            ));
        }
        (Some(_), None) | (None, Some(_)) => {}
    }

    let title = match optional_string_field(obj, "title")? {
        Some(t) => Some(ChromeTitle::new(t)),
        None => Some(match file.as_ref() {
            Some(path) => {
                let name = Path::new(path)
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| path.clone());
                ChromeTitle::new(name)
            }
            None => ChromeTitle::new("Markdown"),
        }),
    };
    let status = optional_string_field(obj, "status")?;

    let buttons = match obj.get("buttons") {
        None => ButtonsPreset::Ok,
        Some(Value::String(s)) => match ButtonsPreset::parse(s) {
            Some(preset) => preset,
            None => {
                let options = ButtonsPreset::all_names().join(", ");
                let mut msg = format!("got '{s}', expected one of: {options}");
                if let Some(suggestion) = closest_match(s, ButtonsPreset::all_names()) {
                    msg.push_str(&format!("; did you mean '{suggestion}'?"));
                }
                return Err(ValidationError::validation("buttons", msg));
            }
        },
        Some(other) => {
            return Err(ValidationError::validation(
                "buttons",
                format!(
                    "field 'buttons' expected string, got {}",
                    json_type_name(other)
                ),
            ));
        }
    };

    if buttons == ButtonsPreset::Custom {
        return Err(ValidationError::validation(
            "buttons",
            "buttons: custom is not supported for markdown",
        ));
    }

    Ok(Command::Markdown {
        title,
        file,
        content,
        status: status.map(ChromeStatus::new),
        buttons,
    })
}

fn validate_question(obj: &Map<String, Value>) -> Result<Command, ValidationError> {
    for key in obj.keys() {
        let key_str = key.as_str();
        if !QUESTION_FIELDS.contains(&key_str) {
            return Err(ValidationError::validation(
                FieldName::new(key_str),
                format!("unknown field '{key_str}'"),
            ));
        }
    }

    let questions_value = match obj.get("questions") {
        None => {
            return Err(ValidationError::validation(
                "questions",
                "missing required field 'questions'",
            ));
        }
        Some(Value::Array(items)) => items,
        Some(other) => {
            return Err(ValidationError::validation(
                "questions",
                format!(
                    "field 'questions' expected array, got {}",
                    json_type_name(other)
                ),
            ));
        }
    };

    // REQ-0062 — 1–4 question cards.
    if questions_value.is_empty() {
        return Err(ValidationError::validation(
            "questions",
            "empty array not allowed; questions must contain 1–4 entries",
        ));
    }
    if questions_value.len() > 4 {
        return Err(ValidationError::validation(
            "questions",
            "max 4 questions per REQ-0062",
        ));
    }

    let mut questions = Vec::with_capacity(questions_value.len());
    let mut questions_raw = Vec::with_capacity(questions_value.len());

    for (qi, card_value) in questions_value.iter().enumerate() {
        let card_obj = match card_value {
            Value::Object(map) => map,
            other => {
                return Err(ValidationError::validation(
                    format!("questions[{qi}]"),
                    format!(
                        "questions[{qi}] expected object, got {}",
                        json_type_name(other)
                    ),
                ));
            }
        };

        for key in card_obj.keys() {
            let key_str = key.as_str();
            if !QUESTION_CARD_FIELDS.contains(&key_str) {
                return Err(ValidationError::validation(
                    format!("questions[{qi}].{key_str}"),
                    format!("unknown field '{key_str}'"),
                ));
            }
        }

        let question = match card_obj.get("question") {
            None => {
                return Err(ValidationError::validation(
                    format!("questions[{qi}].question"),
                    format!("missing required field 'questions[{qi}].question'"),
                ));
            }
            Some(Value::String(s)) if !s.is_empty() => s.clone(),
            Some(Value::String(_)) => {
                return Err(ValidationError::validation(
                    format!("questions[{qi}].question"),
                    format!("questions[{qi}].question must be a non-empty string"),
                ));
            }
            Some(other) => {
                return Err(ValidationError::validation(
                    format!("questions[{qi}].question"),
                    format!(
                        "questions[{qi}].question expected string, got {}",
                        json_type_name(other)
                    ),
                ));
            }
        };

        let header = match card_obj.get("header") {
            None => {
                return Err(ValidationError::validation(
                    format!("questions[{qi}].header"),
                    format!("missing required field 'questions[{qi}].header'"),
                ));
            }
            Some(Value::String(s)) => {
                if s.chars().count() > QUESTION_HEADER_MAX_CHARS {
                    return Err(ValidationError::validation(
                        format!("questions[{qi}].header"),
                        format!(
                            "questions[{qi}].header max {QUESTION_HEADER_MAX_CHARS} characters"
                        ),
                    ));
                }
                s.clone()
            }
            Some(other) => {
                return Err(ValidationError::validation(
                    format!("questions[{qi}].header"),
                    format!(
                        "questions[{qi}].header expected string, got {}",
                        json_type_name(other)
                    ),
                ));
            }
        };

        let multi_select = match card_obj.get("multiSelect") {
            None => {
                return Err(ValidationError::validation(
                    format!("questions[{qi}].multiSelect"),
                    format!("missing required field 'questions[{qi}].multiSelect'"),
                ));
            }
            Some(Value::Bool(b)) => *b,
            Some(other) => {
                return Err(ValidationError::validation(
                    format!("questions[{qi}].multiSelect"),
                    format!(
                        "questions[{qi}].multiSelect expected boolean, got {}",
                        json_type_name(other)
                    ),
                ));
            }
        };

        let options_value = match card_obj.get("options") {
            None => {
                return Err(ValidationError::validation(
                    format!("questions[{qi}].options"),
                    format!("missing required field 'questions[{qi}].options'"),
                ));
            }
            Some(Value::Array(items)) => items,
            Some(other) => {
                return Err(ValidationError::validation(
                    format!("questions[{qi}].options"),
                    format!(
                        "questions[{qi}].options expected array, got {}",
                        json_type_name(other)
                    ),
                ));
            }
        };

        // REQ-0062 — 2–4 options per card.
        if options_value.len() < 2 {
            return Err(ValidationError::validation(
                format!("questions[{qi}].options"),
                format!("questions[{qi}].options min 2 options"),
            ));
        }
        if options_value.len() > 4 {
            return Err(ValidationError::validation(
                format!("questions[{qi}].options"),
                format!("questions[{qi}].options max 4 options"),
            ));
        }

        let mut options = Vec::with_capacity(options_value.len());
        for (oi, opt_value) in options_value.iter().enumerate() {
            let opt_obj = match opt_value {
                Value::Object(map) => map,
                other => {
                    return Err(ValidationError::validation(
                        format!("questions[{qi}].options[{oi}]"),
                        format!(
                            "questions[{qi}].options[{oi}] expected object, got {}",
                            json_type_name(other)
                        ),
                    ));
                }
            };

            for key in opt_obj.keys() {
                let key_str = key.as_str();
                if !QUESTION_OPTION_FIELDS.contains(&key_str) {
                    return Err(ValidationError::validation(
                        format!("questions[{qi}].options[{oi}].{key_str}"),
                        format!("unknown field '{key_str}'"),
                    ));
                }
            }

            let label = match opt_obj.get("label") {
                None => {
                    return Err(ValidationError::validation(
                        format!("questions[{qi}].options[{oi}].label"),
                        format!("missing required field 'questions[{qi}].options[{oi}].label'"),
                    ));
                }
                Some(Value::String(s)) => s.clone(),
                Some(other) => {
                    return Err(ValidationError::validation(
                        format!("questions[{qi}].options[{oi}].label"),
                        format!(
                            "questions[{qi}].options[{oi}].label expected string, got {}",
                            json_type_name(other)
                        ),
                    ));
                }
            };

            let description = match opt_obj.get("description") {
                None => {
                    return Err(ValidationError::validation(
                        format!("questions[{qi}].options[{oi}].description"),
                        format!(
                            "missing required field 'questions[{qi}].options[{oi}].description'"
                        ),
                    ));
                }
                Some(Value::String(s)) => s.clone(),
                Some(other) => {
                    return Err(ValidationError::validation(
                        format!("questions[{qi}].options[{oi}].description"),
                        format!(
                            "questions[{qi}].options[{oi}].description expected string, got {}",
                            json_type_name(other)
                        ),
                    ));
                }
            };

            // preview accepted; rendered (sanitized) in b.8.
            let preview = match opt_obj.get("preview") {
                None => None,
                Some(Value::String(s)) => Some(s.clone()),
                Some(other) => {
                    return Err(ValidationError::validation(
                        format!("questions[{qi}].options[{oi}].preview"),
                        format!(
                            "questions[{qi}].options[{oi}].preview expected string, got {}",
                            json_type_name(other)
                        ),
                    ));
                }
            };

            options.push(QuestionOption {
                label,
                description,
                preview,
            });
        }

        questions.push(QuestionCard {
            question,
            header,
            options,
            multi_select,
        });
        questions_raw.push(card_value.clone());
    }

    Ok(Command::Question {
        questions,
        questions_raw,
    })
}

/// True when `spec` is a named icon (`role` / `role:N`), not a path or data URI.
fn is_named_icon_spec(spec: &str) -> bool {
    if spec.starts_with("data:") {
        return false;
    }
    if spec.contains('/') || spec.contains('\\') || spec.starts_with('.') {
        return false;
    }
    // Same filesystem-extension heuristic as b.2 `looks_like_path`.
    Path::new(spec).extension().is_none()
}

/// Validate a named icon / image spec against the schema catalog (REQ-0031).
fn validate_named_icon(field: &str, spec: &str) -> Result<(String, u32), ValidationError> {
    let (role, variant) = icons::parse_icon_spec(spec).map_err(|()| {
        ValidationError::validation(
            field,
            format!("invalid icon variant in '{spec}'; expected numeric suffix like ':2'"),
        )
    })?;
    if !icons::ROLES.contains(&role.as_str()) {
        return Err(ValidationError::validation(
            field,
            format!(
                "unknown icon '{role}'; valid names: {}",
                icons::ROLES.join(", ")
            ),
        ));
    }
    let max = icons::variant_count(&role);
    if variant < 1 || variant > max {
        return Err(ValidationError::validation(
            field,
            format!("variant {variant} out of range for '{role}' (valid: 1–{max})"),
        ));
    }
    Ok((role, variant))
}

fn require_string_field(obj: &Map<String, Value>, field: &str) -> Result<String, ValidationError> {
    match obj.get(field) {
        None => Err(ValidationError::validation(
            field,
            format!("missing required field '{field}'"),
        )),
        Some(Value::String(s)) => Ok(s.clone()),
        Some(other) => Err(ValidationError::validation(
            field,
            format!(
                "field '{field}' expected string, got {}",
                json_type_name(other)
            ),
        )),
    }
}

fn optional_string_field(
    obj: &Map<String, Value>,
    field: &str,
) -> Result<Option<String>, ValidationError> {
    match obj.get(field) {
        None => Ok(None),
        Some(Value::String(s)) => Ok(Some(s.clone())),
        Some(other) => Err(ValidationError::validation(
            field,
            format!(
                "field '{field}' expected string, got {}",
                json_type_name(other)
            ),
        )),
    }
}

fn optional_bool_field(
    obj: &Map<String, Value>,
    field: &str,
) -> Result<Option<bool>, ValidationError> {
    match obj.get(field) {
        None => Ok(None),
        Some(Value::Bool(b)) => Ok(Some(*b)),
        Some(other) => Err(ValidationError::validation(
            field,
            format!(
                "field '{field}' expected boolean, got {}",
                json_type_name(other)
            ),
        )),
    }
}

fn unknown_type_error(got: &str) -> ValidationError {
    let options = VALID_TYPES.join(", ");
    let mut message = format!("got '{got}', expected one of: {options}");
    if let Some(suggestion) = closest_match(got, VALID_TYPES) {
        message.push_str(&format!("; did you mean '{suggestion}'?"));
    }
    ValidationError::validation("type", message)
}

fn closest_match<'a>(got: &str, options: &[&'a str]) -> Option<&'a str> {
    options
        .iter()
        .copied()
        .filter(|opt| strsim::levenshtein(got, opt) <= 2)
        .min_by_key(|opt| strsim::levenshtein(got, opt))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ButtonsPreset, ChromeStatus, ChromeTitle, MessageLevel};
    use serde_json::json;

    #[test]
    fn chrome_with_title_passes() {
        let cmd = validate(&json!({"type":"chrome","title":"T"})).expect("valid");
        assert_eq!(
            cmd,
            Command::Chrome {
                title: ChromeTitle::new("T"),
                status: None,
            }
        );
    }

    #[test]
    fn chrome_with_title_and_status_passes() {
        let cmd = validate(&json!({"type":"chrome","title":"T","status":"Ready"})).expect("valid");
        assert_eq!(
            cmd,
            Command::Chrome {
                title: ChromeTitle::new("T"),
                status: Some(ChromeStatus::new("Ready")),
            }
        );
    }

    #[test]
    fn chrome_missing_title_fails() {
        let err = validate(&json!({"type":"chrome"})).expect_err("missing title");
        match err {
            ValidationError::Validation { field, message } => {
                assert_eq!(field, "title");
                assert!(message.contains("missing required field 'title'"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn empty_object_missing_type_fails() {
        let err = validate(&json!({})).expect_err("missing type");
        match err {
            ValidationError::Validation { field, message } => {
                assert_eq!(field, "type");
                assert!(message.contains("missing required field 'type'"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn title_without_type_fails() {
        let err = validate(&json!({"title":"T"})).expect_err("missing type");
        match err {
            ValidationError::Validation { field, .. } => assert_eq!(field, "type"),
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn type_null_fails_non_string() {
        let err = validate(&json!({"type":null,"title":"T"})).expect_err("null type");
        match err {
            ValidationError::Validation { field, message } => {
                assert_eq!(field, "type");
                assert!(message.contains("expected string"));
                assert!(message.contains("null"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn type_number_fails_wrong_type() {
        let err = validate(&json!({"type":1,"title":"T"})).expect_err("number type");
        match err {
            ValidationError::Validation { field, message } => {
                assert_eq!(field, "type");
                assert!(message.contains("expected string"));
                assert!(message.contains("number"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn chrome_unknown_field_fails() {
        let err = validate(&json!({"type":"chrome","title":"T","extra":true}))
            .expect_err("unknown field");
        match err {
            ValidationError::Validation { field, message } => {
                assert_eq!(field, "extra");
                assert!(message.contains("unknown field 'extra'"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn chrome_title_wrong_type_fails() {
        let err = validate(&json!({"type":"chrome","title":123})).expect_err("wrong title type");
        match err {
            ValidationError::Validation { field, message } => {
                assert_eq!(field, "title");
                assert!(message.contains("expected string"));
                assert!(message.contains("number"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn chrome_status_wrong_type_fails() {
        let err =
            validate(&json!({"type":"chrome","title":"T","status":false})).expect_err("status");
        match err {
            ValidationError::Validation { field, message } => {
                assert_eq!(field, "status");
                assert!(message.contains("expected string"));
                assert!(message.contains("boolean"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn type_message_ok_passes() {
        let cmd = validate(&json!({
            "type":"message",
            "title":"T",
            "message":"Hi",
            "buttons":"ok"
        }))
        .expect("valid message");
        assert!(matches!(
            cmd,
            Command::Message {
                buttons: ButtonsPreset::Ok,
                default_button: None,
                level: None,
                icon: None,
                image: None,
                markdown: false,
                ..
            }
        ));
    }

    #[test]
    fn type_message_level_and_markdown_pass() {
        let cmd = validate(&json!({
            "type":"message",
            "title":"T",
            "message":"**Hi**",
            "buttons":"ok",
            "level":"warning",
            "markdown": true,
            "icon":"error",
            "image":"data:image/png;base64,AA=="
        }))
        .expect("valid message extras");
        match cmd {
            Command::Message {
                level,
                markdown,
                icon,
                image,
                ..
            } => {
                assert_eq!(level, Some(MessageLevel::Warning));
                assert!(markdown);
                assert_eq!(icon.as_deref(), Some("error"));
                assert_eq!(image.as_deref(), Some("data:image/png;base64,AA=="));
            }
            other => panic!("expected Message, got {other:?}"),
        }
    }

    #[test]
    fn type_message_level_invalid_fails_req0054() {
        let err = validate(&json!({
            "type":"message",
            "title":"T",
            "message":"Hi",
            "buttons":"ok",
            "level":"warnin"
        }))
        .expect_err("bad level");
        match err {
            ValidationError::Validation { field, message } => {
                assert_eq!(field, "level");
                assert!(message.contains("expected one of"));
                assert!(message.contains("did you mean 'warning'"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn type_unknown_fails() {
        let err = validate(&json!({"type":"unknown"})).expect_err("unknown type");
        match err {
            ValidationError::Validation { field, message } => {
                assert_eq!(field, "type");
                assert!(message.contains("unknown"));
                assert!(message.contains("chrome"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn type_near_miss_suggests_chrome() {
        let err = validate(&json!({"type":"chrom","title":"T"})).expect_err("typo");
        match err {
            ValidationError::Validation { message, .. } => {
                assert!(message.contains("did you mean 'chrome'"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn action_show_is_state_error() {
        let err = validate(&json!({"action":"show"})).expect_err("state");
        match err {
            ValidationError::State { field, message } => {
                assert_eq!(field, "action");
                assert!(message.contains("show"));
                assert!(message.contains("--interactive"));
            }
            other => panic!("expected State, got {other:?}"),
        }
    }

    #[test]
    fn action_hide_is_state_error() {
        let err = validate(&json!({"action":"hide"})).expect_err("state");
        assert!(matches!(err, ValidationError::State { .. }));
    }

    #[test]
    fn action_exit_is_state_error() {
        let err = validate(&json!({"action":"exit"})).expect_err("state");
        assert!(matches!(err, ValidationError::State { .. }));
    }

    #[test]
    fn non_object_fails() {
        let err = validate(&json!("chrome")).expect_err("not object");
        match err {
            ValidationError::Validation { field, message } => {
                assert_eq!(field, "type");
                assert!(message.contains("expected JSON object"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn default_button_out_of_range_fails() {
        let err = validate(&json!({
            "type":"message",
            "title":"T",
            "message":"Hi",
            "buttons":"ok",
            "default_button": 1
        }))
        .expect_err("oob");
        match err {
            ValidationError::Validation { field, message } => {
                assert_eq!(field, "default_button");
                assert!(message.contains("out of range"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn custom_without_custom_buttons_fails() {
        let err = validate(&json!({
            "type":"message",
            "title":"T",
            "message":"Hi",
            "buttons":"custom"
        }))
        .expect_err("REQ-0055");
        match err {
            ValidationError::Validation { field, .. } => assert_eq!(field, "custom_buttons"),
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn custom_buttons_with_non_custom_preset_fails() {
        let err = validate(&json!({
            "type":"message",
            "title":"T",
            "message":"Hi",
            "buttons":"ok",
            "custom_buttons":["A"]
        }))
        .expect_err("REQ-0056");
        match err {
            ValidationError::Validation { field, .. } => assert_eq!(field, "custom_buttons"),
            other => panic!("expected Validation, got {other:?}"),
        }
    }
}
