//! L1 HTTP tests for question dialog — preview_html + result shapes.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use wyvern_host::{run, HostOptions, ViewerMode};
use wyvern_schema::{
    validate, Command, CommandResult, QuestionCard, QuestionOption, QuestionResult,
};

fn workspace_ui_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../ui")
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

fn single_select_command() -> Command {
    Command::Question {
        questions: vec![QuestionCard {
            question: "Output format?".into(),
            header: "Format".into(),
            options: vec![
                QuestionOption {
                    label: "JSON".into(),
                    description: "Structured".into(),
                    preview: Some(r#"<pre>{"ok":true}</pre>"#.into()),
                },
                QuestionOption {
                    label: "Plain".into(),
                    description: "Text only".into(),
                    preview: None,
                },
            ],
            multi_select: false,
        }],
        questions_raw: vec![serde_json::json!({
            "question": "Output format?",
            "header": "Format",
            "options": [
                {
                    "label": "JSON",
                    "description": "Structured",
                    "preview": "<pre>{\"ok\":true}</pre>"
                },
                { "label": "Plain", "description": "Text only" }
            ],
            "multiSelect": false
        })],
    }
}

fn wait_for_url_file(path: &Path) -> String {
    let start = std::time::Instant::now();
    loop {
        if let Ok(url) = std::fs::read_to_string(path) {
            let url = url.trim().to_string();
            if url.starts_with("http://") {
                return url;
            }
        }
        if start.elapsed() > Duration::from_secs(15) {
            panic!("timed out waiting for dialog URL file: {}", path.display());
        }
        thread::sleep(Duration::from_millis(20));
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

fn dialog_base(dialog_url: &str) -> String {
    dialog_url
        .trim_end_matches("/question/")
        .trim_end_matches('/')
        .to_string()
}

#[test]
fn run_question_posts_submit_via_http() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let url_file = tmp.path().join("dialog-url");
    let options = host_options(url_file.clone());
    let command = single_select_command();
    let expected_raw = match &command {
        Command::Question { questions_raw, .. } => questions_raw.clone(),
        _ => unreachable!(),
    };
    let handle = thread::spawn(move || run(command, options));

    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_base(&dialog_url);

    let client = reqwest::blocking::Client::new();
    let dialog = wait_for_dialog_ready(&client, &base);
    assert_eq!(dialog["type"], "question");
    assert_eq!(dialog["title"], "Question");
    assert_eq!(dialog["questions"][0]["question"], "Output format?");
    assert_eq!(dialog["questions"][0]["multiSelect"], false);
    assert!(dialog.get("buttons").is_none());

    let preview_html = dialog["questions"][0]["options"][0]["preview_html"]
        .as_str()
        .expect("preview_html");
    assert!(
        preview_html.contains("<pre>") && preview_html.contains("ok"),
        "preview_html={preview_html}"
    );
    assert!(
        dialog["questions"][0]["options"][1]
            .get("preview_html")
            .is_none(),
        "plain option must not have preview_html"
    );

    let page = client
        .get(&dialog_url)
        .send()
        .expect("GET page")
        .error_for_status()
        .expect("page status");
    let html = page.text().expect("html");
    assert!(
        html.contains("question-cards") && html.contains("wyvern-api.js"),
        "expected question shell HTML: {html}"
    );

    let ack: serde_json::Value = client
        .post(format!("{base}/api/result"))
        .json(&serde_json::json!({
            "questions": expected_raw,
            "answers": { "Output format?": "JSON" },
            "response": ""
        }))
        .send()
        .expect("POST result")
        .error_for_status()
        .expect("result status")
        .json()
        .expect("ack json");
    assert_eq!(ack["ok"], true);

    let result = handle.join().expect("host thread").expect("run ok");
    let mut answers = HashMap::new();
    answers.insert("Output format?".into(), "JSON".into());
    assert_eq!(
        result,
        CommandResult::Question(QuestionResult::submitted(expected_raw, answers))
    );
}

#[test]
fn dialog_preview_html_strips_script_tags() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let url_file = tmp.path().join("dialog-url");
    let options = host_options(url_file.clone());
    let command = Command::Question {
        questions: vec![QuestionCard {
            question: "Safe?".into(),
            header: "Safe".into(),
            options: vec![
                QuestionOption {
                    label: "A".into(),
                    description: "a".into(),
                    preview: Some(
                        "Hi <script>alert(1)</script>\n\n<img src=x onerror=alert(2)>".into(),
                    ),
                },
                QuestionOption {
                    label: "B".into(),
                    description: "b".into(),
                    preview: None,
                },
            ],
            multi_select: false,
        }],
        questions_raw: vec![serde_json::json!({
            "question": "Safe?",
            "header": "Safe",
            "options": [
                {
                    "label": "A",
                    "description": "a",
                    "preview": "Hi <script>alert(1)</script>"
                },
                { "label": "B", "description": "b" }
            ],
            "multiSelect": false
        })],
    };
    let handle = thread::spawn(move || run(command, options));

    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_base(&dialog_url);

    let client = reqwest::blocking::Client::new();
    let dialog = wait_for_dialog_ready(&client, &base);

    let preview_html = dialog["questions"][0]["options"][0]["preview_html"]
        .as_str()
        .expect("preview_html");
    let lower = preview_html.to_ascii_lowercase();
    assert!(
        !lower.contains("<script") && !lower.contains("alert(1)"),
        "preview_html={preview_html}"
    );
    assert!(!lower.contains("onerror"), "preview_html={preview_html}");
    assert!(preview_html.contains("Hi"), "preview_html={preview_html}");

    let _ = client
        .post(format!("{base}/api/result"))
        .json(&serde_json::json!({
            "button": "dismissed",
            "questions": [],
            "answers": {},
            "response": ""
        }))
        .send()
        .expect("POST result")
        .error_for_status();
    let _ = handle.join().expect("host thread").expect("run ok");
}

#[test]
fn run_question_dismiss_extended_shape() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let url_file = tmp.path().join("dialog-url");
    let options = host_options(url_file.clone());
    let command = single_select_command();
    let expected_raw = match &command {
        Command::Question { questions_raw, .. } => questions_raw.clone(),
        _ => unreachable!(),
    };
    let handle = thread::spawn(move || run(command, options));

    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_base(&dialog_url);

    let client = reqwest::blocking::Client::new();
    let _ = wait_for_dialog_ready(&client, &base);
    let ack: serde_json::Value = client
        .post(format!("{base}/api/result"))
        .json(&serde_json::json!({
            "button": "dismissed",
            "questions": expected_raw,
            "answers": {},
            "response": ""
        }))
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
        CommandResult::Question(QuestionResult::dismissed(expected_raw))
    );
}

