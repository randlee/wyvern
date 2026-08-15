//! Two-phase template substitution for preexec args and command/host expand.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde_json::Value;
use tempfile::TempDir;

use super::preexec::{create_tmpdir, first_rendered_html, run_preexec, tmpdir_path};
use super::{ExtensionDef, ExtensionError, ExtensionMatch};

/// Context collected from an [`ExtensionMatch`] plus optional preexec outputs.
#[derive(Debug, Clone)]
pub struct MatchContext<'a> {
    /// Matched file path (None for Prefix-only).
    pub path: Option<&'a str>,
    /// Tokens after an argv prefix.
    pub args_after_prefix: &'a [String],
    /// Captured preexec stdout when `preexec.stdout` is `"markdown"`.
    pub preexec_stdout: Option<String>,
    /// Lexicographically first `*.html` under `{tmpdir}/pages/`.
    pub rendered_basename: Option<String>,
    /// Secure temp dir path when `{tmpdir}` is referenced.
    pub tmpdir: Option<PathBuf>,
    /// Unified `{wyvern_share}` root.
    pub wyvern_share: PathBuf,
}

/// Host overrides produced by phase-2 expand (`ui_root` only in Phase F).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HostOverrides {
    /// Replaces CLI `--ui-root` when `Some`.
    pub ui_root: Option<PathBuf>,
}

/// Validated expansion ready for the existing host pipeline.
#[derive(Debug)]
pub struct ExpandedInvocation {
    /// Command JSON that has passed [`wyvern_schema::validate`].
    pub command: Value,
    /// Host option overrides from the extension.
    pub host_overrides: HostOverrides,
    /// Temp dir kept until host exit when `ui_root` is `{tmpdir}`.
    pub temp_guard: Option<TempDir>,
}

#[derive(Clone, Copy)]
enum Phase {
    Preexec,
    Command,
}

/// Build the initial match context (no preexec outputs yet).
#[must_use]
pub fn build_match_context<'a>(m: &'a ExtensionMatch<'a>, _ext: &ExtensionDef) -> MatchContext<'a> {
    MatchContext {
        path: m.path(),
        args_after_prefix: m.args_after_prefix(),
        preexec_stdout: None,
        rendered_basename: None,
        tmpdir: None,
        wyvern_share: super::resolve_wyvern_share(),
    }
}

/// Phase 1: expand `preexec.cmd` and `preexec.args` only.
///
/// # Errors
///
/// Returns [`ExtensionError`] for missing args, unexpected tokens, or bad templates.
pub fn expand_preexec_args(
    ext: &ExtensionDef,
    ctx: &MatchContext<'_>,
) -> Result<(String, Vec<String>), ExtensionError> {
    let Some(pre) = ext.preexec.as_ref() else {
        return Err(ExtensionError::Template {
            message: "expand_preexec_args called without preexec".into(),
        });
    };
    let env = ExpandEnv::from_context(ext, ctx, Phase::Preexec)?;
    let cmd = env.expand_string(&pre.cmd)?;
    let args = env.expand_argv(&pre.args)?;
    Ok((cmd, args))
}

