//! `extends` resolution after registry merge.

use super::{ExtensionDef, ExtensionError, ExtensionId, PreexecSpec};

pub(super) fn apply_extends(exts: Vec<ExtensionDef>) -> Result<Vec<ExtensionDef>, ExtensionError> {
    let mut resolved = Vec::with_capacity(exts.len());
    for ext in &exts {
        resolved.push(resolve_one(ext, &exts, &mut Vec::new())?);
    }
    Ok(resolved)
}

fn resolve_one(
    ext: &ExtensionDef,
    all: &[ExtensionDef],
    stack: &mut Vec<ExtensionId>,
) -> Result<ExtensionDef, ExtensionError> {
    let Some(ref parent_id) = ext.extends else {
        return Ok(ext.clone());
    };
    if stack.iter().any(|id| id == &ext.id) {
        return Err(ExtensionError::InvalidRegistry {
            message: format!("circular extends involving '{}'", ext.id),
        });
    }
    stack.push(ext.id.clone());
    let parent = all
        .iter()
        .find(|candidate| candidate.id == *parent_id)
        .ok_or_else(|| ExtensionError::InvalidRegistry {
            message: format!("extension '{}' extends unknown id '{parent_id}'", ext.id),
        })?;
    let parent = resolve_one(parent, all, stack)?;
    stack.pop();
    Ok(inherit_from(ext, &parent))
}

fn inherit_from(child: &ExtensionDef, parent: &ExtensionDef) -> ExtensionDef {
    let preexec = match (&child.preexec, &parent.preexec) {
        (None, parent_pre) => parent_pre.clone(),
        (Some(child_pre), Some(parent_pre)) => {
            // Partial override: empty/absent child fields keep the parent value
            // so a requires-only child still inherits cmd/args/stdout.
            Some(PreexecSpec {
                cmd: if child_pre.cmd.is_empty() {
                    parent_pre.cmd.clone()
                } else {
                    child_pre.cmd.clone()
                },
                args: if child_pre.args.is_empty() {
                    parent_pre.args.clone()
                } else {
                    child_pre.args.clone()
                },
                requires: if child_pre.requires.is_empty() {
                    parent_pre.requires.clone()
                } else {
                    child_pre.requires.clone()
                },
                stdout: child_pre.stdout.or(parent_pre.stdout),
            })
        }
        (Some(child_pre), None) => Some(child_pre.clone()),
    };
    ExtensionDef {
        id: child.id.clone(),
        match_spec: child.match_spec.clone(),
        extends: child.extends.clone(),
        description: child
            .description
            .clone()
            .or_else(|| parent.description.clone()),
        examples: if child.examples.is_empty() {
            parent.examples.clone()
        } else {
            child.examples.clone()
        },
        preexec,
        expand: child.expand.clone().or_else(|| parent.expand.clone()),
    }
}
