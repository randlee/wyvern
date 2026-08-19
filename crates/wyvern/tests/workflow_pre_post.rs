//! Workflow pre/post runner: merge, stdin, dry-run argv, allowlist, cancel skip.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

use serde_json::json;
use wyvern::{
    run_wizard_workflow_loop, Allowlist, PipelineError, WorkflowRunner, WORKFLOW_SCRIPT_TIMEOUT,
};
use wyvern_host::{HostOptions, ViewerMode};

fn unique_path(prefix: &str) -> PathBuf {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("{prefix}-{}-{n}", std::process::id()))
}

fn workspace_share() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../share/wyvern")
}

fn workspace_ui() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../ui")
}

fn host_options(ui_root: PathBuf, url_file: PathBuf) -> HostOptions {
    HostOptions {
        bind: SocketAddr::from(([127, 0, 0, 1], 0)),
        ui_root,
        shared_ui_root: workspace_ui(),
        viewer: ViewerMode::None,
        dialog_url_env: true,
        dialog_url_file: Some(url_file),
        allow_non_loopback: false,
        session_timeout: Duration::from_secs(30),
        mock_picker: None,
    }
}

fn runner_for(tmp: &Path, share: PathBuf) -> WorkflowRunner {
    WorkflowRunner {
        allowlist: Allowlist {
            share_root: share,
            cwd: tmp.to_path_buf(),
            wizard_dir: tmp.to_path_buf(),
        },
        timeout: WORKFLOW_SCRIPT_TIMEOUT,
    }
}

fn write_script(dir: &Path, name: &str, body: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, body).expect("write script");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&path, perms).unwrap();
    }
    path
}

fn http_client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("http")
}

fn wait_for_url(path: &Path) -> String {
    let start = std::time::Instant::now();
    loop {
        if let Ok(url) = std::fs::read_to_string(path) {
            let url = url.trim().to_string();
            if !url.is_empty() {
                return url;
            }
        }
        if start.elapsed() > Duration::from_secs(15) {
            panic!("timed out waiting for {}", path.display());
        }
        thread::sleep(Duration::from_millis(20));
    }
}

fn wait_for_state(client: &reqwest::blocking::Client, base: &str) -> serde_json::Value {
    let url = format!("{base}/api/wizard/state");
    let start = std::time::Instant::now();
    loop {
        match client.get(&url).send() {
            Ok(resp) if resp.status() == reqwest::StatusCode::OK => {
                return resp.json().expect("state");
            }
            _ => {
                if start.elapsed() > Duration::from_secs(15) {
                    panic!("timed out waiting for {url}");
                }
                thread::sleep(Duration::from_millis(20));
            }
        }
    }
}

fn write_fixture_ui(root: &Path) {
    let pages = root.join("pages");
    std::fs::create_dir_all(&pages).unwrap();
    std::fs::write(
        pages.join("home.html"),
        "<!DOCTYPE html><title>home</title><h1>home</h1>",
    )
    .unwrap();
}

#[test]
fn pre_merges_config_patch() {
    let tmp = tempfile::tempdir().unwrap();
    let script = write_script(
        tmp.path(),
        "pre.py",
        "#!/usr/bin/env python3\nimport json\nprint(json.dumps({\"config_patch\":{\"patched\":True}}))\n",
    );
    let runner = runner_for(tmp.path(), tmp.path().to_path_buf());
    let spec = wyvern_schema::WorkflowSpec {
        pre: Some(wyvern_schema::WorkflowPath::new(
            script.to_string_lossy().into_owned(),
        )),
        post: None,
    };
    let mut config = json!({"seed": true});
    runner.run_pre(&spec, &mut config, false).expect("pre");
    assert_eq!(config["seed"], true);
    assert_eq!(config["patched"], true);
}