/// Phase 2: expand `command` / `command_from_file` and `host`.
///
/// # Errors
///
/// Returns [`ExtensionError`] for template, I/O, or missing-path failures.
pub fn expand_command_host(
    ext: &ExtensionDef,
    ctx: &MatchContext<'_>,
) -> Result<(Value, HostOverrides), ExtensionError> {
    let spec = ext
        .expand
        .as_ref()
        .ok_or_else(|| ExtensionError::Template {
            message: format!("extension '{}' has no expand block", ext.id),
        })?;
    if spec.command.is_some() && spec.command_from_file.is_some() {
        return Err(ExtensionError::Template {
            message: format!(
                "extension '{}' sets both command and command_from_file",
                ext.id
            ),
        });
    }
    let env = ExpandEnv::from_context(ext, ctx, Phase::Command)?;
    let command = if let Some(template) = &spec.command {
        env.expand_value(template)?
    } else if let Some(path_tmpl) = &spec.command_from_file {
        let path = env.expand_string(path_tmpl)?;
        let text = std::fs::read_to_string(&path).map_err(|err| ExtensionError::Io {
            message: format!("command_from_file '{path}': {err}"),
        })?;
        let value: Value = serde_json::from_str(&text).map_err(|err| ExtensionError::Io {
            message: format!("command_from_file '{path}' is not JSON: {err}"),
        })?;
        env.expand_value(&value)?
    } else {
        return Err(ExtensionError::Template {
            message: format!(
                "extension '{}' expand has neither command nor command_from_file",
                ext.id
            ),
        });
    };
    let host_overrides = match spec.host.as_ref().and_then(|h| h.ui_root.as_ref()) {
        Some(tmpl) => HostOverrides {
            ui_root: Some(PathBuf::from(env.expand_string(tmpl)?)),
        },
        None => HostOverrides::default(),
    };
    Ok((command, host_overrides))
}

thread_local! {
    static LAST_CREATED_TMPDIR: std::cell::RefCell<Option<PathBuf>> = const { std::cell::RefCell::new(None) };
}

/// Path of the temp dir created by the last [`expand_and_validate`] on this thread.
#[doc(hidden)]
#[must_use]
pub fn last_created_tmpdir() -> Option<PathBuf> {
    LAST_CREATED_TMPDIR.with(|cell| cell.borrow().clone())
}

/// Create tmpdir if needed, run preexec, expand, and validate.
///
/// On preexec failure the temp dir is dropped immediately (no host launch).
/// On success `temp_guard` is held until the caller drops [`ExpandedInvocation`].
///
/// # Errors
///
/// Returns [`ExtensionError`] for preexec, template, I/O, or validation failure.
pub fn expand_and_validate(
    ext: &ExtensionDef,
    ctx: &MatchContext<'_>,
) -> Result<ExpandedInvocation, ExtensionError> {
    let mut ctx = ctx.clone();
    let temp_guard = if references_tmpdir(ext) {
        let dir = create_tmpdir()?;
        ctx.tmpdir = Some(tmpdir_path(&dir));
        LAST_CREATED_TMPDIR.with(|cell| {
            *cell.borrow_mut() = ctx.tmpdir.clone();
        });
        Some(dir)
    } else {
        None
    };

    if ext.preexec.is_some() {
        let (cmd, args) = match expand_preexec_args(ext, &ctx) {
            Ok(pair) => pair,
            Err(err) => {
                drop(temp_guard);
                return Err(err);
            }
        };
        let stdout_mode = ext.preexec.as_ref().and_then(|p| p.stdout.as_deref());
        match run_preexec(&cmd, &args, stdout_mode) {
            Ok(stdout) => ctx.preexec_stdout = stdout,
            Err(err) => {
                drop(temp_guard);
                return Err(err);
            }
        }
        if references_rendered_basename(ext) {
            let tmp = ctx
                .tmpdir
                .as_deref()
                .ok_or_else(|| ExtensionError::Template {
                    message: "{rendered_basename} requires {tmpdir}".into(),
                })?;
            match first_rendered_html(tmp) {
                Ok(name) => ctx.rendered_basename = Some(name),
                Err(err) => {
                    drop(temp_guard);
                    return Err(err);
                }
            }
        }
    }

    let (command, host_overrides) = match expand_command_host(ext, &ctx) {
        Ok(pair) => pair,
        Err(err) => {
            drop(temp_guard);
            return Err(err);
        }
    };
    if let Err(source) = wyvern_schema::validate(&command) {
        drop(temp_guard);
        return Err(ExtensionError::InvalidCommand { source });
    }
    Ok(ExpandedInvocation {
        command,
        host_overrides,
        temp_guard,
    })
}

