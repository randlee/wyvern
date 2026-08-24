//! Validate `report` commands (Phase H / REQ-0140).

use serde_json::{Map, Value};

use crate::command::Command;
use crate::error::ValidationError;
use crate::field_name::FieldName;
use crate::report::{
    ManifestPanelPath, PanelLabel, PanelRole, ReportCommand, ReportFieldError, ReportMode,
    ReportPagePath, ReportPanelEntry, ReportTitle, MAX_PANEL_LABEL_CHARS, MAX_REPORT_PANELS,
};

use super::helpers::{
    closest_match, json_type_name, optional_window_size_fields, REPORT_FIELDS, REPORT_PANEL_FIELDS,
};

pub(super) fn validate_report(obj: &Map<String, Value>) -> Result<Command, ValidationError> {
    for key in obj.keys() {
        let key_str = key.as_str();
        if !REPORT_FIELDS.contains(&key_str) {
            return Err(ValidationError::validation(
                FieldName::new(key_str),
                format!("unknown field '{key_str}'"),
            ));
        }
    }

    let title = ReportTitle::new(require_non_empty_string(obj, "title")?);
    let page = require_page_path(obj)?;
    let mode = optional_report_mode(obj)?.unwrap_or(ReportMode::View);
    let panels = optional_panels(obj)?;
    if mode == ReportMode::Review && panels.as_ref().is_none_or(Vec::is_empty) {
        return Err(ValidationError::validation(
            "panels",
            "field 'panels' is required when mode is 'review' and must be a non-empty array",
        ));
    }
    let (width, height) = optional_window_size_fields(obj)?;

    Ok(Command::Report(ReportCommand {
        title,
        page,
        mode,
        panels,
        width,
        height,
    }))
}

fn require_non_empty_string(
    obj: &Map<String, Value>,
    field: &str,
) -> Result<String, ValidationError> {
    match obj.get(field) {
        None => Err(ValidationError::validation(
            field,
            format!("missing required field '{field}'"),
        )),
        Some(Value::String(s)) if !s.is_empty() => Ok(s.clone()),
        Some(Value::String(_)) => Err(ValidationError::validation(
            field,
            format!("field '{field}' must be a non-empty string"),
        )),
        Some(other) => Err(ValidationError::validation(
            field,
            format!(
                "field '{field}' expected string, got {}",
                json_type_name(other)
            ),
        )),
    }
}

fn require_page_path(obj: &Map<String, Value>) -> Result<ReportPagePath, ValidationError> {
    let page = require_non_empty_string(obj, "page")?;
    ReportPagePath::try_new(page.clone()).map_err(|_| {
        ValidationError::validation(
            "page",
            format!("field 'page' must end with .html or .xhtml (got '{page}')"),
        )
    })
}

fn optional_report_mode(obj: &Map<String, Value>) -> Result<Option<ReportMode>, ValidationError> {
    match obj.get("mode") {
        None => Ok(None),
        Some(Value::String(s)) => match ReportMode::parse(s) {
            Some(mode) => Ok(Some(mode)),
            None => {
                let options = ReportMode::all_names().join(", ");
                let mut message = format!("got '{s}', expected one of: {options}");
                if let Some(suggestion) = closest_match(s, ReportMode::all_names()) {
                    message.push_str(&format!("; did you mean '{suggestion}'?"));
                }
                Err(ValidationError::validation("mode", message))
            }
        },
        Some(other) => Err(ValidationError::validation(
            "mode",
            format!(
                "field 'mode' expected string, got {}",
                json_type_name(other)
            ),
        )),
    }
}

