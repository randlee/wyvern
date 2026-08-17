//! Validated expand environment and `{arg:*}` parsing.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde_json::Value;

use crate::extensions::{
    build_skill_record, catalog, ArgName, ExtensionDef, ExtensionError, PathRequiresProbe,
    TemplateErrorKind,
};

use super::template::{file_name, parse_template, TemplatePart};
use super::{infer_wizard_root, relpath_from_ui_root, MatchContext, Phase};

pub(super) struct ExpandEnv<'a> {
    ext: &'a ExtensionDef,
    declared: BTreeSet<ArgName>,
    path: Option<&'a str>,
    tmpdir: Option<&'a Path>,
    wyvern_share: &'a Path,
    preexec_stdout: Option<&'a str>,
    rendered_basename: Option<&'a str>,
    args: BTreeMap<ArgName, Vec<String>>,
    phase: Phase,
}

impl<'a> ExpandEnv<'a> {
    pub(super) fn from_context(
        ext: &'a ExtensionDef,
        ctx: &'a MatchContext<'a>,
        phase: Phase,
    ) -> Result<Self, ExtensionError> {
        let skill_args = catalog::declared_skill_args(ext);
        let declared: BTreeSet<ArgName> = skill_args.iter().map(|arg| arg.name.clone()).collect();
        let path_token = ctx.path;
        let args = parse_named_args(ctx.args_after_prefix, &declared, path_token, ext)?;
        let missing: Vec<String> = skill_args
            .iter()
            .filter(|arg| arg.required && !args.contains_key(&arg.name))
            .map(|arg| format!("--{}", arg.name))
            .collect();
        if !missing.is_empty() {
            return Err(missing_args_error(ext, missing, &declared));
        }
        Ok(Self {
            ext,
            declared,
            path: ctx.path,
            tmpdir: ctx.tmpdir.as_deref(),
            wyvern_share: ctx.wyvern_share.as_path(),
            preexec_stdout: ctx.preexec_stdout.as_deref(),
            rendered_basename: ctx.rendered_basename.as_deref(),
            args,
            phase,
        })
    }

    pub(super) fn expand_string(&self, template: &str) -> Result<String, ExtensionError> {
        let mut out = String::new();
        for part in parse_template(template)? {
            match part {
                TemplatePart::Lit(lit) => out.push_str(&lit),
                TemplatePart::Var(name) => out.push_str(&self.lookup_string(&name)?),
            }
        }
        Ok(out)
    }

    pub(super) fn expand_argv(&self, templates: &[String]) -> Result<Vec<String>, ExtensionError> {
        let mut out = Vec::new();
        for tmpl in templates {
            if let Some(name) = tmpl
                .strip_prefix("{arg:")
                .and_then(|rest| rest.strip_suffix(":repeat}"))
            {
                if !name.contains('{') {
                    if let Some(values) = ArgName::new(name).and_then(|n| self.args.get(&n)) {
                        for value in values {
                            out.push(format!("--{name}"));
                            out.push(value.clone());
                        }
                    }
                    continue;
                }
            }
            out.push(self.expand_string(tmpl)?);
        }
        Ok(out)
    }