/// Walk from the file's directory until `wizard.json` or `pages/` is found.
#[must_use]
pub fn infer_wizard_root(path: &Path) -> PathBuf {
    let start = path
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
    let mut current = start.clone();
    loop {
        if current.join("wizard.json").is_file() || current.join("pages").is_dir() {
            return current;
        }
        match current.parent() {
            Some(parent) if parent != current => current = parent.to_path_buf(),
            _ => return start,
        }
    }
}

/// `{path}` relative to `{wizard_root}` using `/` separators.
#[must_use]
pub fn relpath_from_ui_root(path: &Path, wizard_root: &Path) -> String {
    path.strip_prefix(wizard_root)
        .map(|rel| rel.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| path.to_string_lossy().replace('\\', "/"))
}

fn references_tmpdir(ext: &ExtensionDef) -> bool {
    templates_contain(ext, "{tmpdir}")
}

fn references_rendered_basename(ext: &ExtensionDef) -> bool {
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

struct ExpandEnv<'a> {
    path: Option<&'a str>,
    tmpdir: Option<&'a Path>,
    wyvern_share: &'a Path,
    preexec_stdout: Option<&'a str>,
    rendered_basename: Option<&'a str>,
    args: BTreeMap<String, Vec<String>>,
    phase: Phase,
}

impl<'a> ExpandEnv<'a> {
    fn from_context(
        ext: &ExtensionDef,
        ctx: &'a MatchContext<'a>,
        phase: Phase,
    ) -> Result<Self, ExtensionError> {
        let declared = declared_args(ext);
        let path_token = ctx.path;
        let args = parse_named_args(ctx.args_after_prefix, &declared, path_token)?;
        Ok(Self {
            path: ctx.path,
            tmpdir: ctx.tmpdir.as_deref(),
            wyvern_share: ctx.wyvern_share.as_path(),
            preexec_stdout: ctx.preexec_stdout.as_deref(),
            rendered_basename: ctx.rendered_basename.as_deref(),
            args,
            phase,
        })
    }

    fn expand_string(&self, template: &str) -> Result<String, ExtensionError> {
        let mut out = String::new();
        for part in parse_template(template)? {
            match part {
                TemplatePart::Lit(lit) => out.push_str(&lit),
                TemplatePart::Var(name) => out.push_str(&self.lookup_string(&name)?),
            }
        }
        Ok(out)
    }

