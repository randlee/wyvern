//! Validate `report` commands (Phase H / REQ-0140).

use serde_json::{Map, Value};

use crate::command::Command;
use crate::error::ValidationError;
use crate::field_name::FieldName;
use crate::report::{
    ManifestPanelPath, PanelRole, ReportCommand, ReportMode, ReportPagePath, ReportPanelEntry,
    ReportTitle,
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
    let page = ReportPagePath::new(require_page_path(obj)?);
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

fn require_page_path(obj: &Map<String, Value>) -> Result<String, ValidationError> {
    let page = require_non_empty_string(obj, "page")?;
    if !page_has_allowed_suffix(&page) {
        return Err(ValidationError::validation(
            "page",
            format!("field 'page' must end with .html or .xhtml (got '{page}')"),
        ));
    }
    Ok(page)
}

fn page_has_allowed_suffix(page: &str) -> bool {
    let lower = page.to_ascii_lowercase();
    lower.ends_with(".html") || lower.ends_with(".xhtml")
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
    let label = optional_panel_string(index, obj, "label")?;
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
        Some(Value::String(s)) => {
            if !s.to_ascii_lowercase().ends_with(".xhtml") {
                return Err(ValidationError::validation(
                    field,
                    format!("panels[{index}].path must end with .xhtml (got '{s}')"),
                ));
            }
            Ok(ManifestPanelPath::new(s.clone()))
        }
        Some(other) => Err(ValidationError::validation(
            field.clone(),
            format!("{field} expected string, got {}", json_type_name(other)),
        )),
    }
}

fn optional_panel_string(
    index: usize,
    obj: &Map<String, Value>,
    name: &str,
) -> Result<Option<String>, ValidationError> {
    let field = format!("panels[{index}].{name}");
    match obj.get(name) {
        None => Ok(None),
        Some(Value::String(s)) => Ok(Some(s.clone())),
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