    pub(super) fn expand_value(&self, value: &Value) -> Result<Value, ExtensionError> {
        match value {
            Value::String(s) => Ok(Value::String(self.expand_string(s)?)),
            Value::Array(items) => {
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    out.push(self.expand_value(item)?);
                }
                Ok(Value::Array(out))
            }
            Value::Object(map) => {
                let mut out = serde_json::Map::new();
                for (k, v) in map {
                    out.insert(k.clone(), self.expand_value(v)?);
                }
                Ok(Value::Object(out))
            }
            other => Ok(other.clone()),
        }
    }

    fn lookup_string(&self, name: &str) -> Result<String, ExtensionError> {
        if let Some(arg_name) = name.strip_prefix("arg:") {
            if let Some(repeat_name) = arg_name.strip_suffix(":repeat") {
                let values = ArgName::new(repeat_name)
                    .and_then(|n| self.args.get(&n).cloned())
                    .unwrap_or_default();
                let mut tokens = Vec::new();
                for value in values {
                    tokens.push(format!("--{repeat_name}"));
                    tokens.push(value);
                }
                return Ok(tokens.join(" "));
            }
            return ArgName::new(arg_name)
                .and_then(|n| self.args.get(&n).and_then(|v| v.first()).cloned())
                .ok_or_else(|| {
                    missing_args_error(self.ext, vec![format!("--{arg_name}")], &self.declared)
                });
        }
        match name {
            "path" => self.require_path("path").map(ToOwned::to_owned),
            "basename" => {
                let path = self.require_path("basename")?;
                file_name(path, "basename")
            }
            "stem" => {
                let path = self.require_path("stem")?;
                Path::new(path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(ToOwned::to_owned)
                    .ok_or_else(|| {
                        ExtensionError::template(
                            TemplateErrorKind::Unavailable,
                            format!("{{stem}} has no file stem for '{path}'"),
                        )
                    })
            }
            "parent_dir" => {
                let path = self.require_path("parent_dir")?;
                Ok(Path::new(path)
                    .parent()
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or_else(|| ".".into()))
            }
            "wizard_root" => {
                let path = self.require_path("wizard_root")?;
                Ok(infer_wizard_root(Path::new(path))
                    .to_string_lossy()
                    .into_owned())
            }
            "relpath_from_ui_root" => {
                let path = self.require_path("relpath_from_ui_root")?;
                let root = infer_wizard_root(Path::new(path));
                Ok(relpath_from_ui_root(Path::new(path), &root))
            }
            "tmpdir" => self
                .tmpdir
                .map(|p| p.to_string_lossy().into_owned())
                .ok_or_else(|| {
                    ExtensionError::template(
                        TemplateErrorKind::Unavailable,
                        "{tmpdir} was not created for this expansion",
                    )
                }),
            "wyvern_share" => Ok(self.wyvern_share.to_string_lossy().into_owned()),
            "preexec.stdout" => match self.phase {
                Phase::Preexec => Err(ExtensionError::template(
                    TemplateErrorKind::PhaseRestricted,
                    "{preexec.stdout} is only available in phase 2",
                )),
                Phase::Command => self.preexec_stdout.map(ToOwned::to_owned).ok_or_else(|| {
                    ExtensionError::template(
                        TemplateErrorKind::Unavailable,
                        "{preexec.stdout} is empty (no markdown capture)",
                    )
                }),
            },
            "rendered_basename" => match self.phase {
                Phase::Preexec => Err(ExtensionError::template(
                    TemplateErrorKind::PhaseRestricted,
                    "{rendered_basename} is only available in phase 2",
                )),
                Phase::Command => self
                    .rendered_basename
                    .map(ToOwned::to_owned)
                    .ok_or_else(|| {
                        ExtensionError::template(
                            TemplateErrorKind::Unavailable,
                            "{rendered_basename} is not set",
                        )
                    }),
            },
            other => Err(ExtensionError::template(
                TemplateErrorKind::UnknownVariable,
                format!("unknown template variable {{{other}}}"),
            )),
        }
    }

    fn require_path(&self, var: &str) -> Result<&str, ExtensionError> {
        self.path.ok_or_else(|| ExtensionError::PathVarWithoutPath {
            var: var.to_string(),
        })
    }
}

fn parse_named_args(
    tokens: &[String],
    declared: &BTreeSet<ArgName>,
    path_token: Option<&str>,
    ext: &ExtensionDef,
) -> Result<BTreeMap<ArgName, Vec<String>>, ExtensionError> {
    let mut args: BTreeMap<ArgName, Vec<String>> = BTreeMap::new();
    let mut i = 0;
    while i < tokens.len() {
        let token = &tokens[i];
        if let Some(name) = token.strip_prefix("--") {
            if name.is_empty() {
                return Err(unexpected_arg(token, declared, ext));
            }
            let (name, inline) = match name.split_once('=') {
                Some((n, v)) => (n, Some(v.to_string())),
                None => (name, None),
            };
            let Some(arg_name) = ArgName::new(name) else {
                return Err(unexpected_arg(token, declared, ext));
            };
            if !declared.contains(&arg_name) {
                return Err(unexpected_arg(token, declared, ext));
            }
            let value = if let Some(v) = inline {
                v
            } else {
                i += 1;
                tokens
                    .get(i)
                    .cloned()
                    .ok_or_else(|| missing_args_error(ext, vec![format!("--{name}")], declared))?
            };
            args.entry(arg_name).or_default().push(value);
            i += 1;
            continue;
        }
        if path_token == Some(token.as_str()) {
            i += 1;
            continue;
        }
        return Err(unexpected_arg(token, declared, ext));
    }
    Ok(args)
}

fn unexpected_arg(token: &str, declared: &BTreeSet<ArgName>, ext: &ExtensionDef) -> ExtensionError {
    ExtensionError::UnexpectedArg {
        token: token.to_string(),
        declared: declared_strings(declared),
        extension_id: ext.id.clone(),
    }
}

fn missing_args_error(
    ext: &ExtensionDef,
    missing: Vec<String>,
    declared: &BTreeSet<ArgName>,
) -> ExtensionError {
    let record = build_skill_record(ext, &PathRequiresProbe);
    let example = record
        .examples
        .first()
        .cloned()
        .unwrap_or(record.invocation);
    ExtensionError::MissingArgs {
        missing,
        declared: declared_strings(declared),
        extension_id: ext.id.clone(),
        example,
    }
}

fn declared_strings(declared: &BTreeSet<ArgName>) -> BTreeSet<String> {
    declared
        .iter()
        .map(|name| name.as_str().to_string())
        .collect()
}