#[test]
fn post_receives_finish_stdin() {
    let tmp = tempfile::tempdir().unwrap();
    let marker = tmp.path().join("post.json");
    let script = write_script(
        tmp.path(),
        "post.py",
        &format!(
            "#!/usr/bin/env python3\nimport json,sys\nopen({:?},'w').write(sys.stdin.read())\n",
            marker
        ),
    );
    let runner = runner_for(tmp.path(), tmp.path().to_path_buf());
    let spec = wyvern_schema::WorkflowSpec {
        pre: None,
        post: Some(wyvern_schema::WorkflowPath::new(
            script.to_string_lossy().into_owned(),
        )),
    };
    let finish = json!({"button":"finish","data":{"k":1},"stack":[]});
    runner.run_post(&spec, &finish, false).expect("post");
    let got: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&marker).unwrap()).unwrap();
    assert_eq!(got["button"], "finish");
    assert_eq!(got["data"]["k"], 1);
}

#[test]
fn dry_run_appends_flag_to_pre_argv() {
    let tmp = tempfile::tempdir().unwrap();
    let marker = tmp.path().join("argv.json");
    let script = write_script(
        tmp.path(),
        "pre.py",
        &format!(
            "#!/usr/bin/env python3\nimport json,sys\nopen({:?},'w').write(json.dumps(sys.argv))\nprint(json.dumps({{\"config_patch\":{{}}}}))\n",
            marker
        ),
    );
    let runner = runner_for(tmp.path(), tmp.path().to_path_buf());
    let spec = wyvern_schema::WorkflowSpec {
        pre: Some(wyvern_schema::WorkflowPath::new(
            script.to_string_lossy().into_owned(),
        )),
        post: None,
    };
    let mut config = json!({});
    runner.run_pre(&spec, &mut config, true).expect("pre");
    let argv: Vec<String> =
        serde_json::from_str(&std::fs::read_to_string(&marker).unwrap()).unwrap();
    assert!(
        argv.iter().any(|a| a == "--dry-run"),
        "argv must include --dry-run: {argv:?}"
    );
}

#[test]
fn allowlist_denies_escape_and_does_not_start_host() {
    let tmp = tempfile::tempdir().unwrap();
    write_fixture_ui(tmp.path());
    let url_file = unique_path("wyvern-g4-deny-url");
    let runner = runner_for(tmp.path(), tmp.path().to_path_buf());
    let command = json!({
        "type": "wizard",
        "page": { "id": "home", "title": "T", "html": "pages/home.html" },
        "config": {},
        "workflow": { "pre": "../../../../etc/passwd" }
    });
    let err = run_wizard_workflow_loop(
        command,
        host_options(tmp.path().to_path_buf(), url_file.clone()),
        &runner,
        false,
    )
    .expect_err("denied");
    match err {
        PipelineError::Stage { stderr, exit_code } => {
            assert_eq!(exit_code, 9);
            let value: serde_json::Value = serde_json::from_str(&stderr).unwrap();
            assert_eq!(value["code"], "WORKFLOW_ERROR");
            assert_eq!(value["error"], "workflow");
        }
        other => panic!("expected stage, got {other:?}"),
    }
    assert!(
        !url_file.is_file()
            || std::fs::read_to_string(&url_file)
                .unwrap()
                .trim()
                .is_empty(),
        "pre failure must not start the host"
    );
}

