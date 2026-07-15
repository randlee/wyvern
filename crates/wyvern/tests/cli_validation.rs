//! CLI integration: load → validate → run → emit.

use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

fn wyvern() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_wyvern"));
    // Isolate from developer/CI WYVERN_LOG so stdout/stderr assertions stay stable.
    cmd.env_remove("WYVERN_LOG");
    cmd.env("WYVERN_UI_ROOT", workspace_ui_root());
    cmd
}

fn workspace_ui_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../ui")
}

/// Spawns `wyvern`; detects child panic/abort/signal failures.
fn run_wyvern(cmd: &mut Command) -> std::process::Output {
    cmd.env_remove("WYVERN_LOG");
    let output = cmd.output().expect("spawn wyvern");
    assert_child_ok(&output);
    output
}

fn run_json(json: &str) -> (i32, String, String) {
    let output = run_wyvern(wyvern().arg(json));
    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    (code, stdout, stderr)
}

mod child_assert {
    /// Pure predicate — unit-testable without synthesizing `ExitStatus`.
    /// `code == -1` covers Unix signal exits where `status.code()` is `None`.
    pub(super) fn child_failed(stderr: &str, code: i32) -> bool {
        stderr.contains("panicked at")
            || stderr.contains("misaligned pointer")
            || stderr.contains("cannot unwind")
            || stderr.contains("abort")
            || code == -1
    }

    pub(super) fn assert_child_ok(output: &std::process::Output) {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let code = output.status.code().unwrap_or(-1);
        assert!(
            !child_failed(&stderr, code),
            "wyvern child panicked/aborted (use --test-threads=1 on macOS):\n\
             code={code:?}\nstderr={stderr}"
        );
    }
}

use child_assert::{assert_child_ok, child_failed};

fn stderr_json(stderr: &str) -> serde_json::Value {
    // Host may print WYVERN_DIALOG_URL=… before JSON error lines.
    let json_line = stderr
        .lines()
        .rev()
        .find(|line| line.trim_start().starts_with('{'))
        .unwrap_or(stderr.trim());
    serde_json::from_str(json_line.trim()).unwrap_or_else(|err| {
        panic!("stderr is not JSON ({err}): {stderr:?}");
    })
}

/// Spawn wyvern with ephemeral bind; discover URL from `WYVERN_DIALOG_URL=` on stderr.
/// Avoids TOCTOU from pre-binding a free port then rebinding in the child.
fn spawn_wyvern_ephemeral(args: &[&str]) -> (Child, String, thread::JoinHandle<String>) {
    let mut child = wyvern()
        .args(args.iter().copied().chain(["--bind", "127.0.0.1:0"]))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn");

    let stderr = child.stderr.take().expect("stderr");
    let (url_tx, url_rx) = std::sync::mpsc::channel::<String>();
    let stderr_handle = thread::spawn(move || {
        let mut collected = String::new();
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            let line = line.expect("stderr line");
            collected.push_str(&line);
            collected.push('\n');
            if let Some(url) = line.strip_prefix("WYVERN_DIALOG_URL=") {
                let _ = url_tx.send(url.to_string());
            }
        }
        collected
    });

    let dialog_url = url_rx
        .recv_timeout(Duration::from_secs(15))
        .expect("timed out waiting for WYVERN_DIALOG_URL on stderr");
    (child, dialog_url, stderr_handle)
}

fn wait_child_with_stderr(
    mut child: Child,
    stderr_handle: thread::JoinHandle<String>,
) -> std::process::Output {
    let stdout = child.stdout.take().expect("stdout");
    let status = child.wait().expect("wait");
    let mut stdout_buf = Vec::new();
    {
        use std::io::Read;
        let mut stdout = stdout;
        stdout.read_to_end(&mut stdout_buf).expect("read stdout");
    }
    let stderr = stderr_handle.join().expect("stderr thread");
    std::process::Output {
        status,
        stdout: stdout_buf,
        stderr: stderr.into_bytes(),
    }
}

#[test]
fn child_failed_detects_panic_marker() {
    assert!(child_failed("thread 'main' panicked at winit\n", 0));
    assert!(!child_failed("", 0));
}

#[test]
fn child_failed_detects_signal_exit() {
    assert!(child_failed("", -1)); // None mapped via unwrap_or(-1)
}

