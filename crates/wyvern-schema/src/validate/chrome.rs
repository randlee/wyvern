//! Validate `chrome` commands.

use serde_json::{Map, Value};

use crate::chrome::{ChromeStatus, ChromeTitle};
use crate::command::Command;
use crate::error::ValidationError;
use crate::field_name::FieldName;

use super::helpers::{optional_string_field, require_string_field, CHROME_FIELDS};

pub(super) fn validate_chrome(obj: &Map<String, Value>) -> Result<Command, ValidationError> {
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