#[test]
fn pre_config_patch_is_on_first_wizard_state() {
    let tmp = tempfile::tempdir().unwrap();
    write_fixture_ui(tmp.path());
    let share = workspace_share();
    let runner = runner_for(tmp.path(), share);
    let url_file = unique_path("wyvern-g4-pre-url");
    let command = json!({
        "type": "wizard",
        "page": { "id": "home", "title": "T", "html": "pages/home.html" },
        "config": { "seed": true },
        "workflow": { "pre": "{wyvern_share}/testdata/workflow/pre.py" },
        "width": 480,
        "height": 320
    });
    let host = host_options(tmp.path().to_path_buf(), url_file.clone());
    let handle = thread::spawn(move || run_wizard_workflow_loop(command, host, &runner, false));
    let dialog_url = wait_for_url(&url_file);
    let base = dialog_url
        .split_once("/wizard/")
        .map(|(b, _)| b.to_string())
        .expect("wizard url");
    let client = http_client();
    let state = wait_for_state(&client, &base);
    assert_eq!(state["config"]["seed"], true);
    assert_eq!(state["config"]["patched"], true);

    let stack = json!([{ "page": state["page"], "data": {} }]);
    let resp = client
        .post(format!("{base}/api/wizard/finish"))
        .json(&json!({"button":"cancel","data":{},"stack": stack}))
        .send()
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    handle.join().expect("join").expect("cancel ok");
}

#[test]
fn cancel_skips_post() {
    let tmp = tempfile::tempdir().unwrap();
    write_fixture_ui(tmp.path());
    let marker = tmp.path().join("post-should-not-exist");
    let post = write_script(
        tmp.path(),
        "post.py",
        &format!(
            "#!/usr/bin/env python3\nopen({:?},'w').write('ran')\n",
            marker
        ),
    );
    let runner = runner_for(tmp.path(), tmp.path().to_path_buf());
    let url_file = unique_path("wyvern-g4-cancel-url");
    let command = json!({
        "type": "wizard",
        "page": { "id": "home", "title": "T", "html": "pages/home.html" },
        "config": {},
        "workflow": { "post": post.to_string_lossy() },
        "width": 480,
        "height": 320
    });
    let host = host_options(tmp.path().to_path_buf(), url_file.clone());
    let handle = thread::spawn(move || run_wizard_workflow_loop(command, host, &runner, false));
    let dialog_url = wait_for_url(&url_file);
    let base = dialog_url
        .split_once("/wizard/")
        .map(|(b, _)| b.to_string())
        .expect("wizard url");
    let client = http_client();
    let state = wait_for_state(&client, &base);
    let stack = json!([{ "page": state["page"], "data": {} }]);
    let resp = client
        .post(format!("{base}/api/wizard/finish"))
        .json(&json!({"button":"cancel","data":{},"stack": stack}))
        .send()
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    let stdout = handle.join().expect("join").expect("ok");
    assert!(!marker.exists(), "cancel must skip post; stdout={stdout}");
}

#[test]
fn dismissed_skips_post() {
    let tmp = tempfile::tempdir().unwrap();
    write_fixture_ui(tmp.path());
    let marker = tmp.path().join("post-dismissed");
    let post = write_script(
        tmp.path(),
        "post.py",
        &format!(
            "#!/usr/bin/env python3\nopen({:?},'w').write('ran')\n",
            marker
        ),
    );
    let runner = runner_for(tmp.path(), tmp.path().to_path_buf());
    let url_file = unique_path("wyvern-g4-dismiss-url");
    let command = json!({
        "type": "wizard",
        "page": { "id": "home", "title": "T", "html": "pages/home.html" },
        "config": {},
        "workflow": { "post": post.to_string_lossy() },
        "width": 480,
        "height": 320
    });
    let host = host_options(tmp.path().to_path_buf(), url_file.clone());
    let handle = thread::spawn(move || run_wizard_workflow_loop(command, host, &runner, false));
    let dialog_url = wait_for_url(&url_file);
    let base = dialog_url
        .split_once("/wizard/")
        .map(|(b, _)| b.to_string())
        .expect("wizard url");
    let client = http_client();
    let state = wait_for_state(&client, &base);
    let stack = json!([{ "page": state["page"], "data": {} }]);
    let resp = client
        .post(format!("{base}/api/wizard/finish"))
        .json(&json!({"button":"dismissed","data":{},"stack": stack}))
        .send()
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    handle.join().expect("join").expect("ok");
    assert!(!marker.exists(), "dismissed must skip post");
}