#[test]
fn empty_answers_without_button_becomes_dismiss() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let url_file = tmp.path().join("dialog-url");
    let options = host_options(url_file.clone());
    let command = single_select_command();
    let expected_raw = match &command {
        Command::Question { questions_raw, .. } => questions_raw.clone(),
        _ => unreachable!(),
    };
    let handle = thread::spawn(move || run(command, options));

    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_base(&dialog_url);

    let client = reqwest::blocking::Client::new();
    let _ = wait_for_dialog_ready(&client, &base);
    let _ = client
        .post(format!("{base}/api/result"))
        .json(&serde_json::json!({
            "questions": expected_raw,
            "answers": {},
            "response": ""
        }))
        .send()
        .expect("POST result")
        .error_for_status();

    let result = handle.join().expect("host thread").expect("run ok");
    assert_eq!(
        result,
        CommandResult::Question(QuestionResult::dismissed(expected_raw))
    );
}

#[test]
fn result_invalid_question_includes_cause_recovery_docs() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let url_file = tmp.path().join("dialog-url");
    let options = host_options(url_file.clone());
    let handle = thread::spawn(move || run(single_select_command(), options));

    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_base(&dialog_url);

    let client = reqwest::blocking::Client::new();
    let _ = wait_for_dialog_ready(&client, &base);
    let response = client
        .post(format!("{base}/api/result"))
        .json(&serde_json::json!({}))
        .send()
        .expect("POST result");
    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
    let body: serde_json::Value = response.json().expect("error json");
    assert_eq!(body["error"], "bad_request");
    assert!(body["message"]
        .as_str()
        .is_some_and(|m| m.contains("questions")));
    assert!(body["cause"].as_str().is_some_and(|s| !s.is_empty()));
    assert!(body["recovery"].as_array().is_some_and(|a| !a.is_empty()));
    assert!(body["docs"]
        .as_str()
        .is_some_and(|s| s.contains("http-post-schema")));

    let _ = client
        .post(format!("{base}/api/result"))
        .json(&serde_json::json!({
            "button": "dismissed",
            "questions": [],
            "answers": {},
            "response": ""
        }))
        .send()
        .expect("POST result")
        .error_for_status();
    let _ = handle.join().expect("host thread").expect("run ok");
}

