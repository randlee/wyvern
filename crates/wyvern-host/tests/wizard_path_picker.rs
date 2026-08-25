//! L1: wizard sessions may call picker routes; input/message/report regression.

mod support;
use support::http::{http_client, wait_for_dialog_ready, wait_for_url_file, wait_for_wizard_state};

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

use wyvern_host::{
    begin, run, DialogHandle, HostOptions, MockPickerConfig, MockPickerSlotEvent,
    MockPickerSlotLog, ViewerMode,
};
use wyvern_schema::{
    validate, ButtonsPreset, ChromeTitle, Command, InputMode, ReportCommand, ReportMode,
    ReportPagePath, ReportTitle,
};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn workspace_ui_root() -> PathBuf {
    workspace_root().join("ui")
}

fn path_picker_ui_root() -> PathBuf {
    workspace_root().join("share/wyvern/examples/path-picker")
}

fn unique_path(prefix: &str) -> PathBuf {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("{prefix}-{}-{n}", std::process::id()))
}

fn load_path_picker_command() -> Command {
    let path = path_picker_ui_root().join("wizard.json");
    let raw = std::fs::read_to_string(&path).expect("read path-picker wizard.json");
    let value: serde_json::Value = serde_json::from_str(&raw).expect("parse wizard.json");
    validate(&value).expect("validate path-picker wizard.json")
}

fn host_options(
    ui_root: PathBuf,
    url_file: PathBuf,
    mock: Option<MockPickerConfig>,
) -> HostOptions {
    HostOptions {
        bind: SocketAddr::from(([127, 0, 0, 1], 0)),
        ui_root,
        shared_ui_root: workspace_ui_root(),
        viewer: ViewerMode::None,
        dialog_url_env: true,
        dialog_url_file: Some(url_file),
        allow_non_loopback: false,
        session_timeout: Duration::from_secs(30),
        mock_picker: mock,
    }
}

fn start_wizard(
    mock: MockPickerConfig,
) -> (DialogHandle, String, PathBuf, reqwest::blocking::Client) {
    let url_file = unique_path("wyvern-wizard-path-picker-url");
    let handle = begin(
        load_path_picker_command(),
        host_options(path_picker_ui_root(), url_file.clone(), Some(mock)),
    )
    .expect("begin wizard");
    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_url
        .split_once("/wizard/")
        .map(|(b, _)| b.trim_end_matches('/').to_string())
        .expect("wizard path");
    let client = http_client();
    let _ = wait_for_wizard_state(&client, &base);
    (handle, base, url_file, client)
}

fn text_input_command() -> Command {
    Command::Input {
        title: ChromeTitle::new("Name"),
        message: "Enter name".into(),
        status: None,
        icon: None,
        markdown: false,
        multiline: false,
        placeholder: None,
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

fn message_command() -> Command {
    Command::Message {
        title: ChromeTitle::new("T"),
        message: "Hi".into(),
        status: None,
        buttons: ButtonsPreset::Ok,
        custom_buttons: None,
        default_button: None,
        level: None,
        icon: None,
        image: None,
        markdown: false,
        width: None,
        height: None,
    }
}

fn report_command() -> Command {
    Command::Report(ReportCommand {
        title: ReportTitle::new("XHTML review"),
        page: ReportPagePath::new("pages/view.xhtml"),
        mode: ReportMode::View,
        panels: None,
        width: None,
        height: None,
    })
}

fn write_report_ui_root() -> PathBuf {
    let root = unique_path("wyvern-path-picker-report-ui");
    std::fs::create_dir_all(root.join("pages")).expect("pages");
    std::fs::write(
        root.join("pages").join("view.xhtml"),
        "<!DOCTYPE html><html><body class=\"report\"><main>ok</main></body></html>",
    )
    .expect("write page");
    root
}

fn post_picker(
    client: &reqwest::blocking::Client,
    url: &str,
    body: serde_json::Value,
) -> reqwest::blocking::Response {
    client.post(url).json(&body).send().expect("POST picker")
}

fn assert_picker_error_envelope(body: &serde_json::Value) {
    assert_eq!(body["ok"], false, "expected ok=false in {body}");
    assert_eq!(
        body["error"], "bad_request",
        "expected bad_request in {body}"
    );
    assert!(
        body["message"].as_str().is_some_and(|s| !s.is_empty()),
        "expected non-empty message in {body}"
    );
    assert!(
        body["cause"].as_str().is_some_and(|s| !s.is_empty()),
        "expected non-empty cause in {body}"
    );
    assert!(
        body["recovery"].as_array().is_some_and(|a| !a.is_empty()),
        "expected non-empty recovery in {body}"
    );
    assert!(
        body["docs"]
            .as_str()
            .is_some_and(|s| s.contains("http-post-schema")),
        "expected docs pointer in {body}"
    );
}

fn assert_recovery_mentions_input_and_wizard(body: &serde_json::Value) {
    let recovery = body["recovery"]
        .as_array()
        .expect("recovery array")
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let recovery_l = recovery.to_ascii_lowercase();
    assert!(
        recovery_l.contains("input") && recovery_l.contains("wizard"),
        "recovery must mention input and wizard per REQ-HOST-0151: {recovery}"
    );
}

fn picker_error_json(resp: reqwest::blocking::Response) -> serde_json::Value {
    assert_eq!(resp.status(), reqwest::StatusCode::BAD_REQUEST);
    resp.json().expect("error json")
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
fn wizard_path_picker_file_returns_mock_paths() {
    let fixture = unique_path("wyvern-wizard-path-picker-file");
    std::fs::write(&fixture, b"fixture").expect("write fixture");

    let (handle, base, url_file, client) =
        start_wizard(MockPickerConfig::path(fixture.to_string_lossy()));
    let picker: serde_json::Value = post_picker(
        &client,
        &format!("{base}/api/picker/file"),
        serde_json::json!({}),
    )
    .error_for_status()
    .expect("picker status")
    .json()
    .expect("picker json");
    assert_eq!(picker["ok"], true);
    let paths = picker["paths"].as_array().expect("paths array");
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0], fixture.to_string_lossy().as_ref());

    let _ = handle.viewer_exited_without_result();
    let _ = std::fs::remove_file(&url_file);
    let _ = std::fs::remove_file(&fixture);
}

#[test]
fn wizard_path_picker_folder_returns_mock_paths() {
    let fixture = unique_path("wyvern-wizard-path-picker-folder");
    std::fs::create_dir_all(&fixture).expect("create fixture dir");

    let (handle, base, url_file, client) =
        start_wizard(MockPickerConfig::path(fixture.to_string_lossy()));
    let picker: serde_json::Value = post_picker(
        &client,
        &format!("{base}/api/picker/folder"),
        serde_json::json!({}),
    )
    .error_for_status()
    .expect("picker status")
    .json()
    .expect("picker json");
    assert_eq!(picker["ok"], true);
    let paths = picker["paths"].as_array().expect("paths array");
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0], fixture.to_string_lossy().as_ref());

    let _ = handle.viewer_exited_without_result();
    let _ = std::fs::remove_file(&url_file);
    let _ = std::fs::remove_dir_all(&fixture);
}

