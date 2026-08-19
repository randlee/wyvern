//! Pre merge → mock finish → post against an isolated hook dir (g.5).

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use serde_json::{json, Value};
use wyvern::{Allowlist, WorkflowRunner, WORKFLOW_SCRIPT_TIMEOUT};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn workspace_share() -> PathBuf {
    workspace_root().join("share/wyvern")
}

fn script(name: &str) -> PathBuf {
    workspace_root().join("scripts/ext").join(name)
}

fn resolve_python() -> &'static str {
    for name in ["python3", "py", "python"] {
        if std::process::Command::new(name)
            .arg("-c")
            .arg("import sys")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
        {
            return name;
        }
    }
    panic!("python3, py, or python is required for AskUserQuestion hook tests");
}

/// Isolate hook dirs via per-Command `extra_env` (`WYVERN_HOME` / `WYVERN_REPO_ROOT`).
///
/// Do not mutate process-global `HOME`/`USERPROFILE` — that races under
/// `--test-threads` and does not isolate on Windows (same rationale as
/// wyvern-host picker/markdown test hooks: prefer per-call injection over a
/// process-global ENV_LOCK).
fn runner_for(tmp: &Path, home: &Path, repo: &Path) -> WorkflowRunner {
    WorkflowRunner {
        allowlist: Allowlist {
            share_root: workspace_share(),
            cwd: workspace_root(),
            wizard_dir: tmp.to_path_buf(),
        },
        timeout: WORKFLOW_SCRIPT_TIMEOUT,
        extra_env: vec![
            (
                OsString::from("WYVERN_HOME"),
                home.as_os_str().to_os_string(),
            ),
            (
                OsString::from("WYVERN_REPO_ROOT"),
                repo.as_os_str().to_os_string(),
            ),
        ],
    }
}

fn pre_spec() -> wyvern_schema::WorkflowSpec {
    wyvern_schema::WorkflowSpec {
        pre: Some(wyvern_schema::WorkflowPath::new(
            script("query-askuserquestion-hook.py")
                .to_string_lossy()
                .into_owned(),
        )),
        post: None,
    }
}

fn post_spec() -> wyvern_schema::WorkflowSpec {
    wyvern_schema::WorkflowSpec {
        pre: None,
        post: Some(wyvern_schema::WorkflowPath::new(
            script("apply-askuserquestion-hook.py")
                .to_string_lossy()
                .into_owned(),
        )),
    }
}

fn load_jsonc(path: &Path) -> Value {
    let raw = std::fs::read_to_string(path).unwrap_or_else(|err| {
        panic!("read {}: {err}", path.display());
    });
    let stripped: String = raw
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    serde_json::from_str(&stripped).unwrap_or_else(|err| {
        panic!("parse JSONC {}: {err}", path.display());
    })
}

fn managed_hooks(settings: &Value) -> Vec<&Value> {
    settings["hooks"]["PreToolUse"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|entry| {
            entry["hooks"].as_array().is_some_and(|hooks| {
                hooks
                    .iter()
                    .any(|hook| hook["managed_by"] == "wyvern:askuserquestion-hook")
            })
        })
        .collect()
}

fn finish_hook_config(global: bool, repo: bool) -> Value {
    json!({
        "button": "finish",
        "data": {
            "hook_config": {
                "global": { "enabled": global },
                "repo": { "enabled": repo }
            }
        },
        "stack": []
    })
}

