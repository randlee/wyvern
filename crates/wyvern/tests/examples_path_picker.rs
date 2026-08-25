//! Bundled `share/wyvern/examples/path-picker/` CLI smoke (Phase I i.1).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root")
}

fn example_dir() -> PathBuf {
    workspace_root().join("share/wyvern/examples/path-picker")
}

fn packaged_example_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("share/wyvern/examples/path-picker")
}

fn example_rel_paths() -> &'static [&'static str] {
    &[
        "README.md",
        "wizard.json",
        "app.js",
        "pages/sources.html",
        "pages/review.html",
    ]
}

fn wyvern() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_wyvern"));
    cmd.env_remove("WYVERN_LOG");
    cmd.env_remove("WYVERN_SHARE");
    cmd.env_remove("WYVERN_VIEWER_BIN");
    cmd.env("WYVERN_VIEWER", "none");
    cmd
}

fn wait_for_url_file(path: &Path) -> String {
    let start = std::time::Instant::now();
    loop {
        if let Ok(url) = std::fs::read_to_string(path) {
            let url = url.trim().to_string();
            if !url.is_empty() {
                return url;
            }
        }
        if start.elapsed() > Duration::from_secs(20) {
            panic!("timed out waiting for dialog URL file {}", path.display());
        }
        thread::sleep(Duration::from_millis(20));
    }
}

fn http_client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .pool_max_idle_per_host(0)
        .build()
        .expect("http client")
}

fn wait_for_wizard_state(client: &reqwest::blocking::Client, base: &str) -> serde_json::Value {
    let url = format!("{base}/api/wizard/state");
    let start = std::time::Instant::now();
    loop {
        match client.get(&url).send() {
            Ok(resp) if resp.status() == reqwest::StatusCode::OK => {
                return resp.json().expect("state json");
            }
            Ok(_) | Err(_) => {
                if start.elapsed() > Duration::from_secs(15) {
                    panic!("timed out waiting for GET /api/wizard/state at {url}");
                }
                thread::sleep(Duration::from_millis(20));
            }
        }
    }
}

#[test]
fn examples_path_picker_tree_exists() {
    let root = example_dir();
    for rel in example_rel_paths() {
        let path = root.join(rel);
        assert!(path.is_file(), "missing example file: {}", path.display());
    }
}

#[test]
fn examples_path_picker_share_parity() {
    for rel in example_rel_paths() {
        let workspace = std::fs::read_to_string(example_dir().join(rel)).expect("workspace");
        let packaged = std::fs::read_to_string(packaged_example_dir().join(rel)).expect("packaged");
        assert_eq!(
            workspace, packaged,
            "crates/wyvern/share must track share/wyvern/examples/path-picker/{rel}"
        );
    }
}

#[test]
fn examples_path_picker_readme_documents_gui_and_headless() {
    let readme = std::fs::read_to_string(example_dir().join("README.md")).expect("README");
    assert!(
        readme.contains("{wyvern_share}/examples/path-picker/wizard.json"),
        "README must document GUI with {{wyvern_share}} path: {readme}"
    );
    assert!(
        readme.contains("--ui-root {wyvern_share}/examples/path-picker"),
        "README must document {{wyvern_share}} ui-root: {readme}"
    );
    assert!(
        readme.contains("WYVERN_MOCK_PICKER_PATH") && readme.contains("WYVERN_VIEWER=none"),
        "README must document headless mock commands: {readme}"
    );
}

#[test]
fn examples_path_picker_cli_headless_finish_includes_paths() {
    let root = example_dir();
    let wizard = root.join("wizard.json");
    let fixture = tempfile::NamedTempFile::new().expect("picker fixture");
    std::fs::write(fixture.path(), b"fixture").expect("write fixture");
    let tmp = tempfile::tempdir().expect("tempdir");
    let url_file = tmp.path().join("dialog-url");

    let child = wyvern()
        .args([
            wizard.to_str().expect("utf8"),
            "--ui-root",
            root.to_str().expect("utf8"),
            "--viewer",
            "none",
        ])
        .env("WYVERN_DIALOG_URL_FILE", &url_file)
        .env("WYVERN_MOCK_PICKER_PATH", fixture.path())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn wyvern");

    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_url
        .split_once("/wizard/")
        .map(|(b, _)| b.trim_end_matches('/').to_string())
        .expect("wizard URL");
    let client = http_client();
    let _ = wait_for_wizard_state(&client, &base);

    let file_picker: serde_json::Value = client
        .post(format!("{base}/api/picker/file"))
        .json(&serde_json::json!({ "multiple": true }))
        .send()
        .expect("POST file picker")
        .error_for_status()
        .expect("file picker status")
        .json()
        .expect("file picker json");
    assert_eq!(file_picker["ok"], true);
    let file_paths = file_picker["paths"].clone();
    assert!(
        file_paths.as_array().is_some_and(|a| !a.is_empty()),
        "expected mock file paths"
    );

    let folder_picker: serde_json::Value = client
        .post(format!("{base}/api/picker/folder"))
        .json(&serde_json::json!({}))
        .send()
        .expect("POST folder picker")
        .error_for_status()
        .expect("folder picker status")
        .json()
        .expect("folder picker json");
    assert_eq!(folder_picker["ok"], true);
    let folder_paths = folder_picker["paths"].clone();
    assert!(
        folder_paths.as_array().is_some_and(|a| !a.is_empty()),
        "expected mock folder paths"
    );

    let finish_data = serde_json::json!({
        "file_paths": file_paths,
        "folder_paths": folder_paths
    });
    let sources_page = serde_json::json!({
        "id": "sources",
        "title": "Choose paths",
        "html": "pages/sources.html"
    });
    let review_page = serde_json::json!({
        "id": "review",
        "title": "Review paths",
        "html": "pages/review.html"
    });

    let nav = client
        .post(format!("{base}/api/wizard/navigate"))
        .json(&serde_json::json!({
            "action": "next",
            "data": finish_data,
            "next": review_page
        }))
        .send()
        .expect("navigate");
    assert_eq!(nav.status(), reqwest::StatusCode::OK, "navigate failed");

    let stack = serde_json::json!([
        { "page": sources_page, "data": finish_data },
        { "page": review_page, "data": finish_data }
    ]);
    let finish = client
        .post(format!("{base}/api/wizard/finish"))
        .json(&serde_json::json!({
            "button": "finish",
            "data": finish_data,
            "stack": stack
        }))
        .send()
        .expect("finish");
    assert_eq!(
        finish.status(),
        reqwest::StatusCode::OK,
        "finish failed: {}",
        finish.text().unwrap_or_default()
    );

    let output = child.wait_with_output().expect("wait wyvern");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(0),
        "expected exit 0; stdout={stdout} stderr={stderr}"
    );
    let value: serde_json::Value = serde_json::from_str(stdout.trim()).expect("stdout JSON");
    assert_eq!(value["button"], "finish");
    assert_eq!(value["data"]["file_paths"], file_paths);
    assert_eq!(value["data"]["folder_paths"], folder_paths);
    assert!(
        value["stack"].as_array().is_some_and(|s| s.len() == 2),
        "expected 2-page stack: {value}"
    );
}
