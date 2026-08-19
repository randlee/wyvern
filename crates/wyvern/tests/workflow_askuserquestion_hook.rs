//! Pre merge → mock finish → post against an isolated hook dir (g.5).

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde_json::{json, Value};
use wyvern::{Allowlist, WorkflowRunner, WORKFLOW_SCRIPT_TIMEOUT};

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn workspace_share() -> PathBuf {
    workspace_root().join("share/wyvern")
}

fn script(name: &str) -> PathBuf {
    workspace_root().join("scripts/ext").join(name)
}

fn runner_for(tmp: &Path) -> WorkflowRunner {
    WorkflowRunner {
        allowlist: Allowlist {
            share_root: workspace_share(),
            cwd: workspace_root(),
            wizard_dir: tmp.to_path_buf(),
        },
        timeout: WORKFLOW_SCRIPT_TIMEOUT,
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

fn with_hook_env<T>(home: &Path, repo: &Path, f: impl FnOnce() -> T) -> T {
    let _guard = ENV_LOCK.lock().expect("env lock");
    let old_home = std::env::var_os("HOME");
    let old_repo = std::env::var_os("WYVERN_REPO_ROOT");
    std::env::set_var("HOME", home);
    std::env::set_var("WYVERN_REPO_ROOT", repo);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
    match old_home {
        Some(value) => std::env::set_var("HOME", value),
        None => std::env::remove_var("HOME"),
    }
    match old_repo {
        Some(value) => std::env::set_var("WYVERN_REPO_ROOT", value),
        None => std::env::remove_var("WYVERN_REPO_ROOT"),
    }
    match result {
        Ok(value) => value,
        Err(payload) => std::panic::resume_unwind(payload),
    }
}

fn load_jsonc(path: &Path) -> Value {
    let raw = std::fs::read_to_string(path).unwrap_or_else(|_| "{}".into());
    let stripped: String = raw
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    serde_json::from_str(&stripped).unwrap_or(json!({}))
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

#[test]
fn pre_merges_hook_state_then_post_writes_markers() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().join("home");
    let repo = tmp.path().join("repo");
    std::fs::create_dir_all(&home).unwrap();
    std::fs::create_dir_all(&repo).unwrap();
    let runner = runner_for(tmp.path());

    with_hook_env(&home, &repo, || {
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

        let global_settings = home.join(".claude/settings.json");
        let repo_settings = repo.join(".claude/settings.local.json");
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
        assert_eq!(hooks[0]["matcher"], "AskUserQuestion");
        assert_eq!(
            hooks[0]["hooks"][0]["managed_by"],
            "wyvern:askuserquestion-hook"
        );
        assert_eq!(hooks[0]["hooks"][0]["version"], 1);

        let sidecar = home.join(".claude/wyvern-askuserquestion-bin");
        assert!(sidecar.is_file(), "global sidecar baked WYVERN_BIN");

        let mut config_after = json!({});
        runner
            .run_pre(&pre_spec(), &mut config_after, false)
            .expect("pre after install");
        assert_eq!(config_after["hook_state"]["global"]["enabled"], true);
        assert_eq!(config_after["hook_state"]["global"]["installed"], true);
        assert_eq!(config_after["hook_state"]["repo"]["enabled"], true);
        assert_eq!(config_after["hook_state"]["repo"]["installed"], true);
    });
}

#[test]
fn dry_run_writes_no_hook_files() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().join("home");
    let repo = tmp.path().join("repo");
    std::fs::create_dir_all(&home).unwrap();
    std::fs::create_dir_all(&repo).unwrap();
    let runner = runner_for(tmp.path());

    with_hook_env(&home, &repo, || {
        runner
            .run_post(&post_spec(), &finish_hook_config(true, true), true)
            .expect("dry-run post");
        assert!(!home.join(".claude/settings.json").exists());
        assert!(!repo.join(".claude/settings.local.json").exists());
    });
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
        home.join(".claude/settings.json"),
        serde_json::to_string_pretty(&unrelated).unwrap(),
    )
    .unwrap();
    let runner = runner_for(tmp.path());

    with_hook_env(&home, &repo, || {
        runner
            .run_post(&post_spec(), &finish_hook_config(true, false), false)
            .expect("enable global");
        let after_enable = load_jsonc(&home.join(".claude/settings.json"));
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
        let after_disable = load_jsonc(&home.join(".claude/settings.json"));
        assert_eq!(managed_hooks(&after_disable).len(), 0);
        assert_eq!(after_disable["hooks"]["PreToolUse"][0]["matcher"], "Bash");
        assert!(!repo.join(".claude/settings.local.json").exists());
    });

    let status = std::process::Command::new("python3")
        .arg(script("apply-askuserquestion-hook.py"))
        .arg("--remove")
        .env("HOME", &home)
        .env("WYVERN_REPO_ROOT", &repo)
        .status()
        .expect("remove");
    assert!(status.success());
    let after_remove = load_jsonc(&home.join(".claude/settings.json"));
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