#[test]
fn wizard_path_picker_input_file_merge_still_returns_paths() {
    let fixture = unique_path("wyvern-wizard-path-picker-input");
    std::fs::write(&fixture, b"fixture").expect("write fixture");

    let url_file = unique_path("wyvern-wizard-path-picker-input-url");
    let options = host_options(
        workspace_ui_root(),
        url_file.clone(),
        Some(MockPickerConfig::path(fixture.to_string_lossy())),
    );
    let handle = thread::spawn(move || run(file_input_command(), options));

    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_url.trim_end_matches("/input/").trim_end_matches('/');
    let client = http_client();
    let _ = wait_for_dialog_ready(&client, base);
    let picker: serde_json::Value = post_picker(
        &client,
        &format!("{base}/api/picker/file"),
        serde_json::json!({}),
    )
    .error_for_status()
    .expect("picker status")
    .json()
    .expect("picker json");
    assert_eq!(picker["ok"], true);
    assert!(
        picker["paths"].as_array().is_some_and(|a| !a.is_empty()),
        "expected paths in {picker}"
    );

    let _ = client
        .post(format!("{base}/api/result"))
        .json(&serde_json::json!({"button": "cancel"}))
        .send();
    let _ = handle.join();
    let _ = std::fs::remove_file(&url_file);
    let _ = std::fs::remove_file(&fixture);
}

#[test]
fn wizard_path_picker_message_routes_return_400() {
    let url_file = unique_path("wyvern-wizard-path-picker-message-url");
    let options = host_options(workspace_ui_root(), url_file.clone(), None);
    let handle = thread::spawn(move || run(message_command(), options));

    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_url
        .trim_end_matches("/message/")
        .trim_end_matches('/');
    let client = http_client();
    let _ = wait_for_dialog_ready(&client, base);

    let file = picker_error_json(post_picker(
        &client,
        &format!("{base}/api/picker/file"),
        serde_json::json!({}),
    ));
    assert_picker_error_envelope(&file);
    assert_recovery_mentions_input_and_wizard(&file);
    let folder = picker_error_json(post_picker(
        &client,
        &format!("{base}/api/picker/folder"),
        serde_json::json!({}),
    ));
    assert_picker_error_envelope(&folder);
    assert_recovery_mentions_input_and_wizard(&folder);

    let _ = client
        .post(format!("{base}/api/result"))
        .json(&serde_json::json!({"button": "ok"}))
        .send();
    let _ = handle.join();
    let _ = std::fs::remove_file(&url_file);
}