#[test]
fn cli_chrome_missing_title_validation_error() {
    let (code, _stdout, stderr) = run_json(r#"{"type":"chrome"}"#);
    assert_ne!(code, 0);
    let value = stderr_json(&stderr);
    assert_eq!(value["error"], "validation");
    assert_eq!(value["field"], "title");
}

#[test]
fn cli_empty_object_missing_type() {
    let (code, _stdout, stderr) = run_json("{}");
    assert_ne!(code, 0);
    let value = stderr_json(&stderr);
    assert_eq!(value["error"], "validation");
    assert_eq!(value["field"], "type");
}

#[test]
fn cli_type_null_validation_error() {
    let (code, _stdout, stderr) = run_json(r#"{"type":null,"title":"T"}"#);
    assert_ne!(code, 0);
    let value = stderr_json(&stderr);
    assert_eq!(value["error"], "validation");
    assert_eq!(value["field"], "type");
    assert!(value["message"].as_str().unwrap().contains("null"));
}

#[test]
fn cli_unknown_field_validation_error() {
    let (code, _stdout, stderr) = run_json(r#"{"type":"chrome","title":"T","extra":1}"#);
    assert_ne!(code, 0);
    let value = stderr_json(&stderr);
    assert_eq!(value["error"], "validation");
    assert_eq!(value["field"], "extra");
}

#[test]
fn cli_type_message_level_invalid_validation_error() {
    let (code, stdout, stderr) = run_json(
        r#"{"type":"message","title":"T","message":"Hi","buttons":"ok","level":"critical"}"#,
    );
    assert_ne!(code, 0);
    assert!(stdout.trim().is_empty(), "stdout={stdout}");
    let value = stderr_json(&stderr);
    assert_eq!(value["error"], "validation");
    assert_eq!(value["field"], "level");
    assert!(value["message"]
        .as_str()
        .unwrap()
        .contains("expected one of"));
}

#[test]
fn cli_type_message_missing_buttons_validation_error() {
    let (code, _stdout, stderr) = run_json(r#"{"type":"message","title":"T","message":"Hi"}"#);
    assert_ne!(code, 0);
    let value = stderr_json(&stderr);
    assert_eq!(value["error"], "validation");
    assert_eq!(value["field"], "buttons");
}

#[test]
fn cli_input_multiline_with_file_validation_error() {
    let (code, stdout, stderr) =
        run_json(r#"{"type":"input","title":"T","message":"M","mode":"file","multiline":true}"#);
    assert_ne!(code, 0);
    assert!(stdout.trim().is_empty(), "stdout={stdout}");
    let value = stderr_json(&stderr);
    assert_eq!(value["error"], "validation");
    assert_eq!(value["field"], "multiline");
}

#[test]
fn cli_input_filter_with_text_validation_error() {
    let (code, stdout, stderr) =
        run_json(r#"{"type":"input","title":"T","message":"M","filter":["*.rs"]}"#);
    assert_ne!(code, 0);
    assert!(stdout.trim().is_empty(), "stdout={stdout}");
    let value = stderr_json(&stderr);
    assert_eq!(value["error"], "validation");
    assert_eq!(value["field"], "filter");
}

#[test]
fn cli_type_unknown_validation_error() {
    let (code, _stdout, stderr) = run_json(r#"{"type":"unknown"}"#);
    assert_ne!(code, 0);
    let value = stderr_json(&stderr);
    assert_eq!(value["error"], "validation");
    assert_eq!(value["field"], "type");
}

#[test]
fn cli_markdown_missing_file_is_io() {
    let (code, stdout, stderr) =
        run_json(r#"{"type":"markdown","file":"/definitely/missing/wyvern-b5.md"}"#);
    assert_ne!(code, 0);
    assert!(stdout.trim().is_empty(), "stdout={stdout}");
    let value = stderr_json(&stderr);
    assert_eq!(value["error"], "io");
    assert_eq!(value["field"], "file");
}

#[test]
fn cli_markdown_both_file_and_content_validation_error() {
    let (code, stdout, stderr) =
        run_json(r##"{"type":"markdown","file":"doc.md","content":"# Hi"}"##);
    assert_ne!(code, 0);
    assert!(stdout.trim().is_empty(), "stdout={stdout}");
    let value = stderr_json(&stderr);
    assert_eq!(value["error"], "validation");
    assert_eq!(value["field"], "file");
    assert!(value["message"]
        .as_str()
        .unwrap()
        .contains("exactly one of"));
}

#[test]
fn cli_action_show_state_error() {
    let (code, _stdout, stderr) = run_json(r#"{"action":"show"}"#);
    assert_ne!(code, 0);
    let value = stderr_json(&stderr);
    assert_eq!(value["error"], "state");
    assert_eq!(value["field"], "action");
}

#[test]
fn cli_wrong_title_type_expected_got() {
    let (code, _stdout, stderr) = run_json(r#"{"type":"chrome","title":123}"#);
    assert_ne!(code, 0);
    let value = stderr_json(&stderr);
    assert_eq!(value["error"], "validation");
    assert_eq!(value["field"], "title");
    let message = value["message"].as_str().unwrap();
    assert!(message.contains("expected string"));
    assert!(message.contains("number"));
}

/// Incomplete wizard JSON still fails validation (missing `page`).
#[test]
fn cli_wizard_incomplete_is_validation_error() {
    let (code, stdout, stderr) = run_json(r#"{"type":"wizard"}"#);
    assert_ne!(code, 0);
    assert!(stdout.trim().is_empty(), "stdout={stdout}");
    let value = stderr_json(&stderr);
    assert_eq!(value["error"], "validation");
    assert_eq!(value["field"], "page");
}

#[test]
fn cli_chrome_viewer_none_posts_ok() {
    let json = r#"{"type":"chrome","title":"T"}"#;
    let (child, dialog_url, stderr_handle) = spawn_wyvern_ephemeral(&[json, "--viewer", "none"]);
    let base = dialog_base_url(&dialog_url);

    let client = wait_for_http(&format!("{base}/api/dialog"));
    let dialog: serde_json::Value = client
        .get(format!("{base}/api/dialog"))
        .send()
        .expect("dialog")
        .json()
        .expect("json");
    assert_eq!(dialog["type"], "chrome");
    assert_eq!(dialog["title"], "T");

    let ack: serde_json::Value = client
        .post(format!("{base}/api/result"))
        .json(&serde_json::json!({"button": "ok"}))
        .send()
        .expect("result")
        .json()
        .expect("ack json");
    assert_eq!(ack["ok"], true);

    let output = child.wait_with_output().expect("wait");
    let _ = stderr_handle.join();
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), r#"{"button":"ok"}"#);
}

#[test]
fn cli_message_viewer_none_posts_ok() {
    let json = r#"{"type":"message","title":"T","message":"Hi","buttons":"ok"}"#;
    let (child, dialog_url, stderr_handle) = spawn_wyvern_ephemeral(&[json, "--viewer", "none"]);
    let base = dialog_base_url(&dialog_url);

    let client = wait_for_http(&format!("{base}/api/dialog"));
    let dialog: serde_json::Value = client
        .get(format!("{base}/api/dialog"))
        .send()
        .expect("dialog")
        .json()
        .expect("json");
    assert_eq!(dialog["type"], "message");

    let ack: serde_json::Value = client
        .post(format!("{base}/api/result"))
        .json(&serde_json::json!({"button":"ok"}))
        .send()
        .expect("post")
        .json()
        .expect("ack");
    assert_eq!(ack["ok"], true);

    let output = wait_child_with_stderr(child, stderr_handle);
    assert_child_ok(&output);
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), r#"{"button":"ok"}"#);
}

#[test]
fn cli_message_omitted_viewer_defaults_to_embedded() {
    // Product default is embedded. Isolate a wyvern copy without a sibling
    // wyvern-viewer so discovery fails closed (HOST_VIEWER_ERROR), not silent none.
    let tmp = tempfile::tempdir().expect("tmp");
    let wyvern_src = env!("CARGO_BIN_EXE_wyvern");
    let wyvern_dst = tmp.path().join(if cfg!(windows) {
        "wyvern.exe"
    } else {
        "wyvern"
    });
    std::fs::copy(wyvern_src, &wyvern_dst).expect("copy wyvern");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&wyvern_dst).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&wyvern_dst, perms).unwrap();
    }

    let json = r#"{"type":"message","title":"T","message":"Hi","buttons":"ok"}"#;
    let output = Command::new(&wyvern_dst)
        .env_remove("WYVERN_LOG")
        .env_remove("WYVERN_VIEWER")
        .env_remove("WYVERN_VIEWER_BIN")
        .env_remove("CARGO_BIN_EXE_wyvern-viewer")
        .env("PATH", tmp.path())
        .env("WYVERN_UI_ROOT", workspace_ui_root())
        .arg(json)
        .output()
        .expect("spawn isolated wyvern");
    assert_child_ok(&output);
    assert_eq!(
        output.status.code(),
        Some(wyvern_schema::ErrorCode::HostViewerError.exit_code())
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    let value = stderr_json(&stderr);
    assert_eq!(value["code"], "HOST_VIEWER_ERROR");
    assert!(
        value["message"]
            .as_str()
            .unwrap_or("")
            .contains("wyvern-viewer")
            || value["cause"]
                .as_str()
                .unwrap_or("")
                .contains("wyvern-viewer"),
        "stderr={stderr}"
    );
}

#[test]
fn cli_named_viewer_missing_is_host_viewer_error() {
    // AC3: --viewer safari with forced-missing registry entry → HOST_VIEWER_ERROR.
    let tmp = tempfile::tempdir().expect("tmp");
    let registry = tmp.path().join("browsers.json");
    std::fs::write(
        &registry,
        r#"{"version":1,"updated_at":"1970-01-01T00:00:00.000000000Z","platform":"test","entries":[]}"#,
    )
    .expect("write registry");
    let json = r#"{"type":"message","title":"T","message":"Hi","buttons":"ok"}"#;
    let output = run_wyvern(
        wyvern()
            .env("WYVERN_BROWSERS_FILE", &registry)
            .env("WYVERN_SAFARI_PATH", tmp.path().join("no-safari-bin"))
            .env("WYVERN_CHROME_PATH", tmp.path().join("no-chrome-bin"))
            .env("WYVERN_EDGE_PATH", tmp.path().join("no-edge-bin"))
            .env("WYVERN_FIREFOX_PATH", tmp.path().join("no-firefox-bin"))
            .args([json, "--viewer", "safari", "--bind", "127.0.0.1:0"]),
    );
    assert_eq!(
        output.status.code(),
        Some(wyvern_schema::ErrorCode::HostViewerError.exit_code())
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    let value = stderr_json(&stderr);
    assert_eq!(value["code"], "HOST_VIEWER_ERROR");
    assert!(
        value["message"].as_str().unwrap_or("").contains("safari")
            || value["cause"].as_str().unwrap_or("").contains("Safari"),
        "stderr={stderr}"
    );
}

#[test]
fn cli_message_wyvern_viewer_env_none_publishes_url() {
    let json = r#"{"type":"message","title":"T","message":"Hi","buttons":"ok"}"#;
    let mut child = wyvern()
        .env("WYVERN_VIEWER", "none")
        .args([json, "--bind", "127.0.0.1:0"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn");

    let stderr = child.stderr.take().expect("stderr");
    let (url_tx, url_rx) = std::sync::mpsc::channel::<String>();
    let stderr_handle = thread::spawn(move || {
        let mut collected = String::new();
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            let line = line.expect("stderr line");
            collected.push_str(&line);
            collected.push('\n');
            if let Some(url) = line.strip_prefix("WYVERN_DIALOG_URL=") {
                let _ = url_tx.send(url.to_string());
            }
        }
        collected
    });

    let dialog_url = url_rx
        .recv_timeout(Duration::from_secs(15))
        .expect("timed out waiting for WYVERN_DIALOG_URL on stderr");
    let base = dialog_base_url(&dialog_url);

    let client = wait_for_http(&format!("{base}/api/dialog"));
    let _ = client
        .post(format!("{base}/api/result"))
        .json(&serde_json::json!({"button":"ok"}))
        .send()
        .expect("post");

    let output = wait_child_with_stderr(child, stderr_handle);
    assert_child_ok(&output);
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), r#"{"button":"ok"}"#);
}

/// Strip path suffix from published dialog URL (e.g. `http://127.0.0.1:PORT/message/`).
fn dialog_base_url(dialog_url: &str) -> String {
    dialog_url
        .trim_end_matches('/')
        .trim_end_matches("/chrome")
        .trim_end_matches("/message")
        .trim_end_matches("/input")
        .trim_end_matches("/markdown")
        .trim_end_matches("/question")
        .to_string()
}

fn wait_for_http(url: &str) -> reqwest::blocking::Client {
    let client = reqwest::blocking::Client::new();
    let start = std::time::Instant::now();
    let budget = Duration::from_secs(15);
    loop {
        if client
            .get(url)
            .send()
            .map(|r| r.status().is_success())
            .unwrap_or(false)
        {
            return client;
        }
        if start.elapsed() > budget {
            panic!("timed out waiting for {url}");
        }
        thread::sleep(Duration::from_millis(25));
    }
}
