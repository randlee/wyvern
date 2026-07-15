//! Viewer dismiss routing: wizard finish stack vs blocking `/api/result` (d.8).

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use serde_json::{json, Value};
use wyvern_viewer::{
    is_wizard_dialog_url, post_dismissed, wizard_dismiss_finish_body, WizardStateDto,
};

#[derive(Debug, Default, Clone)]
struct Recorded {
    methods: Vec<String>,
    paths: Vec<String>,
    bodies: Vec<String>,
}

fn spawn_mock_host(state: Value, recorded: Arc<Mutex<Recorded>>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = listener.local_addr().expect("addr");
    thread::spawn(move || {
        // Accept GET state + POST finish (or a single POST /api/result).
        for _ in 0..2 {
            let Ok((mut stream, _)) = listener.accept() else {
                break;
            };
            stream.set_read_timeout(Some(Duration::from_secs(2))).ok();
            let mut buf = [0u8; 8192];
            let n = stream.read(&mut buf).unwrap_or(0);
            let req = String::from_utf8_lossy(&buf[..n]);
            let first = req.lines().next().unwrap_or("");
            let mut parts = first.split_whitespace();
            let method = parts.next().unwrap_or("").to_string();
            let path = parts.next().unwrap_or("").to_string();
            let body = req
                .split("\r\n\r\n")
                .nth(1)
                .or_else(|| req.split("\n\n").nth(1))
                .unwrap_or("")
                .to_string();
            {
                let mut rec = recorded.lock().expect("lock");
                rec.methods.push(method.clone());
                rec.paths.push(path.clone());
                rec.bodies.push(body);
            }

            let (status, resp_body) = if method == "GET" && path.starts_with("/api/wizard/state") {
                ("200 OK", serde_json::to_string(&state).expect("state"))
            } else if method == "POST" && path.starts_with("/api/wizard/finish") {
                (
                    "200 OK",
                    r#"{"button":"dismissed","data":{},"stack":[]}"#.to_string(),
                )
            } else if method == "POST" && path.starts_with("/api/result") {
                ("200 OK", r#"{"button":"dismissed"}"#.to_string())
            } else {
                ("404 Not Found", "{}".to_string())
            };
            let response = format!(
                "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{resp_body}",
                resp_body.len()
            );
            let _ = stream.write_all(response.as_bytes());
        }
    });
    format!("http://{addr}")
}

#[test]
fn wizard_dismiss_posts_finish_with_full_visited_stack() {
    let state = json!({
        "type": "wizard",
        "config": {},
        "page": { "id": "b", "title": "b", "html": "pages/b.html" },
        "page_data": { "draft": true },
        "stack": [
            {
                "page": { "id": "a", "title": "a", "html": "pages/a.html" },
                "data": { "a": 1 }
            }
        ]
    });
    let recorded = Arc::new(Mutex::new(Recorded::default()));
    let base = spawn_mock_host(state.clone(), Arc::clone(&recorded));
    let dialog_url = format!("{base}/wizard/pages/b.html");
    assert!(is_wizard_dialog_url(&dialog_url));

    post_dismissed(&dialog_url);
    // Allow the mock thread to accept both requests.
    thread::sleep(Duration::from_millis(100));

    let rec = recorded.lock().expect("lock");
    assert_eq!(rec.methods.len(), 2, "expected GET state + POST finish");
    assert_eq!(rec.methods[0], "GET");
    assert!(rec.paths[0].starts_with("/api/wizard/state"));
    assert_eq!(rec.methods[1], "POST");
    assert!(rec.paths[1].starts_with("/api/wizard/finish"));

    let body: Value = serde_json::from_str(&rec.bodies[1]).expect("finish json");
    let dto: WizardStateDto = serde_json::from_value(state).expect("dto");
    let expected = wizard_dismiss_finish_body(&dto).expect("expected body");
    assert_eq!(body, expected);
    assert_eq!(body["button"], "dismissed");
    assert_eq!(body["stack"].as_array().expect("stack").len(), 2);
    assert_eq!(body["stack"][1]["data"], json!({ "draft": true }));
}

#[test]
fn wizard_dismiss_blocking_posts_api_result_only() {
    let recorded = Arc::new(Mutex::new(Recorded::default()));
    let base = spawn_mock_host(json!({}), Arc::clone(&recorded));
    let dialog_url = format!("{base}/message/");
    assert!(!is_wizard_dialog_url(&dialog_url));

    post_dismissed(&dialog_url);
    thread::sleep(Duration::from_millis(100));

    let rec = recorded.lock().expect("lock");
    assert_eq!(rec.methods.len(), 1);
    assert_eq!(rec.methods[0], "POST");
    assert!(rec.paths[0].starts_with("/api/result"));
    assert_eq!(rec.bodies[0], r#"{"button":"dismissed"}"#);
}
