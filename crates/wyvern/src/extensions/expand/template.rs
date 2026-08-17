//! Template parsing and path/file helpers for expand.

use std::io::Read;
use std::path::Path;

use serde_json::Value;

use crate::extensions::{ExtensionDef, ExtensionError, TemplateErrorKind};

/// 1 MiB cap for `command_from_file` JSON (RSH-003).
pub(super) const MAX_COMMAND_FROM_FILE_BYTES: usize = 1024 * 1024;

pub(super) enum TemplatePart {
    Lit(String),
    Var(String),
}

pub(super) fn parse_template(template: &str) -> Result<Vec<TemplatePart>, ExtensionError> {
    let mut out = Vec::new();
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        if start > 0 {
            out.push(TemplatePart::Lit(rest[..start].to_string()));
        }
        let after = &rest[start + 1..];
        let Some(end) = after.find('}') else {
            return Err(ExtensionError::template(
                TemplateErrorKind::UnclosedBrace,
                format!("unclosed '{{' in template '{template}'"),
            ));
        };
        out.push(TemplatePart::Var(after[..end].to_string()));
        rest = &after[end + 1..];
    }
    if !rest.is_empty() {
        out.push(TemplatePart::Lit(rest.to_string()));
    }
    Ok(out)
}

pub(super) fn references_tmpdir(ext: &ExtensionDef) -> bool {
    templates_contain(ext, "{tmpdir}")
}

pub(super) fn references_rendered_basename(ext: &ExtensionDef) -> bool {
    templates_contain(ext, "{rendered_basename}")
}

fn templates_contain(ext: &ExtensionDef, needle: &str) -> bool {
    if let Some(pre) = &ext.preexec {
        if pre.cmd.contains(needle) || pre.args.iter().any(|a| a.contains(needle)) {
            return true;
        }
    }
    if let Some(exp) = &ext.expand {
        if exp
            .command_from_file
            .as_deref()
            .is_some_and(|s| s.contains(needle))
        {
            return true;
        }
        if exp
            .host
            .as_ref()
            .and_then(|h| h.ui_root.as_deref())
            .is_some_and(|s| s.contains(needle))
        {
            return true;
        }
        if let Some(cmd) = &exp.command {
            if value_contains(cmd, needle) {
                return true;
            }
        }
    }
    false
}

fn value_contains(value: &Value, needle: &str) -> bool {
    match value {
        Value::String(s) => s.contains(needle),
        Value::Array(items) => items.iter().any(|v| value_contains(v, needle)),
        Value::Object(map) => map.values().any(|v| value_contains(v, needle)),
        _ => false,
    }
}

pub(super) fn read_command_from_file(path: &str) -> Result<String, ExtensionError> {
    let file = std::fs::File::open(path).map_err(|err| ExtensionError::Io {
        message: format!("command_from_file '{path}': {err}"),
        source: Some(Box::new(err)),
    })?;
    let mut buf = Vec::new();
    let n = file
        .take(MAX_COMMAND_FROM_FILE_BYTES as u64 + 1)
        .read_to_end(&mut buf)
        .map_err(|err| ExtensionError::Io {
            message: format!("command_from_file '{path}': {err}"),
            source: Some(Box::new(err)),
        })?;
    if n > MAX_COMMAND_FROM_FILE_BYTES {
        return Err(ExtensionError::Io {
            message: format!(
                "command_from_file '{path}' exceeds maximum of {MAX_COMMAND_FROM_FILE_BYTES} bytes"
            ),
            source: None,
        });
    }
    String::from_utf8(buf).map_err(|err| ExtensionError::Io {
        message: format!("command_from_file '{path}' is not valid UTF-8: {err}"),
        source: Some(Box::new(err)),
    })
}

pub(super) fn file_name(path: &str, var: &str) -> Result<String, ExtensionError> {
    Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            ExtensionError::template(
                TemplateErrorKind::Unavailable,
                format!("{{{var}}} has no file name for '{path}'"),
            )
        })
}