fn optional_panels(
    obj: &Map<String, Value>,
) -> Result<Option<Vec<ReportPanelEntry>>, ValidationError> {
    match obj.get("panels") {
        None => Ok(None),
        Some(Value::Array(items)) => {
            if items.len() > MAX_REPORT_PANELS {
                return Err(ValidationError::validation(
                    "panels",
                    format!("field 'panels' must have at most {MAX_REPORT_PANELS} items"),
                ));
            }
            let mut panels = Vec::with_capacity(items.len());
            for (index, item) in items.iter().enumerate() {
                panels.push(validate_panel_entry(index, item)?);
            }
            Ok(Some(panels))
        }
        Some(other) => Err(ValidationError::validation(
            "panels",
            format!(
                "field 'panels' expected array, got {}",
                json_type_name(other)
            ),
        )),
    }
}

fn validate_panel_entry(index: usize, value: &Value) -> Result<ReportPanelEntry, ValidationError> {
    let obj = match value {
        Value::Object(map) => map,
        other => {
            return Err(ValidationError::validation(
                format!("panels[{index}]"),
                format!(
                    "panels[{index}] expected object, got {}",
                    json_type_name(other)
                ),
            ));
        }
    };
    for key in obj.keys() {
        let key_str = key.as_str();
        if !REPORT_PANEL_FIELDS.contains(&key_str) {
            return Err(ValidationError::validation(
                format!("panels[{index}].{key_str}"),
                format!("unknown field '{key_str}'"),
            ));
        }
    }
    let path = require_panel_path(index, obj)?;
    let label = optional_panel_label(index, obj)?;
    let role = optional_panel_role(index, obj)?;
    Ok(ReportPanelEntry { path, label, role })
}

fn require_panel_path(
    index: usize,
    obj: &Map<String, Value>,
) -> Result<ManifestPanelPath, ValidationError> {
    let field = format!("panels[{index}].path");
    match obj.get("path") {
        None => Err(ValidationError::validation(
            field.clone(),
            format!("missing required field '{field}'"),
        )),
        Some(Value::String(s)) if s.is_empty() => Err(ValidationError::validation(
            field,
            format!("panels[{index}].path must be a non-empty string"),
        )),
        Some(Value::String(s)) => ManifestPanelPath::try_new(s.clone()).map_err(|_| {
            ValidationError::validation(
                field,
                format!("panels[{index}].path must end with .xhtml (got '{s}')"),
            )
        }),
        Some(other) => Err(ValidationError::validation(
            field.clone(),
            format!("{field} expected string, got {}", json_type_name(other)),
        )),
    }
}

fn optional_panel_label(
    index: usize,
    obj: &Map<String, Value>,
) -> Result<Option<PanelLabel>, ValidationError> {
    let field = format!("panels[{index}].label");
    match obj.get("label") {
        None => Ok(None),
        Some(Value::String(s)) => PanelLabel::try_new(s.clone()).map(Some).map_err(|err| {
            let message = match err {
                ReportFieldError::Empty => {
                    format!("panels[{index}].label must be a non-empty string")
                }
                ReportFieldError::LabelTooLong => format!(
                    "panels[{index}].label must be at most {MAX_PANEL_LABEL_CHARS} characters"
                ),
                other => format!("panels[{index}].label is invalid: {other}"),
            };
            ValidationError::validation(field, message)
        }),
        Some(other) => Err(ValidationError::validation(
            field.clone(),
            format!("{field} expected string, got {}", json_type_name(other)),
        )),
    }
}

fn optional_panel_role(
    index: usize,
    obj: &Map<String, Value>,
) -> Result<Option<PanelRole>, ValidationError> {
    let field = format!("panels[{index}].role");
    match obj.get("role") {
        None => Ok(None),
        Some(Value::String(s)) => match PanelRole::parse(s) {
            Some(role) => Ok(Some(role)),
            None => {
                let options = PanelRole::all_names().join(", ");
                Err(ValidationError::validation(
                    field,
                    format!("got '{s}', expected one of: {options}"),
                ))
            }
        },
        Some(other) => Err(ValidationError::validation(
            field.clone(),
            format!("{field} expected string, got {}", json_type_name(other)),
        )),
    }
}
