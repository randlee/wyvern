//! L1 HTTP tests for input dialog + picker routes.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

use wyvern_host::{
    run, HostOptions, MockPickerConfig, MockPickerSlotEvent, MockPickerSlotLog, ViewerMode,
};
use wyvern_schema::{
    ButtonsPreset, ChromeTitle, Command, CommandResult, InputMode, InputResult, InputValue,
};

fn workspace_ui_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../ui")
}

fn unique_path(prefix: &str) -> PathBuf {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("{prefix}-{}-{n}", std::process::id()))
}

fn host_options(url_file: PathBuf) -> HostOptions {
    HostOptions {
        bind: SocketAddr::from(([127, 0, 0, 1], 0)),
        ui_root: workspace_ui_root(),
        viewer: ViewerMode::None,
        dialog_url_env: true,
        dialog_url_file: Some(url_file),
        allow_non_loopback: false,
        session_timeout: wyvern_host::DEFAULT_SESSION_TIMEOUT,
        mock_picker: None,
    }
}

fn host_options_with_mock(url_file: PathBuf, mock: MockPickerConfig) -> HostOptions {
    let mut options = host_options(url_file);
    options.mock_picker = Some(mock);
    options
}

fn text_input_command() -> Command {
    Command::Input {
        title: ChromeTitle::new("Name"),
        message: "Enter name".into(),
        status: None,
        icon: None,
        markdown: false,
        multiline: false,
        placeholder: Some("hint".into()),
        default: None,
        password: false,
        mode: InputMode::Text,
        filter: None,
        multiple: false,
        start_path: None,
        buttons: ButtonsPreset::OkCancel,

        width: None,
        height: None,
    }
}

fn password_input_command() -> Command {
    Command::Input {
        title: ChromeTitle::new("Secret"),
        message: "Enter password".into(),
        status: None,
        icon: None,
        markdown: false,
        multiline: false,
        placeholder: None,
        default: None,
        password: true,
        mode: InputMode::Text,
        filter: None,
        multiple: false,
        start_path: None,
        buttons: ButtonsPreset::OkCancel,

        width: None,
        height: None,
    }
}

fn file_input_command() -> Command {
    Command::Input {
        title: ChromeTitle::new("File"),
        message: "Pick a file".into(),
        status: None,
        icon: None,
        markdown: false,
        multiline: false,
        placeholder: None,
        default: None,
        password: false,
        mode: InputMode::File,
        filter: Some(vec!["*.txt".into()]),
        multiple: false,
        start_path: None,
        buttons: ButtonsPreset::OkCancel,

        width: None,
        height: None,
    }
}

fn multi_file_input_command() -> Command {
    Command::Input {
        title: ChromeTitle::new("Files"),
        message: "Pick files".into(),
        status: None,
        icon: None,
        markdown: false,
        multiline: false,
        placeholder: None,
        default: None,
        password: false,
        mode: InputMode::File,
        filter: Some(vec!["*.txt".into()]),
        multiple: true,
        start_path: None,
        buttons: ButtonsPreset::OkCancel,

        width: None,
        height: None,
    }
}

fn folder_input_command() -> Command {
    Command::Input {
        title: ChromeTitle::new("Folder"),
        message: "Pick a folder".into(),
        status: None,
        icon: None,
        markdown: false,
        multiline: false,
        placeholder: None,
        default: None,
        password: false,
        mode: InputMode::Folder,
        filter: None,
        multiple: false,
        start_path: None,
        buttons: ButtonsPreset::OkCancel,

        width: None,
        height: None,
    }
}

fn wait_for_url_file(path: &std::path::Path) -> String {
    let start = std::time::Instant::now();
    loop {
        if let Ok(url) = std::fs::read_to_string(path) {
            let url = url.trim().to_string();
            if url.starts_with("http://") {
                return url;
            }
        }
        if start.elapsed() > Duration::from_secs(15) {
            panic!("timed out waiting for dialog URL file {}", path.display());
        }
        thread::sleep(Duration::from_millis(25));
    }
}

