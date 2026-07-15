//! Validate `wizard` commands (Phase D / REQ-0017 / REQ-0026).

use serde_json::{Map, Value};

use crate::command::Command;
use crate::error::ValidationError;
use crate::field_name::FieldName;
use crate::wizard::{WizardCommand, WizardPageDescriptor, WizardPageLayout};

use super::helpers::{
    json_type_name, optional_window_size_fields, WIZARD_FIELDS, WIZARD_PAGE_FIELDS,
};

pub(super) fn validate_wizard(obj: &Map<String, Value>) -> Result<Command, ValidationError> {
    for key in obj.keys() {
        let key_str = key.as_str();
        if !WIZARD_FIELDS.contains(&key_str) {
            return Err(ValidationError::validation(
                FieldName::new(key_str),
                format!("unknown field '{key_str}'"),
            ));
        }
    }

    let page_value = match obj.get("page") {
        None => {
            return Err(ValidationError::validation(
                "page",
                "missing required field 'page'",
            ));
        }
        Some(Value::Object(map)) => map,
        Some(other) => {
            return Err(ValidationError::validation(
                "page",
                format!(
                    "field 'page' expected object, got {}",
                    json_type_name(other)
                ),
            ));
        }
    };

    for key in page_value.keys() {
        let key_str = key.as_str();
        if !WIZARD_PAGE_FIELDS.contains(&key_str) {
            return Err(ValidationError::validation(
                format!("page.{key_str}"),
                format!("unknown field '{key_str}'"),
            ));
        }
    }

    let id = require_non_empty_page_string(page_value, "id")?;
    let title = require_non_empty_page_string(page_value, "title")?;
    let html = require_non_empty_page_string(page_value, "html")?;
    let layout = optional_page_layout(page_value)?;

    let config = match obj.get("config") {
        None => Value::Object(Map::new()),
        Some(value) => value.clone(),
    };

    let (width, height) = optional_window_size_fields(obj)?;

    Ok(Command::Wizard(WizardCommand {
        page: WizardPageDescriptor {
            id,
            title,
            html,
            layout,
        },
        config,
        width,
        height,
    }))
}

fn require_non_empty_page_string(
    page: &Map<String, Value>,
    field: &str,
) -> Result<String, ValidationError> {
    let path = format!("page.{field}");
    match page.get(field) {
        None => Err(ValidationError::validation(
            path.clone(),
            format!("missing required field '{path}'"),
        )),
        Some(Value::String(s)) if !s.is_empty() => Ok(s.clone()),
        Some(Value::String(_)) => Err(ValidationError::validation(
            path.clone(),
            format!("{path} must be a non-empty string"),
        )),
        Some(other) => Err(ValidationError::validation(
            path.clone(),
            format!("{path} expected string, got {}", json_type_name(other)),
        )),
    }
}

fn optional_page_layout(
    page: &Map<String, Value>,
) -> Result<Option<WizardPageLayout>, ValidationError> {
    match page.get("layout") {
        None => Ok(None),
        Some(Value::String(s)) => match WizardPageLayout::parse(s) {
            Some(layout) => Ok(Some(layout)),
            None => {
                let options = WizardPageLayout::all_names().join(", ");
                Err(ValidationError::validation(
                    "page.layout",
                    format!("got '{s}', expected one of: {options}"),
                ))
            }
        },
        Some(other) => Err(ValidationError::validation(
            "page.layout",
            format!("page.layout expected string, got {}", json_type_name(other)),
        )),
    }
}
