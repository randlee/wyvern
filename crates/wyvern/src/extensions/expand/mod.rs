//! Two-phase template substitution for preexec args and command/host expand.

mod env;
mod preexec_orchestration;
mod template;

use std::path::{Path, PathBuf};

use serde_json::Value;
use tempfile::TempDir;

use super::{ExtensionDef, ExtensionError, ExtensionMatch, PreexecSpec, TemplateErrorKind};
use env::ExpandEnv;
use template::read_command_from_file;

#[doc(inline)]
pub use preexec_orchestration::{expand_and_validate, last_created_tmpdir};

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
pub(super) enum Phase {
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
/// Takes [`PreexecSpec`] so callers cannot invoke this without a preexec block.
/// `ext` is still required so `{arg:*}` declarations on command/host templates
/// are accepted during remainder parsing.
///
/// # Errors
///
/// Returns [`ExtensionError`] for missing args, unexpected tokens, or bad templates.
pub fn expand_preexec_args(
    pre: &PreexecSpec,
    ext: &ExtensionDef,
    ctx: &MatchContext<'_>,
) -> Result<(String, Vec<String>), ExtensionError> {
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
    let spec = ext.expand.as_ref().ok_or_else(|| {
        ExtensionError::template(
            TemplateErrorKind::InvalidSpec,
            format!("extension '{}' has no expand block", ext.id),
        )
    })?;
    if spec.command.is_some() && spec.command_from_file.is_some() {
        return Err(ExtensionError::template(
            TemplateErrorKind::InvalidSpec,
            format!(
                "extension '{}' sets both command and command_from_file",
                ext.id
            ),
        ));
    }
    let env = ExpandEnv::from_context(ext, ctx, Phase::Command)?;
    let command = if let Some(template) = &spec.command {
        env.expand_value(template)?
    } else if let Some(path_tmpl) = &spec.command_from_file {
        let path = env.expand_string(path_tmpl)?;
        // Expand the path only — file contents are Command JSON as written
        // (literal braces must survive, e.g. wizard.json).
        let text = read_command_from_file(&path)?;
        serde_json::from_str(&text).map_err(|err| ExtensionError::Io {
            message: format!("command_from_file '{path}' is not JSON: {err}"),
            source: Some(Box::new(err)),
        })?
    } else {
        return Err(ExtensionError::template(
            TemplateErrorKind::InvalidSpec,
            format!(
                "extension '{}' expand has neither command nor command_from_file",
                ext.id
            ),
        ));
    };
    let host_overrides = match spec.host.as_ref().and_then(|h| h.ui_root.as_ref()) {
        Some(tmpl) => HostOverrides {
            ui_root: Some(PathBuf::from(env.expand_string(tmpl)?)),
        },
        None => HostOverrides::default(),
    };
    Ok((command, host_overrides))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extensions::{ExtensionRegistry, SHIPPED_EXTENSIONS_JSON};
    use template::MAX_COMMAND_FROM_FILE_BYTES;

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
            "test-root".into(),
            "--var-file".into(),
            "a.j2".into(),
            "--var-file".into(),
            "b.j2".into(),
        ];
        let matched = registry.match_argv(&argv).expect("match");
        let ctx = build_match_context(&matched, matched.extension());
        let pre = matched.extension().preexec.as_ref().expect("preexec");
        let (cmd, args) = expand_preexec_args(pre, matched.extension(), &ctx).expect("preexec");
        assert_eq!(cmd, "true");
        assert_eq!(
            args,
            vec![
                "--root",
                "test-root",
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
        assert!(
            matches!(err, crate::extensions::ExtensionError::MissingArgs { ref missing, ref example, .. } if missing.iter().any(|m| m == "--root") && !example.is_empty()),
            "{err:?}"
        );
    }

    #[test]
    fn missing_flag_value_populates_example() {
        let json = r#"{
          "version": 1,
          "extensions": [
            {
              "id": "needs-root",
              "examples": ["wyvern compose render --root DIR"],
              "match": { "argv_prefix": ["compose", "render"] },
              "expand": {
                "command": { "type": "markdown", "content": "{arg:root}" }
              }
            }
          ]
        }"#;
        let registry = ExtensionRegistry::from_json_str(json).expect("parse");
        let argv = vec!["compose".into(), "render".into(), "--root".into()];
        let matched = registry.match_argv(&argv).expect("match");
        let ctx = build_match_context(&matched, matched.extension());
        let err = expand_command_host(matched.extension(), &ctx).expect_err("missing value");
        match err {
            crate::extensions::ExtensionError::MissingArgs {
                missing,
                example,
                extension_id,
                ..
            } => {
                assert!(missing.iter().any(|m| m == "--root"), "{missing:?}");
                assert!(
                    !example.is_empty() && example.contains("compose render"),
                    "{example}"
                );
                assert_eq!(extension_id.as_str(), "needs-root");
            }
            other => panic!("expected MissingArgs, got {other:?}"),
        }
    }