/// Strip the Windows extended-length prefix that `canonicalize()` adds (`\\?\`).
///
/// Python's `os.path.realpath` + `subprocess.list2cmdline` bake the ordinary
/// drive path, so tests must not require a verbatim canonical string match.
fn strip_windows_extended_prefix(path: &str) -> &str {
    path.strip_prefix(r"\\?\")
        .or_else(|| path.strip_prefix("//?/"))
        .unwrap_or(path)
}

fn normalize_path_for_compare(path: &str) -> String {
    let unified = strip_windows_extended_prefix(path).replace('\\', "/");
    if cfg!(windows) {
        unified.to_ascii_lowercase()
    } else {
        unified
    }
}

fn utf8_file_name(path: &Path) -> Option<&str> {
    path.file_name().and_then(|name| name.to_str())
}

fn quoted_tokens(command: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let mut chars = command.char_indices();
    while let Some((idx, ch)) = chars.next() {
        if ch != '\'' && ch != '"' {
            continue;
        }
        let start = idx + ch.len_utf8();
        let mut end = None;
        for (inner_idx, inner) in chars.by_ref() {
            if inner == ch {
                end = Some(inner_idx);
                break;
            }
        }
        if let Some(end) = end {
            tokens.push(&command[start..end]);
        }
    }
    tokens
}

/// True when a space-containing script path appears as one quoted token.
///
/// Matches POSIX `shlex.quote` (`'…'`) and Windows `list2cmdline` (`"…"`).
/// Comparison uses basename + parent leaf so a Windows `\\?\` canonical
/// prefix or drive-letter case difference cannot fail a correctly quoted
/// command.
fn command_quotes_script(command: &str, script_path: &Path) -> bool {
    let file_name = match utf8_file_name(script_path) {
        Some(name) if !name.is_empty() => name,
        _ => return false,
    };
    let parent_leaf = script_path
        .parent()
        .and_then(utf8_file_name)
        .filter(|leaf| !leaf.is_empty());
    let raw = script_path.to_string_lossy();
    if !raw.contains(' ') {
        return true;
    }

    let name_norm = normalize_path_for_compare(file_name);
    let parent_norm = parent_leaf.map(normalize_path_for_compare);
    quoted_tokens(command).into_iter().any(|token| {
        if !token.contains(' ') {
            return false;
        }
        let token_norm = normalize_path_for_compare(token);
        if !token_norm.contains(&name_norm) {
            return false;
        }
        parent_norm
            .as_ref()
            .is_none_or(|parent| token_norm.contains(parent))
    })
}

fn assert_command_quotes_script(command: &str, script_path: &Path) {
    assert!(
        command_quotes_script(command, script_path),
        "space-containing script path must be quoted in hook command: {command}"
    );
}

#[test]
fn pre_merges_hook_state_then_post_writes_markers() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().join("home");
    let repo = tmp.path().join("repo");
    std::fs::create_dir_all(&home).unwrap();
    std::fs::create_dir_all(&repo).unwrap();
    let runner = runner_for(tmp.path(), &home, &repo);

    let mut config = json!({});
    runner
        .run_pre(&pre_spec(), &mut config, false)
        .expect("pre");
    assert_eq!(config["hook_state"]["global"]["enabled"], false);
    assert_eq!(config["hook_state"]["global"]["installed"], false);
    assert_eq!(config["hook_state"]["repo"]["enabled"], false);
    assert_eq!(config["hook_state"]["repo"]["installed"], false);

    runner
        .run_post(&post_spec(), &finish_hook_config(true, true), false)
        .expect("post");

    let global_settings = home.join(".claude").join("settings.json");
    let repo_settings = repo.join(".claude").join("settings.local.json");
    assert!(global_settings.is_file(), "global settings created");
    assert!(repo_settings.is_file(), "repo settings created");

    let global_text = std::fs::read_to_string(&global_settings).unwrap();
    assert!(
        global_text.contains("# wyvern:askuserquestion-hook v1"),
        "global marker comment: {global_text}"
    );
    let global_json = load_jsonc(&global_settings);
    let hooks = managed_hooks(&global_json);
    assert_eq!(hooks.len(), 1);
    let command = hooks[0]["hooks"][0]["command"].as_str().expect("command");
    assert!(
        command.contains("apply-askuserquestion-hook.py") && command.contains("--invoke"),
        "{command}"
    );
    assert!(
        !command.contains("${WYVERN_SHARE}") && !command.contains("{wyvern_share}"),
        "command must bake an absolute script path: {command}"
    );
    assert_command_quotes_script(command, &script("apply-askuserquestion-hook.py"));
    assert_eq!(hooks[0]["matcher"], "AskUserQuestion");
    assert_eq!(
        hooks[0]["hooks"][0]["managed_by"],
        "wyvern:askuserquestion-hook"
    );
    assert_eq!(hooks[0]["hooks"][0]["version"], 1);

    let sidecar = home.join(".claude").join("wyvern-askuserquestion-bin");
    assert!(sidecar.is_file(), "global sidecar baked WYVERN_BIN");

    let mut config_after = json!({});
    runner
        .run_pre(&pre_spec(), &mut config_after, false)
        .expect("pre after install");
    assert_eq!(config_after["hook_state"]["global"]["enabled"], true);
    assert_eq!(config_after["hook_state"]["global"]["installed"], true);
    assert_eq!(config_after["hook_state"]["repo"]["enabled"], true);
    assert_eq!(config_after["hook_state"]["repo"]["installed"], true);
}

#[test]
fn hook_command_quotes_space_containing_script_path() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().join("home");
    let repo = tmp.path().join("repo");
    let spaced = tmp.path().join("path with spaces");
    std::fs::create_dir_all(&home).unwrap();
    std::fs::create_dir_all(&repo).unwrap();
    std::fs::create_dir_all(&spaced).unwrap();
    let copied = spaced.join("apply-askuserquestion-hook.py");
    std::fs::copy(script("apply-askuserquestion-hook.py"), &copied).expect("copy apply script");

    let status = std::process::Command::new(resolve_python())
        .arg(&copied)
        .env("WYVERN_HOME", &home)
        .env("WYVERN_REPO_ROOT", &repo)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            child
                .stdin
                .as_mut()
                .expect("stdin")
                .write_all(finish_hook_config(true, false).to_string().as_bytes())?;
            child.wait_with_output()
        })
        .expect("apply from spaced path");
    assert!(
        status.status.success(),
        "apply failed: {}",
        String::from_utf8_lossy(&status.stderr)
    );

    let settings = load_jsonc(&home.join(".claude").join("settings.json"));
    let hooks = managed_hooks(&settings);
    assert_eq!(hooks.len(), 1);
    let command = hooks[0]["hooks"][0]["command"].as_str().expect("command");
    assert_command_quotes_script(command, &copied);
    assert!(
        command.contains("path with spaces"),
        "baked command should mention the spaced directory: {command}"
    );
}