#[test]
fn validate_then_run_multi_select_shape() {
    let input = serde_json::json!({
        "type": "question",
        "questions": [{
            "question": "Pick tools",
            "header": "Tools",
            "options": [
                { "label": "JSON", "description": "A" },
                { "label": "Plain", "description": "B" }
            ],
            "multiSelect": true
        }]
    });
    let command = validate(&input).expect("validate");
    let expected_raw = match &command {
        Command::Question { questions_raw, .. } => questions_raw.clone(),
        other => panic!("expected Question, got {other:?}"),
    };

    let tmp = tempfile::tempdir().expect("temp dir");
    let url_file = tmp.path().join("dialog-url");
    let options = host_options(url_file.clone());
    let handle = thread::spawn(move || run(command, options));

    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_base(&dialog_url);

    let client = reqwest::blocking::Client::new();
    let dialog = wait_for_dialog_ready(&client, &base);
    assert_eq!(dialog["questions"][0]["multiSelect"], true);

    let _ = client
        .post(format!("{base}/api/result"))
        .json(&serde_json::json!({
            "questions": expected_raw,
            "answers": { "Pick tools": "JSON, Plain" },
            "response": ""
        }))
        .send()
        .expect("POST result")
        .error_for_status();

    let result = handle.join().expect("host thread").expect("run ok");
    let mut answers = HashMap::new();
    answers.insert("Pick tools".into(), "JSON, Plain".into());
    assert_eq!(
        result,
        CommandResult::Question(QuestionResult::submitted(expected_raw, answers))
    );
}

#[test]
fn result_rejects_unknown_answer_keys() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let url_file = tmp.path().join("dialog-url");
    let options = host_options(url_file.clone());
    let command = single_select_command();
    let expected_raw = match &command {
        Command::Question { questions_raw, .. } => questions_raw.clone(),
        _ => unreachable!(),
    };
    let handle = thread::spawn(move || run(command, options));

    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_base(&dialog_url);

    let client = reqwest::blocking::Client::new();
    let _ = wait_for_dialog_ready(&client, &base);
    let response = client
        .post(format!("{base}/api/result"))
        .json(&serde_json::json!({
            "questions": expected_raw,
            "answers": { "Not a real prompt": "JSON" },
            "response": ""
        }))
        .send()
        .expect("POST result");
    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
    let body: serde_json::Value = response.json().expect("error json");
    assert_eq!(body["error"], "bad_request");
    assert!(body["message"]
        .as_str()
        .is_some_and(|m| m.contains("answers key") && m.contains("Not a real prompt")));

    let _ = client
        .post(format!("{base}/api/result"))
        .json(&serde_json::json!({
            "button": "dismissed",
            "questions": expected_raw,
            "answers": {},
            "response": ""
        }))
        .send()
        .expect("POST result")
        .error_for_status();
    let _ = handle.join().expect("host thread").expect("run ok");
}

#[test]
fn dialog_rejects_oversized_question_preview() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let url_file = tmp.path().join("dialog-url");
    let options = host_options(url_file.clone());
    let oversized = "x".repeat(wyvern_schema::MARKDOWN_CONTENT_MAX_BYTES + 1);
    let command = Command::Question {
        questions: vec![QuestionCard {
            question: "Q?".into(),
            header: "Hdr".into(),
            options: vec![
                QuestionOption {
                    label: "A".into(),
                    description: "a".into(),
                    preview: Some(oversized),
                },
                QuestionOption {
                    label: "B".into(),
                    description: "b".into(),
                    preview: None,
                },
            ],
            multi_select: false,
        }],
        questions_raw: vec![serde_json::json!({
            "question": "Q?",
            "header": "Hdr",
            "options": [
                { "label": "A", "description": "a", "preview": "x" },
                { "label": "B", "description": "b" }
            ],
            "multiSelect": false
        })],
    };
    let handle = thread::spawn(move || run(command, options));

    let dialog_url = wait_for_url_file(&url_file);
    let base = dialog_base(&dialog_url);

    let client = reqwest::blocking::Client::new();
    let start = std::time::Instant::now();
    let response = loop {
        match client.get(format!("{base}/api/dialog")).send() {
            Ok(resp) => break resp,
            Err(_) if start.elapsed() < Duration::from_secs(15) => {
                thread::sleep(Duration::from_millis(20));
            }
            Err(err) => panic!("GET dialog failed: {err}"),
        }
    };
    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
    let body: serde_json::Value = response.json().expect("error json");
    assert_eq!(body["error"], "bad_request");
    assert!(body["message"]
        .as_str()
        .is_some_and(|m| m.contains("preview") && m.contains("exceeds maximum")));
    assert!(body["docs"]
        .as_str()
        .is_some_and(|s| s.contains("c13-host-question")));

    let _ = client
        .post(format!("{base}/api/result"))
        .json(&serde_json::json!({
            "button": "dismissed",
            "questions": [],
            "answers": {},
            "response": ""
        }))
        .send();
    let _ = handle.join();
}