    #[test]
    fn path_parts_expand() {
        let json = r#"{
          "version": 1,
          "extensions": [{
            "id": "parts",
            "match": { "positional_suffix": ".md" },
            "expand": {
              "command": {
                "type": "markdown",
                "content": "{path}|{basename}|{stem}|{parent_dir}"
              }
            }
          }]
        }"#;
        let registry = ExtensionRegistry::from_json_str(json).expect("parse");
        let argv = vec!["docs/readme.md".to_string()];
        let matched = registry.match_argv(&argv).expect("match");
        let ctx = build_match_context(&matched, matched.extension());
        let (cmd, _) = expand_command_host(matched.extension(), &ctx).expect("expand");
        assert_eq!(cmd["content"], "docs/readme.md|readme.md|readme|docs");
    }

    #[test]
    fn arg_equals_form_parses() {
        let json = r#"{
          "version": 1,
          "extensions": [{
            "id": "compose-render",
            "match": { "argv_prefix": ["compose", "render"] },
            "expand": {
              "command": { "type": "markdown", "content": "{arg:root}" }
            }
          }]
        }"#;
        let registry = ExtensionRegistry::from_json_str(json).expect("parse");
        let argv = vec!["compose".into(), "render".into(), "--root=test-root".into()];
        let matched = registry.match_argv(&argv).expect("match");
        let ctx = build_match_context(&matched, matched.extension());
        let (cmd, _) = expand_command_host(matched.extension(), &ctx).expect("expand");
        assert_eq!(cmd["content"], "test-root");
    }

    #[test]
    fn arg_repeat_zero_occurrences_splices_empty() {
        let json = r#"{
          "version": 1,
          "extensions": [{
            "id": "compose-render",
            "match": { "argv_prefix": ["compose", "render"] },
            "preexec": {
              "cmd": "true",
              "args": ["--root", "{arg:root}", "{arg:var-file:repeat}"]
            },
            "expand": {
              "command": { "type": "markdown", "content": "{arg:root}" }
            }
          }]
        }"#;
        let registry = ExtensionRegistry::from_json_str(json).expect("parse");
        let argv = vec![
            "compose".into(),
            "render".into(),
            "--root".into(),
            "test-root".into(),
        ];
        let matched = registry.match_argv(&argv).expect("match");
        let ctx = build_match_context(&matched, matched.extension());
        let pre = matched.extension().preexec.as_ref().expect("preexec");
        let (_, args) = expand_preexec_args(pre, matched.extension(), &ctx).expect("preexec");
        assert_eq!(args, vec!["--root", "test-root"]);
    }

    #[test]
    fn path_var_on_prefix_only_is_error() {
        let json = r#"{
          "version": 1,
          "extensions": [{
            "id": "compose-render",
            "match": { "argv_prefix": ["compose", "render"] },
            "expand": {
              "command": { "type": "markdown", "file": "{path}" }
            }
          }]
        }"#;
        let registry = ExtensionRegistry::from_json_str(json).expect("parse");
        let argv = vec!["compose".into(), "render".into()];
        let matched = registry.match_argv(&argv).expect("match");
        let ctx = build_match_context(&matched, matched.extension());
        let err = expand_command_host(matched.extension(), &ctx).expect_err("path");
        assert!(matches!(
            err,
            crate::extensions::ExtensionError::PathVarWithoutPath { .. }
        ));
    }

    #[test]
    fn expand_and_validate_markdown_suffix() {
        let registry = ExtensionRegistry::from_json_str(SHIPPED_EXTENSIONS_JSON).expect("shipped");
        let argv = vec!["doc.md".to_string()];
        let matched = registry.match_argv(&argv).expect("match");
        let ctx = build_match_context(&matched, matched.extension());
        let expanded = expand_and_validate(matched.extension(), &ctx).expect("expand");
        assert_eq!(expanded.command["type"], "markdown");
        assert_eq!(expanded.command["file"], "doc.md");
        assert!(expanded.temp_guard.is_none());
    }

    #[test]
    fn tmpdir_guard_present_then_dropped() {
        let json = r#"{
          "version": 1,
          "extensions": [{
            "id": "tmp-host",
            "match": { "positional_suffix": ".md" },
            "expand": {
              "command": { "type": "markdown", "file": "{path}" },
              "host": { "ui_root": "{tmpdir}" }
            }
          }]
        }"#;
        let registry = ExtensionRegistry::from_json_str(json).expect("parse");
        let argv = vec!["doc.md".to_string()];
        let matched = registry.match_argv(&argv).expect("match");
        let ctx = build_match_context(&matched, matched.extension());
        let expanded = expand_and_validate(matched.extension(), &ctx).expect("expand");
        let tmp = expanded
            .temp_guard
            .as_ref()
            .expect("temp_guard")
            .path()
            .to_path_buf();
        assert!(tmp.is_dir());
        drop(expanded);
        assert!(!tmp.exists());
    }

    #[test]
    fn rendered_basename_substitutes_foo_html() {
        let json = r#"{
          "version": 1,
          "extensions": [{
            "id": "rendered",
            "match": { "positional_suffix": ".md" },
            "expand": {
              "command": { "type": "markdown", "content": "{rendered_basename}" }
            }
          }]
        }"#;
        let registry = ExtensionRegistry::from_json_str(json).expect("parse");
        let argv = vec!["doc.md".to_string()];
        let matched = registry.match_argv(&argv).expect("match");
        let mut ctx = build_match_context(&matched, matched.extension());
        ctx.rendered_basename = Some("foo.html".to_string());
        let (cmd, _) = expand_command_host(matched.extension(), &ctx).expect("expand");
        assert_eq!(cmd["content"], "foo.html");
        assert!(cmd["content"]
            .as_str()
            .is_some_and(|s| s.contains("foo.html")));
    }

    #[cfg(unix)]
    #[test]
    fn rendered_basename_end_to_end_from_tmpdir_pages() {
        let json = r#"{
          "version": 1,
          "extensions": [{
            "id": "rendered-e2e",
            "match": { "positional_suffix": ".md" },
            "preexec": {
              "cmd": "sh",
              "args": [
                "-c",
                "mkdir -p \"$1/pages\" && printf '<p>x</p>' > \"$1/pages/foo.html\"",
                "preexec",
                "{tmpdir}"
              ]
            },
            "expand": {
              "command": { "type": "markdown", "content": "{rendered_basename}" }
            }
          }]
        }"#;
        let registry = ExtensionRegistry::from_json_str(json).expect("parse");
        let argv = vec!["doc.md".to_string()];
        let matched = registry.match_argv(&argv).expect("match");
        let ctx = build_match_context(&matched, matched.extension());
        let expanded = expand_and_validate(matched.extension(), &ctx).expect("expand");
        assert_eq!(expanded.command["content"], "foo.html");
    }

    #[test]
    fn command_from_file_does_not_expand_literal_braces() {
        let tmp = tempfile::tempdir().expect("tmp");
        let path = tmp.path().join("cmd.json");
        std::fs::write(
            &path,
            r#"{"type":"markdown","content":"use {braces} literally"}"#,
        )
        .expect("write");
        let json = format!(
            r#"{{
          "version": 1,
          "extensions": [{{
            "id": "from-file",
            "match": {{ "positional_suffix": ".md" }},
            "expand": {{ "command_from_file": "{}" }}
          }}]
        }}"#,
            path.display().to_string().replace('\\', "/")
        );
        let registry = ExtensionRegistry::from_json_str(&json).expect("parse");
        let argv = vec!["doc.md".to_string()];
        let matched = registry.match_argv(&argv).expect("match");
        let ctx = build_match_context(&matched, matched.extension());
        let (cmd, _) = expand_command_host(matched.extension(), &ctx).expect("expand");
        assert_eq!(cmd["content"], "use {braces} literally");
    }

    #[test]
    fn command_from_file_rejects_oversize() {
        let tmp = tempfile::tempdir().expect("tmp");
        let path = tmp.path().join("huge.json");
        let body = format!(
            r#"{{"type":"markdown","content":"{}"}}"#,
            "x".repeat(MAX_COMMAND_FROM_FILE_BYTES)
        );
        std::fs::write(&path, &body).expect("write");
        let json = format!(
            r#"{{
          "version": 1,
          "extensions": [{{
            "id": "from-file",
            "match": {{ "positional_suffix": ".md" }},
            "expand": {{ "command_from_file": "{}" }}
          }}]
        }}"#,
            path.display().to_string().replace('\\', "/")
        );
        let registry = ExtensionRegistry::from_json_str(&json).expect("parse");
        let argv = vec!["doc.md".to_string()];
        let matched = registry.match_argv(&argv).expect("match");
        let ctx = build_match_context(&matched, matched.extension());
        let err = expand_command_host(matched.extension(), &ctx).expect_err("oversize");
        assert!(
            matches!(err, crate::extensions::ExtensionError::Io { ref message, .. } if message.contains("exceeds")),
            "{err}"
        );
    }
}