    fn expand_argv(&self, templates: &[String]) -> Result<Vec<String>, ExtensionError> {
        let mut out = Vec::new();
        for tmpl in templates {
            if let Some(name) = tmpl
                .strip_prefix("{arg:")
                .and_then(|rest| rest.strip_suffix(":repeat}"))
            {
                if !name.contains('{') {
                    if let Some(values) = self.args.get(name) {
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

    fn expand_value(&self, value: &Value) -> Result<Value, ExtensionError> {
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
                let values = self.args.get(repeat_name).cloned().unwrap_or_default();
                let mut tokens = Vec::new();
                for value in values {
                    tokens.push(format!("--{repeat_name}"));
                    tokens.push(value);
                }
                return Ok(tokens.join(" "));
            }
            return self
                .args
                .get(arg_name)
                .and_then(|v| v.first())
                .cloned()
                .ok_or_else(|| ExtensionError::MissingArg {
                    name: arg_name.to_string(),
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
                    .ok_or_else(|| ExtensionError::Template {
                        message: format!("{{stem}} has no file stem for '{path}'"),
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
                .ok_or_else(|| ExtensionError::Template {
                    message: "{tmpdir} was not created for this expansion".into(),
                }),
            "wyvern_share" => Ok(self.wyvern_share.to_string_lossy().into_owned()),
            "preexec.stdout" => match self.phase {
                Phase::Preexec => Err(ExtensionError::Template {
                    message: "{preexec.stdout} is only available in phase 2".into(),
                }),
                Phase::Command => self.preexec_stdout.map(ToOwned::to_owned).ok_or_else(|| {
                    ExtensionError::Template {
                        message: "{preexec.stdout} is empty (no markdown capture)".into(),
                    }
                }),
            },
            "rendered_basename" => match self.phase {
                Phase::Preexec => Err(ExtensionError::Template {
                    message: "{rendered_basename} is only available in phase 2".into(),
                }),
                Phase::Command => self
                    .rendered_basename
                    .map(ToOwned::to_owned)
                    .ok_or_else(|| ExtensionError::Template {
                        message: "{rendered_basename} is not set".into(),
                    }),
            },
            other => Err(ExtensionError::Template {
                message: format!("unknown template variable {{{other}}}"),
            }),
        }
    }

    fn require_path(&self, var: &str) -> Result<&str, ExtensionError> {
        self.path.ok_or_else(|| ExtensionError::PathVarWithoutPath {
            var: var.to_string(),
        })
    }
}

fn file_name(path: &str, var: &str) -> Result<String, ExtensionError> {
    Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .map(ToOwned::to_owned)
        .ok_or_else(|| ExtensionError::Template {
            message: format!("{{{var}}} has no file name for '{path}'"),
        })
}

enum TemplatePart {
    Lit(String),
    Var(String),
}

fn parse_template(template: &str) -> Result<Vec<TemplatePart>, ExtensionError> {
    let mut out = Vec::new();
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        if start > 0 {
            out.push(TemplatePart::Lit(rest[..start].to_string()));
        }
        let after = &rest[start + 1..];
        let Some(end) = after.find('}') else {
            return Err(ExtensionError::Template {
                message: format!("unclosed '{{' in template '{template}'"),
            });
        };
        out.push(TemplatePart::Var(after[..end].to_string()));
        rest = &after[end + 1..];
    }
    if !rest.is_empty() {
        out.push(TemplatePart::Lit(rest.to_string()));
    }
    Ok(out)
}

fn collect_template_vars(template: &str, into: &mut BTreeSet<String>) {
    if let Ok(parts) = parse_template(template) {
        for part in parts {
            if let TemplatePart::Var(name) = part {
                into.insert(name);
            }
        }
    }
}

fn collect_value_vars(value: &Value, into: &mut BTreeSet<String>) {
    match value {
        Value::String(s) => collect_template_vars(s, into),
        Value::Array(items) => {
            for item in items {
                collect_value_vars(item, into);
            }
        }
        Value::Object(map) => {
            for v in map.values() {
                collect_value_vars(v, into);
            }
        }
        _ => {}
    }
}

fn declared_args(ext: &ExtensionDef) -> BTreeSet<String> {
    let mut vars = BTreeSet::new();
    if let Some(pre) = &ext.preexec {
        collect_template_vars(&pre.cmd, &mut vars);
        for arg in &pre.args {
            collect_template_vars(arg, &mut vars);
        }
    }
    if let Some(exp) = &ext.expand {
        if let Some(cmd) = &exp.command {
            collect_value_vars(cmd, &mut vars);
        }
        if let Some(path) = &exp.command_from_file {
            collect_template_vars(path, &mut vars);
        }
        if let Some(ui) = exp.host.as_ref().and_then(|h| h.ui_root.as_ref()) {
            collect_template_vars(ui, &mut vars);
        }
    }
    vars.into_iter()
        .filter_map(|v| {
            v.strip_prefix("arg:")
                .map(|name| name.strip_suffix(":repeat").unwrap_or(name).to_string())
        })
        .collect()
}

fn parse_named_args(
    tokens: &[String],
    declared: &BTreeSet<String>,
    path_token: Option<&str>,
) -> Result<BTreeMap<String, Vec<String>>, ExtensionError> {
    let mut args: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut i = 0;
    while i < tokens.len() {
        let token = &tokens[i];
        if let Some(name) = token.strip_prefix("--") {
            if name.is_empty() {
                return Err(ExtensionError::UnexpectedArg {
                    token: token.clone(),
                });
            }
            let (name, inline) = match name.split_once('=') {
                Some((n, v)) => (n.to_string(), Some(v.to_string())),
                None => (name.to_string(), None),
            };
            if !declared.contains(&name) {
                return Err(ExtensionError::UnexpectedArg {
                    token: token.clone(),
                });
            }
            let value = if let Some(v) = inline {
                v
            } else {
                i += 1;
                tokens
                    .get(i)
                    .cloned()
                    .ok_or_else(|| ExtensionError::MissingArg { name: name.clone() })?
            };
            args.entry(name).or_default().push(value);
            i += 1;
            continue;
        }
        if path_token == Some(token.as_str()) {
            i += 1;
            continue;
        }
        return Err(ExtensionError::UnexpectedArg {
            token: token.clone(),
        });
    }
    Ok(args)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extensions::{ExtensionRegistry, SHIPPED_EXTENSIONS_JSON};

    #[test]
    fn md_suffix_expands_path_parts() {
        let registry = ExtensionRegistry::from_json_str(SHIPPED_EXTENSIONS_JSON).expect("shipped");
        let argv = vec!["docs/readme.md".to_string()];
        let matched = registry.match_argv(&argv).expect("match");
        let ctx = build_match_context(&matched, matched.extension());
        let (cmd, host) = expand_command_host(matched.extension(), &ctx).expect("expand");
        assert_eq!(cmd["type"], "markdown");
        assert_eq!(cmd["file"], "docs/readme.md");
        assert!(host.ui_root.is_none());
        wyvern_schema::validate(&cmd).expect("validate");
    }

    #[test]
    fn wizard_root_walk_unit() {
        let tmp = tempfile::tempdir().expect("tmp");
        let root = tmp.path().join("single-page");
        let pages = root.join("pages");
        std::fs::create_dir_all(&pages).expect("mkdir");
        std::fs::write(root.join("wizard.json"), "{}").expect("wizard.json");
        let html = pages.join("only.html");
        std::fs::write(&html, "<p>x</p>").expect("html");
        let inferred = infer_wizard_root(&html);
        assert_eq!(inferred, root);
        assert_eq!(relpath_from_ui_root(&html, &inferred), "pages/only.html");
    }

    #[test]
    fn arg_repeat_splices_tokens() {
        let json = r#"{
          "version": 1,
          "extensions": [
            {
              "id": "compose-render",
              "match": { "argv_prefix": ["compose", "render"] },
              "preexec": {
                "cmd": "true",
                "args": ["--root", "{arg:root}", "{arg:var-file:repeat}"]
              },
              "expand": {
                "command": { "type": "markdown", "content": "{arg:root}" }
              }
            }
          ]
        }"#;
        let registry = ExtensionRegistry::from_json_str(json).expect("parse");
        let argv = vec![
            "compose".into(),
            "render".into(),
            "--root".into(),
            "/tmp/r".into(),
            "--var-file".into(),
            "a.j2".into(),
            "--var-file".into(),
            "b.j2".into(),
        ];
        let matched = registry.match_argv(&argv).expect("match");
        let ctx = build_match_context(&matched, matched.extension());
        let (cmd, args) = expand_preexec_args(matched.extension(), &ctx).expect("preexec");
        assert_eq!(cmd, "true");
        assert_eq!(
            args,
            vec![
                "--root",
                "/tmp/r",
                "--var-file",
                "a.j2",
                "--var-file",
                "b.j2"
            ]
        );
    }

    #[test]
    fn missing_required_arg_errors() {
        let json = r#"{
          "version": 1,
          "extensions": [
            {
              "id": "needs-root",
              "match": { "argv_prefix": ["compose", "render"] },
              "expand": {
                "command": { "type": "markdown", "content": "{arg:root}" }
              }
            }
          ]
        }"#;
        let registry = ExtensionRegistry::from_json_str(json).expect("parse");
        let argv = vec!["compose".into(), "render".into()];
        let matched = registry.match_argv(&argv).expect("match");
        let ctx = build_match_context(&matched, matched.extension());
        let err = expand_command_host(matched.extension(), &ctx).expect_err("missing");
        assert!(matches!(err, ExtensionError::MissingArg { .. }));
    }
}