/// Poll `GET /api/dialog` until HTTP 200 (URL file alone is not readiness).
fn wait_for_dialog_ready(client: &reqwest::blocking::Client, base: &str) -> serde_json::Value {
    let url = format!("{base}/api/dialog");
    let start = std::time::Instant::now();
    loop {
        match client.get(&url).send() {
            Ok(resp) if resp.status() == reqwest::StatusCode::OK => {
                return resp.json().expect("dialog json");
            }
            Ok(_) | Err(_) => {
                if start.elapsed() > Duration::from_secs(15) {
                    panic!("timed out waiting for GET /api/dialog at {url}");
                }
                thread::sleep(Duration::from_millis(20));
            }
        }
    }
}

/// Poll until the mock picker records its first Enter (slot held).
fn wait_for_picker_enter(slot_log: &MockPickerSlotLog) {
    let start = std::time::Instant::now();
    loop {
        if slot_log.events().contains(&MockPickerSlotEvent::Enter) {
            return;
        }
        if start.elapsed() > Duration::from_secs(5) {
            panic!("timed out waiting for first picker Enter in slot_log");
        }
        thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn run_input_text_posts_ok_via_http() {
    let url_file = unique_path("wyvern-host-input-url");
    let options = host_options(url_file.clone());
    let handle = thread::spawn(move || run(text_input_command(), options));

    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_url.trim_end_matches("/input/").trim_end_matches('/');

    let client = reqwest::blocking::Client::new();
    let dialog = wait_for_dialog_ready(&client, base);
    assert_eq!(dialog["type"], "input");
    assert_eq!(dialog["mode"], "text");
    assert_eq!(dialog["title"], "Name");
    assert_eq!(dialog["password"], false);

    let ack: serde_json::Value = client
        .post(format!("{base}/api/result"))
        .json(&serde_json::json!({"button":"ok","input":"Ada"}))
        .send()
        .expect("POST result")
        .error_for_status()
        .expect("result status")
        .json()
        .expect("ack json");
    assert_eq!(ack["ok"], true);

    let result = handle.join().expect("host thread").expect("run ok");
    assert_eq!(
        result,
        CommandResult::Input(InputResult {
            button: wyvern_schema::ButtonLabel::new("ok"),
            input: Some(InputValue::Text("Ada".into())),
        })
    );

    let _ = std::fs::remove_file(&url_file);
}

#[test]
fn run_input_password_posts_ok_via_http() {
    let url_file = unique_path("wyvern-host-input-password-url");
    let options = host_options(url_file.clone());
    let handle = thread::spawn(move || run(password_input_command(), options));

    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_url.trim_end_matches("/input/").trim_end_matches('/');

    let client = reqwest::blocking::Client::new();
    let dialog = wait_for_dialog_ready(&client, base);
    assert_eq!(dialog["type"], "input");
    assert_eq!(dialog["password"], true);
    assert_eq!(dialog["mode"], "text");

    let secret = "s3cret-value";
    let ack: serde_json::Value = client
        .post(format!("{base}/api/result"))
        .json(&serde_json::json!({"button":"ok","input": secret}))
        .send()
        .expect("POST result")
        .error_for_status()
        .expect("result status")
        .json()
        .expect("ack json");
    assert_eq!(ack["ok"], true);

    let result = handle.join().expect("host thread").expect("run ok");
    assert_eq!(
        result,
        CommandResult::Input(InputResult {
            button: wyvern_schema::ButtonLabel::new("ok"),
            input: Some(InputValue::Text(secret.into())),
        })
    );

    let _ = std::fs::remove_file(&url_file);
}

#[test]
fn picker_file_returns_picker_response_json() {
    let fixture = unique_path("wyvern-picker-fixture");
    std::fs::write(&fixture, b"fixture").expect("write fixture");

    let url_file = unique_path("wyvern-host-picker-url");
    let options = host_options_with_mock(
        url_file.clone(),
        MockPickerConfig::path(fixture.to_string_lossy()),
    );
    let handle = thread::spawn(move || run(file_input_command(), options));

    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_url.trim_end_matches("/input/").trim_end_matches('/');

    let client = reqwest::blocking::Client::new();
    let _ = wait_for_dialog_ready(&client, base);
    let picker: serde_json::Value = client
        .post(format!("{base}/api/picker/file"))
        .json(&serde_json::json!({}))
        .send()
        .expect("POST picker")
        .error_for_status()
        .expect("picker status")
        .json()
        .expect("picker json");
    assert_eq!(picker["ok"], true);
    assert!(
        picker["paths"].as_array().is_some_and(|a| !a.is_empty()),
        "expected paths in {picker}"
    );
    assert!(picker.get("cancelled").is_none() || picker["cancelled"].is_null());

    let path = picker["paths"][0]
        .as_str()
        .expect("path string")
        .to_string();
    let ack: serde_json::Value = client
        .post(format!("{base}/api/result"))
        .json(&serde_json::json!({"button":"ok","input": path}))
        .send()
        .expect("POST result")
        .error_for_status()
        .expect("result status")
        .json()
        .expect("ack");
    assert_eq!(ack["ok"], true);

    let result = handle.join().expect("host thread").expect("run ok");
    match result {
        CommandResult::Input(InputResult {
            button,
            input: Some(InputValue::Text(p)),
        }) => {
            assert_eq!(button.as_str(), "ok");
            assert_eq!(PathBuf::from(p), fixture);
        }
        other => panic!("unexpected result: {other:?}"),
    }

    let _ = std::fs::remove_file(&url_file);
    let _ = std::fs::remove_file(&fixture);
}

#[test]
fn picker_file_cancelled_returns_cancelled_json() {
    let url_file = unique_path("wyvern-host-picker-cancel");
    let options = host_options_with_mock(url_file.clone(), MockPickerConfig::cancel());
    let handle = thread::spawn(move || run(file_input_command(), options));

    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_url.trim_end_matches("/input/").trim_end_matches('/');

    let client = reqwest::blocking::Client::new();
    let _ = wait_for_dialog_ready(&client, base);
    let picker: serde_json::Value = client
        .post(format!("{base}/api/picker/file"))
        .json(&serde_json::json!({}))
        .send()
        .expect("POST picker")
        .error_for_status()
        .expect("picker status")
        .json()
        .expect("picker json");
    assert_eq!(picker["ok"], false);
    assert_eq!(picker["cancelled"], true);

    let _ = client
        .post(format!("{base}/api/result"))
        .json(&serde_json::json!({"button":"cancel"}))
        .send();
    let _ = handle.join();

    let _ = std::fs::remove_file(&url_file);
}

#[test]
fn picker_multi_file_returns_paths_array_via_result() {
    let fixture_a = unique_path("wyvern-picker-multi-a");
    let fixture_b = unique_path("wyvern-picker-multi-b");
    std::fs::write(&fixture_a, b"a").expect("write a");
    std::fs::write(&fixture_b, b"b").expect("write b");
    let joined = std::env::join_paths([&fixture_a, &fixture_b]).expect("join paths");

    let url_file = unique_path("wyvern-host-picker-multi-url");
    let options = host_options_with_mock(
        url_file.clone(),
        MockPickerConfig::path(joined.to_string_lossy()),
    );
    let handle = thread::spawn(move || run(multi_file_input_command(), options));

    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_url.trim_end_matches("/input/").trim_end_matches('/');

    let client = reqwest::blocking::Client::new();
    let dialog = wait_for_dialog_ready(&client, base);
    assert_eq!(dialog["multiple"], true);

    let picker: serde_json::Value = client
        .post(format!("{base}/api/picker/file"))
        .json(&serde_json::json!({"multiple": true}))
        .send()
        .expect("POST picker")
        .error_for_status()
        .expect("picker status")
        .json()
        .expect("picker json");
    assert_eq!(picker["ok"], true);
    let paths = picker["paths"].as_array().expect("paths array");
    assert_eq!(paths.len(), 2);

    let ack: serde_json::Value = client
        .post(format!("{base}/api/result"))
        .json(&serde_json::json!({
            "button": "ok",
            "input": paths,
        }))
        .send()
        .expect("POST result")
        .error_for_status()
        .expect("result status")
        .json()
        .expect("ack");
    assert_eq!(ack["ok"], true);

    let result = handle.join().expect("host thread").expect("run ok");
    match result {
        CommandResult::Input(InputResult {
            button,
            input: Some(InputValue::Paths(p)),
        }) => {
            assert_eq!(button.as_str(), "ok");
            assert_eq!(p.len(), 2);
            assert_eq!(PathBuf::from(&p[0]), fixture_a);
            assert_eq!(PathBuf::from(&p[1]), fixture_b);
        }
        other => panic!("unexpected result: {other:?}"),
    }

    let _ = std::fs::remove_file(&url_file);
    let _ = std::fs::remove_file(&fixture_a);
    let _ = std::fs::remove_file(&fixture_b);
}

#[test]
fn picker_folder_returns_picker_response_json() {
    let fixture = unique_path("wyvern-picker-folder-fixture");
    std::fs::create_dir_all(&fixture).expect("create fixture dir");

    let url_file = unique_path("wyvern-host-picker-folder-url");
    let options = host_options_with_mock(
        url_file.clone(),
        MockPickerConfig::path(fixture.to_string_lossy()),
    );
    let handle = thread::spawn(move || run(folder_input_command(), options));

    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_url.trim_end_matches("/input/").trim_end_matches('/');

    let client = reqwest::blocking::Client::new();
    let _ = wait_for_dialog_ready(&client, base);
    let picker: serde_json::Value = client
        .post(format!("{base}/api/picker/folder"))
        .json(&serde_json::json!({}))
        .send()
        .expect("POST picker")
        .error_for_status()
        .expect("picker status")
        .json()
        .expect("picker json");
    assert_eq!(picker["ok"], true);
    assert!(
        picker["paths"].as_array().is_some_and(|a| !a.is_empty()),
        "expected paths in {picker}"
    );
    assert!(picker.get("cancelled").is_none() || picker["cancelled"].is_null());

    let path = picker["paths"][0]
        .as_str()
        .expect("path string")
        .to_string();
    let ack: serde_json::Value = client
        .post(format!("{base}/api/result"))
        .json(&serde_json::json!({"button":"ok","input": path}))
        .send()
        .expect("POST result")
        .error_for_status()
        .expect("result status")
        .json()
        .expect("ack");
    assert_eq!(ack["ok"], true);

    let result = handle.join().expect("host thread").expect("run ok");
    match result {
        CommandResult::Input(InputResult {
            button,
            input: Some(InputValue::Text(p)),
        }) => {
            assert_eq!(button.as_str(), "ok");
            assert_eq!(PathBuf::from(p), fixture);
        }
        other => panic!("unexpected result: {other:?}"),
    }

    let _ = std::fs::remove_file(&url_file);
    let _ = std::fs::remove_dir_all(&fixture);
}

#[test]
fn picker_folder_cancelled_returns_cancelled_json() {
    let url_file = unique_path("wyvern-host-picker-folder-cancel");
    let options = host_options_with_mock(url_file.clone(), MockPickerConfig::cancel());
    let handle = thread::spawn(move || run(folder_input_command(), options));

    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_url.trim_end_matches("/input/").trim_end_matches('/');

    let client = reqwest::blocking::Client::new();
    let _ = wait_for_dialog_ready(&client, base);
    let picker: serde_json::Value = client
        .post(format!("{base}/api/picker/folder"))
        .json(&serde_json::json!({}))
        .send()
        .expect("POST picker")
        .error_for_status()
        .expect("picker status")
        .json()
        .expect("picker json");
    assert_eq!(picker["ok"], false);
    assert_eq!(picker["cancelled"], true);

    let _ = client
        .post(format!("{base}/api/result"))
        .json(&serde_json::json!({"button":"cancel"}))
        .send();
    let _ = handle.join();

    let _ = std::fs::remove_file(&url_file);
}

#[test]
fn picker_rejects_empty_filter_override_with_recovery_fields() {
    let url_file = unique_path("wyvern-host-picker-bad-filter");
    let options = host_options_with_mock(url_file.clone(), MockPickerConfig::cancel());
    let handle = thread::spawn(move || run(file_input_command(), options));

    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_url.trim_end_matches("/input/").trim_end_matches('/');

    let client = reqwest::blocking::Client::new();
    let _ = wait_for_dialog_ready(&client, base);
    let resp = client
        .post(format!("{base}/api/picker/file"))
        .json(&serde_json::json!({"filter": [""]}))
        .send()
        .expect("POST picker");
    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().expect("error json");
    assert_eq!(body["ok"], false);
    assert_eq!(body["error"], "bad_request");
    assert!(body["cause"].as_str().is_some_and(|s| !s.is_empty()));
    assert!(body["recovery"].as_array().is_some_and(|a| !a.is_empty()));
    assert!(body["docs"]
        .as_str()
        .is_some_and(|s| s.contains("http-post-schema")));

    let _ = client
        .post(format!("{base}/api/result"))
        .json(&serde_json::json!({"button":"cancel"}))
        .send();
    let _ = handle.join();
    let _ = std::fs::remove_file(&url_file);
}

#[test]
fn picker_rejects_empty_start_path_override() {
    let url_file = unique_path("wyvern-host-picker-bad-start");
    let options = host_options_with_mock(url_file.clone(), MockPickerConfig::cancel());
    let handle = thread::spawn(move || run(file_input_command(), options));

    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_url.trim_end_matches("/input/").trim_end_matches('/');

    let client = reqwest::blocking::Client::new();
    let _ = wait_for_dialog_ready(&client, base);
    let resp = client
        .post(format!("{base}/api/picker/file"))
        .json(&serde_json::json!({"start_path": ""}))
        .send()
        .expect("POST picker");
    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().expect("error json");
    assert_eq!(body["error"], "bad_request");
    assert!(body.get("cause").is_some());
    assert!(body["recovery"].as_array().is_some_and(|a| !a.is_empty()));

    let _ = client
        .post(format!("{base}/api/result"))
        .json(&serde_json::json!({"button":"cancel"}))
        .send();
    let _ = handle.join();
    let _ = std::fs::remove_file(&url_file);
}

#[test]
fn picker_slot_held_until_blocking_task_finishes() {
    // Slow mock keeps the OwnedSemaphorePermit inside spawn_blocking; a concurrent
    // second POST must wait. Prove serialization via enter/exit slot order, not
    // wall-clock elapsed time.
    let fixture = unique_path("wyvern-picker-hold-fixture");
    std::fs::write(&fixture, b"hold").expect("write fixture");

    let slot_log = MockPickerSlotLog::new();
    let url_file = unique_path("wyvern-host-picker-hold");
    let options = host_options_with_mock(
        url_file.clone(),
        MockPickerConfig::path_with_delay(fixture.to_string_lossy(), Duration::from_millis(200))
            .with_slot_log(slot_log.clone()),
    );
    let handle = thread::spawn(move || run(file_input_command(), options));

    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_url.trim_end_matches("/input/").trim_end_matches('/');
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("client");
    let _ = wait_for_dialog_ready(&client, base);

    let base_a = base.to_string();
    let base_b = base.to_string();
    let t1 = thread::spawn(move || {
        reqwest::blocking::Client::new()
            .post(format!("{base_a}/api/picker/file"))
            .json(&serde_json::json!({}))
            .send()
            .expect("POST picker 1")
            .error_for_status()
            .expect("picker 1 status")
            .json::<serde_json::Value>()
            .expect("picker 1 json")
    });
    // Wait until the first request has entered the mock body (holds the slot).
    wait_for_picker_enter(&slot_log);
    let t2 = thread::spawn(move || {
        reqwest::blocking::Client::new()
            .post(format!("{base_b}/api/picker/file"))
            .json(&serde_json::json!({}))
            .send()
            .expect("POST picker 2")
            .error_for_status()
            .expect("picker 2 status")
            .json::<serde_json::Value>()
            .expect("picker 2 json")
    });

    let p1 = t1.join().expect("t1");
    let p2 = t2.join().expect("t2");
    assert_eq!(p1["ok"], true);
    assert_eq!(p2["ok"], true);
    assert_eq!(
        slot_log.events(),
        [
            MockPickerSlotEvent::Enter,
            MockPickerSlotEvent::Exit,
            MockPickerSlotEvent::Enter,
            MockPickerSlotEvent::Exit,
        ],
        "picker bodies must serialize (no overlapping enter before prior exit)"
    );

    let _ = client
        .post(format!("{base}/api/result"))
        .json(&serde_json::json!({"button":"cancel"}))
        .send();
    let _ = handle.join();
    let _ = std::fs::remove_file(&url_file);
    let _ = std::fs::remove_file(&fixture);
}
