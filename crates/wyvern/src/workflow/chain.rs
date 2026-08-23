//! `next_wizard` resolution and wizard config deep-merge.

use std::path::PathBuf;

use serde_json::{Map, Value};

use super::{Allowlist, WorkflowError};
use crate::error::LoadError;
use crate::extensions::infer_wizard_root;
use crate::input::read_file_capped;
use wyvern_schema::NextWizard;

/// Next hop loaded from `next_wizard.path` (CLI resolves; host does not).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NextInvocation {
    /// Loaded wizard command JSON.
    pub command: Value,
    /// UI root for the next host session.
    pub ui_root: PathBuf,
    /// Directory of the next `wizard.json`.
    pub wizard_dir: PathBuf,
    /// `next_wizard.input` deep-merged into the next wizard `config`.
    pub input: Value,
}

/// Resolve an optional `next_wizard` object on finish JSON.
///
/// Deserializes the hop to [`NextWizard`] so path / input / `ui_root` are typed
/// once; the pipeline must not walk the finish JSON again for `input`.
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
    let next: NextWizard =
        serde_json::from_value(next.clone()).map_err(|err| WorkflowError::Resolve {
            path: String::new(),
            cause: format!("next_wizard is invalid: {err}"),
        })?;
    let path = next.path.as_str();
    let wizard_path = allowlist.resolve_allowed(path)?;
    let text = read_file_capped(&wizard_path).map_err(|err| match err {
        LoadError::Io { message, .. } => WorkflowError::Resolve {
            path: path.to_string(),
            cause: message,
        },
        LoadError::Parse { message } | LoadError::Usage { message, .. } => WorkflowError::Resolve {
            path: path.to_string(),
            cause: message,
        },
    })?;
    let command: Value = serde_json::from_str(&text).map_err(|err| WorkflowError::Resolve {
        path: path.to_string(),
        cause: format!("wizard.json is not JSON: {err}"),
    })?;
    let wizard_dir = infer_wizard_root(&wizard_path);
    let ui_root = match next.ui_root.as_ref() {
        Some(raw) => allowlist.resolve_allowed(raw.as_str())?,
        None => wizard_dir.clone(),
    };
    Ok(Some(NextInvocation {
        command,
        ui_root,
        wizard_dir,
        input: next.input,
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

    #[test]
    fn resolve_next_wizard_carries_typed_input() {
        let tmp = tempfile::tempdir().expect("tmp");
        let wizard = tmp.path().join("wizard.json");
        std::fs::write(
            &wizard,
            r#"{"type":"wizard","page":{"id":"a","title":"T","html":"a.html"}}"#,
        )
        .unwrap();
        let allow = Allowlist {
            share_root: tmp.path().to_path_buf(),
            cwd: tmp.path().to_path_buf(),
            wizard_dir: tmp.path().to_path_buf(),
        };
        let finish = json!({
            "button": "finish",
            "next_wizard": {
                "path": wizard.to_string_lossy(),
                "input": {"from": "a"}
            }
        });
        let next = resolve_next_wizard(&finish, &allow)
            .expect("ok")
            .expect("some");
        assert_eq!(next.input, json!({"from": "a"}));
        assert_eq!(next.command["type"], "wizard");
    }
}