#[test]
fn wizard_path_picker_report_routes_return_400() {
    let url_file = unique_path("wyvern-wizard-path-picker-report-url");
    let ui_root = write_report_ui_root();
    let handle = begin(
        report_command(),
        host_options(ui_root.clone(), url_file.clone(), None),
    )
    .expect("begin report");
    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_url
        .split_once("/report/")
        .map(|(b, _)| b.trim_end_matches('/').to_string())
        .expect("report path");
    let client = http_client();
    let start = std::time::Instant::now();
    loop {
        match client.get(&dialog_url).send() {
            Ok(resp) if resp.status().is_success() => break,
            Ok(_) | Err(_) => {
                if start.elapsed() > Duration::from_secs(15) {
                    panic!("timed out waiting for report page {dialog_url}");
                }
                thread::sleep(Duration::from_millis(20));
            }
        }
    }

    let file = picker_error_json(post_picker(
        &client,
        &format!("{base}/api/picker/file"),
        serde_json::json!({}),
    ));
    assert_picker_error_envelope(&file);
    assert_recovery_mentions_input_and_wizard(&file);
    let folder = picker_error_json(post_picker(
        &client,
        &format!("{base}/api/picker/folder"),
        serde_json::json!({}),
    ));
    assert_picker_error_envelope(&folder);
    assert_recovery_mentions_input_and_wizard(&folder);

    let _ = handle.viewer_exited_without_result();
    let _ = std::fs::remove_file(&url_file);
    let _ = std::fs::remove_dir_all(&ui_root);
}

#[test]
fn wizard_path_picker_wrong_mode_returns_structured_error() {
    let url_file = unique_path("wyvern-wizard-path-picker-wrong-mode-url");
    let options = host_options(workspace_ui_root(), url_file.clone(), None);
    let handle = thread::spawn(move || run(text_input_command(), options));

    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_url.trim_end_matches("/input/").trim_end_matches('/');
    let client = http_client();
    let _ = wait_for_dialog_ready(&client, base);

    let file = picker_error_json(post_picker(
        &client,
        &format!("{base}/api/picker/file"),
        serde_json::json!({}),
    ));
    assert_picker_error_envelope(&file);
    assert!(
        file["message"]
            .as_str()
            .is_some_and(|s| s.contains("mode 'file'") && s.contains("text")),
        "wrong-mode file picker should name required and actual modes: {file}"
    );

    let folder = picker_error_json(post_picker(
        &client,
        &format!("{base}/api/picker/folder"),
        serde_json::json!({}),
    ));
    assert_picker_error_envelope(&folder);
    assert!(
        folder["message"]
            .as_str()
            .is_some_and(|s| s.contains("mode 'folder'") && s.contains("text")),
        "wrong-mode folder picker should name required and actual modes: {folder}"
    );

    let _ = client
        .post(format!("{base}/api/result"))
        .json(&serde_json::json!({"button": "cancel"}))
        .send();
    let _ = handle.join();
    let _ = std::fs::remove_file(&url_file);
}

#[test]
fn wizard_path_picker_rejects_empty_filter_override_with_recovery_fields() {
    let (handle, base, url_file, client) = start_wizard(MockPickerConfig::cancel());
    let body = picker_error_json(post_picker(
        &client,
        &format!("{base}/api/picker/file"),
        serde_json::json!({"filter": [""]}),
    ));
    assert_picker_error_envelope(&body);
    assert!(
        body["message"]
            .as_str()
            .is_some_and(|s| s.contains("filter[0]")),
        "empty filter should name the index: {body}"
    );

    let _ = handle.viewer_exited_without_result();
    let _ = std::fs::remove_file(&url_file);
}

#[test]
fn wizard_path_picker_rejects_empty_start_path_override() {
    let (handle, base, url_file, client) = start_wizard(MockPickerConfig::cancel());
    let file = picker_error_json(post_picker(
        &client,
        &format!("{base}/api/picker/file"),
        serde_json::json!({"start_path": ""}),
    ));
    assert_picker_error_envelope(&file);
    let folder = picker_error_json(post_picker(
        &client,
        &format!("{base}/api/picker/folder"),
        serde_json::json!({"start_path": ""}),
    ));
    assert_picker_error_envelope(&folder);

    let _ = handle.viewer_exited_without_result();
    let _ = std::fs::remove_file(&url_file);
}

#[test]
fn wizard_path_picker_slot_held_until_blocking_task_finishes() {
    let fixture = unique_path("wyvern-wizard-picker-hold-fixture");
    std::fs::write(&fixture, b"hold").expect("write fixture");

    let slot_log = MockPickerSlotLog::new();
    let (handle, base, url_file, _client) = start_wizard(
        MockPickerConfig::path_with_delay(fixture.to_string_lossy(), Duration::from_millis(200))
            .with_slot_log(slot_log.clone()),
    );

    let base_a = base.clone();
    let base_b = base;
    let t1 = thread::spawn(move || {
        http_client()
            .post(format!("{base_a}/api/picker/file"))
            .json(&serde_json::json!({}))
            .send()
            .expect("POST picker 1")
            .error_for_status()
            .expect("picker 1 status")
            .json::<serde_json::Value>()
            .expect("picker 1 json")
    });
    wait_for_picker_enter(&slot_log);
    let t2 = thread::spawn(move || {
        http_client()
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
        "wizard picker bodies must serialize (no overlapping enter before prior exit)"
    );

    let _ = handle.viewer_exited_without_result();
    let _ = std::fs::remove_file(&url_file);
    let _ = std::fs::remove_file(&fixture);
}
