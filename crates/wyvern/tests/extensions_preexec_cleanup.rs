//! Tmpdir lifecycle: present during host, deleted after exit, deleted on preexec failure.

use std::path::PathBuf;

use wyvern::extensions::{
    build_match_context, create_tmpdir, expand_and_validate, ExtensionError, ExtensionRegistry,
    MatchContext,
};

fn registry_with(json: &str) -> ExtensionRegistry {
    ExtensionRegistry::from_json_str(json).expect("registry")
}

fn tmpdir_host_extension(preexec: Option<(&str, &[&str])>) -> String {
    let mut ext = serde_json::json!({
        "id": "tmpdir-test",
        "match": { "positional_suffix": ".md" },
        "expand": {
            "command": { "type": "markdown", "file": "{path}" },
            "host": { "ui_root": "{tmpdir}" }
        }
    });
    if let Some((cmd, args)) = preexec {
        ext["preexec"] = serde_json::json!({
            "cmd": cmd,
            "args": args,
        });
    }
    serde_json::json!({
        "version": 1,
        "extensions": [ext]
    })
    .to_string()
}

fn failing_preexec() -> (&'static str, Vec<&'static str>) {
    #[cfg(windows)]
    {
        ("cmd", vec!["/C", "exit 1"])
    }
    #[cfg(not(windows))]
    {
        ("false", vec![])
    }
}

#[test]
fn tmpdir_present_during_invocation() {
    let registry = registry_with(&tmpdir_host_extension(None));
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
    assert!(tmp.is_dir(), "tmpdir present: {}", tmp.display());
    drop(expanded);
    assert!(
        !tmp.exists(),
        "tmpdir deleted after drop: {}",
        tmp.display()
    );
}

#[test]
fn tmpdir_deleted_on_preexec_failure() {
    let (cmd, args) = failing_preexec();
    let registry = registry_with(&tmpdir_host_extension(Some((cmd, &args))));
    let argv = vec!["doc.md".to_string()];
    let matched = registry.match_argv(&argv).expect("match");
    let ctx = build_match_context(&matched, matched.extension());
    let err = expand_and_validate(matched.extension(), &ctx).expect_err("preexec fail");
    assert!(matches!(err, ExtensionError::Preexec { .. }), "{err}");
    let tmp = wyvern::extensions::last_created_tmpdir().expect("tmpdir was created");
    assert!(
        !tmp.exists(),
        "tmpdir must be deleted immediately on preexec failure: {}",
        tmp.display()
    );
}

#[test]
fn tmpdir_path_persists_while_guard_held() {
    let guard = create_tmpdir().expect("create_tmpdir");
    let path = guard.path().to_path_buf();
    assert!(path.is_dir(), "held guard path: {}", path.display());
    drop(guard);
    assert!(!path.exists(), "path gone after drop: {}", path.display());
}

#[test]
fn extensions_preexec_cleanup_deleted_after_host_exit() {
    let registry = registry_with(&tmpdir_host_extension(None));
    let argv = vec!["doc.md".to_string()];
    let matched = registry.match_argv(&argv).expect("match");
    let ctx = build_match_context(&matched, matched.extension());
    let expanded = expand_and_validate(matched.extension(), &ctx).expect("expand");
    let tmp: PathBuf = expanded
        .temp_guard
        .as_ref()
        .expect("temp_guard")
        .path()
        .to_path_buf();
    assert!(
        tmp.is_dir(),
        "tmpdir must be present during host: {}",
        tmp.display()
    );
    drop(expanded);
    assert!(
        !tmp.exists(),
        "tmpdir must be deleted after mocked host exit: {}",
        tmp.display()
    );
}

#[test]
fn extensions_preexec_cleanup_deleted_on_preexec_failure() {
    let (cmd, args) = failing_preexec();
    let registry = registry_with(&tmpdir_host_extension(Some((cmd, &args))));
    let argv = vec!["doc.md".to_string()];
    let matched = registry.match_argv(&argv).expect("match");
    let ctx = build_match_context(&matched, matched.extension());
    let err = expand_and_validate(matched.extension(), &ctx).expect_err("preexec fail");
    assert!(matches!(err, ExtensionError::Preexec { .. }), "{err}");
    let tmp = wyvern::extensions::last_created_tmpdir().expect("tmpdir was created");
    assert!(
        !tmp.exists(),
        "tmpdir must be deleted immediately on preexec failure: {}",
        tmp.display()
    );
}

#[test]
fn extensions_preexec_cleanup_present_during_host() {
    let registry = registry_with(&tmpdir_host_extension(None));
    let argv = vec!["doc.md".to_string()];
    let matched = registry.match_argv(&argv).expect("match");
    let ctx: MatchContext<'_> = build_match_context(&matched, matched.extension());
    let expanded = expand_and_validate(matched.extension(), &ctx).expect("expand");
    let guard = expanded.temp_guard.as_ref().expect("temp_guard");
    assert!(
        guard.path().is_dir(),
        "tmpdir present during host: {}",
        guard.path().display()
    );
}

#[cfg(unix)]
#[test]
fn extensions_preexec_stdout_markdown_capture() {
    let json = serde_json::json!({
        "version": 1,
        "extensions": [{
            "id": "stdout-md",
            "match": { "positional_suffix": ".md" },
            "preexec": {
                "cmd": "printf",
                "args": ["# captured"],
                "stdout": "markdown"
            },
            "expand": {
                "command": { "type": "markdown", "content": "{preexec.stdout}" }
            }
        }]
    })
    .to_string();
    let registry = registry_with(&json);
    let argv = vec!["doc.md".to_string()];
    let matched = registry.match_argv(&argv).expect("match");
    let ctx = build_match_context(&matched, matched.extension());
    let expanded = expand_and_validate(matched.extension(), &ctx).expect("expand");
    assert_eq!(expanded.command["type"], "markdown");
    assert_eq!(expanded.command["content"], "# captured");
}
