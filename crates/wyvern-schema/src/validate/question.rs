//! Validate `question` commands.

use serde_json::{Map, Value};

use crate::command::{Command, QuestionCard, QuestionOption};
use crate::error::ValidationError;
use crate::field_name::FieldName;

use super::helpers::{
    json_type_name, optional_window_size_fields, QUESTION_CARD_FIELDS, QUESTION_FIELDS,
    QUESTION_HEADER_MAX_CHARS, QUESTION_OPTION_FIELDS,
};

pub(super) fn validate_question(obj: &Map<String, Value>) -> Result<Command, ValidationError> {
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

    let (width, height) = optional_window_size_fields(obj)?;

    Ok(Command::Question {
        questions,
        questions_raw,
        width,
        height,
    })
}