#[test]
fn command_quote_check_accepts_list2cmdline_without_verbatim_prefix() {
    let script = Path::new(
        r"C:\Users\runner\AppData\Local\Temp\.tmpX\path with spaces\apply-askuserquestion-hook.py",
    );
    let command = r#""C:\Python\python.exe" "C:\Users\runner\AppData\Local\Temp\.tmpX\path with spaces\apply-askuserquestion-hook.py" --invoke"#;
    assert!(command_quotes_script(command, script));

    let verbatim = Path::new(
        r"\\?\C:\Users\runner\AppData\Local\Temp\.tmpX\path with spaces\apply-askuserquestion-hook.py",
    );
    assert!(
        command_quotes_script(command, verbatim),
        "Windows canonicalize() prefix must not be required in the baked command"
    );
}

#[test]
fn command_quote_check_rejects_unquoted_spaced_path() {
    let script = Path::new("/tmp/path with spaces/apply-askuserquestion-hook.py");
    let command = "/usr/bin/python3 /tmp/path with spaces/apply-askuserquestion-hook.py --invoke";
    assert!(!command_quotes_script(command, script));
}

#[test]
fn dry_run_writes_no_hook_files() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().join("home");
    let repo = tmp.path().join("repo");
    std::fs::create_dir_all(&home).unwrap();
    std::fs::create_dir_all(&repo).unwrap();
    let runner = runner_for(tmp.path(), &home, &repo);

    runner
        .run_post(&post_spec(), &finish_hook_config(true, true), true)
        .expect("dry-run post");
    assert!(!home.join(".claude").join("settings.json").exists());
    assert!(!repo.join(".claude").join("settings.local.json").exists());
}

#[test]
fn both_disabled_and_remove_strip_only_managed_entries() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().join("home");
    let repo = tmp.path().join("repo");
    std::fs::create_dir_all(home.join(".claude")).unwrap();
    std::fs::create_dir_all(repo.join(".claude")).unwrap();
    let unrelated = json!({
        "hooks": {
            "PreToolUse": [
                {
                    "matcher": "Bash",
                    "hooks": [{ "type": "command", "command": "echo keep" }]
                }
            ]
        }
    });
    std::fs::write(
        home.join(".claude").join("settings.json"),
        serde_json::to_string_pretty(&unrelated).unwrap(),
    )
    .unwrap();
    let runner = runner_for(tmp.path(), &home, &repo);

    runner
        .run_post(&post_spec(), &finish_hook_config(true, false), false)
        .expect("enable global");
    let after_enable = load_jsonc(&home.join(".claude").join("settings.json"));
    assert_eq!(
        after_enable["hooks"]["PreToolUse"]
            .as_array()
            .unwrap()
            .len(),
        2
    );

    runner
        .run_post(&post_spec(), &finish_hook_config(false, false), false)
        .expect("disable both");
    let after_disable = load_jsonc(&home.join(".claude").join("settings.json"));
    assert_eq!(managed_hooks(&after_disable).len(), 0);
    assert_eq!(after_disable["hooks"]["PreToolUse"][0]["matcher"], "Bash");
    assert!(!repo.join(".claude").join("settings.local.json").exists());

    let status = std::process::Command::new(resolve_python())
        .arg(script("apply-askuserquestion-hook.py"))
        .arg("--remove")
        .env("WYVERN_HOME", &home)
        .env("WYVERN_REPO_ROOT", &repo)
        .status()
        .expect("remove");
    assert!(status.success());
    let after_remove = load_jsonc(&home.join(".claude").join("settings.json"));
    assert_eq!(managed_hooks(&after_remove).len(), 0);
    assert_eq!(after_remove["hooks"]["PreToolUse"][0]["matcher"], "Bash");
}

#[test]
fn shipped_wizard_declares_pre_post_and_toggles() {
    let wizard_path = workspace_share().join("examples/askuserquestion-hook/wizard.json");
    let wizard: Value =
        serde_json::from_str(&std::fs::read_to_string(wizard_path).unwrap()).unwrap();
    wyvern_schema::validate(&wizard).expect("wizard schema");
    assert_eq!(wizard["page"]["id"], "toggles");
    assert_eq!(
        wizard["workflow"]["pre"],
        "{wyvern_share}/scripts/ext/query-askuserquestion-hook.py"
    );
    assert_eq!(
        wizard["workflow"]["post"],
        "{wyvern_share}/scripts/ext/apply-askuserquestion-hook.py"
    );
    let app =
        std::fs::read_to_string(workspace_share().join("examples/askuserquestion-hook/app.js"))
            .unwrap();
    assert!(app.contains("hook_config"));
    assert!(
        !app.contains("settings.json"),
        "page JS must not touch hook files"
    );
}
