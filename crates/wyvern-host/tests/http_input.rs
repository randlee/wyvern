//! L1 HTTP tests for input dialog + picker routes.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use serial_test::serial;
use wyvern_host::{run, HostOptions, ViewerMode};
use wyvern_schema::{
    ButtonsPreset, ChromeTitle, Command, CommandResult, InputMode, InputResult, InputValue,
};

fn workspace_ui_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../ui")
}

fn unique_path(prefix: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "{prefix}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ))
}

fn host_options(url_file: PathBuf) -> HostOptions {
    HostOptions {
        bind: SocketAddr::from(([127, 0, 0, 1], 0)),
        ui_root: workspace_ui_root(),
        viewer: ViewerMode::None,
        dialog_url_env: true,
        dialog_url_file: Some(url_file),
    }
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
    }
}

fn wait_for_url_file(path: &std::path::Path) -> String {
    for _ in 0..200 {
        if let Ok(url) = std::fs::read_to_string(path) {
            let url = url.trim().to_string();
            if url.starts_with("http://") {
                return url;
            }
        }
        thread::sleep(Duration::from_millis(25));
    }
    panic!("timed out waiting for dialog URL file {}", path.display());
}

#[test]
fn run_input_text_posts_ok_via_http() {
    let url_file = unique_path("wyvern-host-input-url");
    let options = host_options(url_file.clone());
    let handle = thread::spawn(move || run(text_input_command(), options));

    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_url.trim_end_matches("/input/").trim_end_matches('/');

    let client = reqwest::blocking::Client::new();
    let dialog: serde_json::Value = client
        .get(format!("{base}/api/dialog"))
        .send()
        .expect("GET dialog")
        .error_for_status()
        .expect("dialog status")
        .json()
        .expect("dialog json");
    assert_eq!(dialog["type"], "input");
    assert_eq!(dialog["mode"], "text");
    assert_eq!(dialog["title"], "Name");

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
#[serial]
fn picker_file_returns_picker_response_json() {
    let fixture = unique_path("wyvern-picker-fixture");
    std::fs::write(&fixture, b"fixture").expect("write fixture");

    // SAFETY: #[serial] owns the mock env for this test's duration.
    unsafe {
        std::env::set_var("WYVERN_MOCK_PICKER_PATH", &fixture);
    }

    let url_file = unique_path("wyvern-host-picker-url");
    let options = host_options(url_file.clone());
    let handle = thread::spawn(move || run(file_input_command(), options));

    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_url.trim_end_matches("/input/").trim_end_matches('/');

    let client = reqwest::blocking::Client::new();
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

    unsafe {
        std::env::remove_var("WYVERN_MOCK_PICKER_PATH");
    }
    let _ = std::fs::remove_file(&url_file);
    let _ = std::fs::remove_file(&fixture);
}

#[test]
#[serial]
fn picker_file_cancelled_returns_cancelled_json() {
    // SAFETY: #[serial] owns the mock env for this test's duration.
    unsafe {
        std::env::set_var("WYVERN_MOCK_PICKER_PATH", "");
    }

    let url_file = unique_path("wyvern-host-picker-cancel");
    let options = host_options(url_file.clone());
    let handle = thread::spawn(move || run(file_input_command(), options));

    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_url.trim_end_matches("/input/").trim_end_matches('/');

    let client = reqwest::blocking::Client::new();
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

    unsafe {
        std::env::remove_var("WYVERN_MOCK_PICKER_PATH");
    }
    let _ = std::fs::remove_file(&url_file);
}

#[test]
#[serial]
fn picker_folder_returns_picker_response_json() {
    let fixture = unique_path("wyvern-picker-folder-fixture");
    std::fs::create_dir_all(&fixture).expect("create fixture dir");

    // SAFETY: #[serial] owns the mock env for this test's duration.
    unsafe {
        std::env::set_var("WYVERN_MOCK_PICKER_PATH", &fixture);
    }

    let url_file = unique_path("wyvern-host-picker-folder-url");
    let options = host_options(url_file.clone());
    let handle = thread::spawn(move || run(folder_input_command(), options));

    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_url.trim_end_matches("/input/").trim_end_matches('/');

    let client = reqwest::blocking::Client::new();
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

    unsafe {
        std::env::remove_var("WYVERN_MOCK_PICKER_PATH");
    }
    let _ = std::fs::remove_file(&url_file);
    let _ = std::fs::remove_dir_all(&fixture);
}

#[test]
#[serial]
fn picker_folder_cancelled_returns_cancelled_json() {
    // SAFETY: #[serial] owns the mock env for this test's duration.
    unsafe {
        std::env::set_var("WYVERN_MOCK_PICKER_PATH", "");
    }

    let url_file = unique_path("wyvern-host-picker-folder-cancel");
    let options = host_options(url_file.clone());
    let handle = thread::spawn(move || run(folder_input_command(), options));

    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_url.trim_end_matches("/input/").trim_end_matches('/');

    let client = reqwest::blocking::Client::new();
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

    unsafe {
        std::env::remove_var("WYVERN_MOCK_PICKER_PATH");
    }
    let _ = std::fs::remove_file(&url_file);
}
