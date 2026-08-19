//! `next_wizard` resolution and wizard config deep-merge.

use std::path::PathBuf;

use serde_json::{Map, Value};

use super::{Allowlist, WorkflowError};
use crate::extensions::infer_wizard_root;

/// Next hop loaded from `next_wizard.path` (CLI resolves; host does not).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NextInvocation {
    /// Loaded wizard command JSON.
    pub command: Value,
    /// UI root for the next host session.
    pub ui_root: PathBuf,
    /// Directory of the next `wizard.json`.
    pub wizard_dir: PathBuf,
}

/// Resolve an optional `next_wizard` object on finish JSON.
///
/// # Errors
///
/// Returns [`WorkflowError::Resolve`] or [`WorkflowError::PathDenied`] when the
/// path or `ui_root` cannot be allowed.
pub fn resolve_next_wizard(
    finish: &Value,
    allowlist: &Allowlist,
) -> Result<Option<NextInvocation>, WorkflowError> {
    let Some(next) = finish.get("next_wizard") else {
        return Ok(None);
    };
    if next.is_null() {
        return Ok(None);
    }
    let obj = next.as_object().ok_or_else(|| WorkflowError::Resolve {
        path: String::new(),
        cause: "next_wizard must be an object".into(),
    })?;
    let path = obj
        .get("path")
        .and_then(Value::as_str)
        .ok_or_else(|| WorkflowError::Resolve {
            path: String::new(),
            cause: "next_wizard.path is required".into(),
        })?;
    let wizard_path = allowlist.resolve_allowed(path)?;
    let text = std::fs::read_to_string(&wizard_path).map_err(|err| WorkflowError::Resolve {
        path: path.to_string(),
        cause: format!("could not read wizard.json: {err}"),
    })?;
    let command: Value = serde_json::from_str(&text).map_err(|err| WorkflowError::Resolve {
        path: path.to_string(),
        cause: format!("wizard.json is not JSON: {err}"),
    })?;
    let wizard_dir = infer_wizard_root(&wizard_path);
    let ui_root = match obj.get("ui_root").and_then(Value::as_str) {
        Some(raw) => allowlist.resolve_allowed(raw)?,
        None => wizard_dir.clone(),
    };
    Ok(Some(NextInvocation {
        command,
        ui_root,
        wizard_dir,
    }))
}

/// Deep-merge `base ← input ← config_patch`.
///
/// Object keys deep-merge; arrays and scalars replace. Non-object `input` or
/// `config_patch` is [`WorkflowError::Merge`].
///
/// # Errors
///
/// Returns [`WorkflowError::Merge`] when `input` or `config_patch` is not an object.
pub fn merge_wizard_config(
    base: Value,
    input: Value,
    config_patch: Option<Value>,
) -> Result<Value, WorkflowError> {
    if !input.is_object() {
        return Err(WorkflowError::Merge {
            cause: "next_wizard.input must be a JSON object".into(),
        });
    }
    let mut out = deep_merge(base, input);
    if let Some(patch) = config_patch {
        if !patch.is_object() {
            return Err(WorkflowError::Merge {
                cause: "config_patch must be a JSON object".into(),
            });
        }
        out = deep_merge(out, patch);
    }
    Ok(out)
}

fn deep_merge(base: Value, overlay: Value) -> Value {
    match (base, overlay) {
        (Value::Object(mut base_map), Value::Object(overlay_map)) => {
            merge_objects(&mut base_map, overlay_map);
            Value::Object(base_map)
        }
        (_, overlay) => overlay,
    }
}

fn merge_objects(base: &mut Map<String, Value>, overlay: Map<String, Value>) {
    for (key, value) in overlay {
        match base.remove(&key) {
            Some(existing) => base.insert(key, deep_merge(existing, value)),
            None => base.insert(key, value),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn merge_deep_objects_replace_arrays() {
        let merged = merge_wizard_config(
            json!({"a": {"x": 1, "y": 2}, "keep": true, "list": [1]}),
            json!({"a": {"y": 9, "z": 3}, "list": [2, 3]}),
            Some(json!({"extra": false})),
        )
        .expect("merge");
        assert_eq!(
            merged,
            json!({"a": {"x": 1, "y": 9, "z": 3}, "keep": true, "list": [2, 3], "extra": false})
        );
    }

    #[test]
    fn merge_rejects_non_object_input() {
        let err = merge_wizard_config(json!({}), json!([]), None).expect_err("input");
        assert!(matches!(err, WorkflowError::Merge { .. }));
    }

    #[test]
    fn resolve_absent_next_wizard_is_none() {
        let tmp = tempfile::tempdir().expect("tmp");
        let allow = Allowlist {
            share_root: tmp.path().to_path_buf(),
            cwd: tmp.path().to_path_buf(),
            wizard_dir: tmp.path().to_path_buf(),
        };
        assert!(resolve_next_wizard(&json!({"button": "finish"}), &allow)
            .expect("ok")
            .is_none());
    }
}
