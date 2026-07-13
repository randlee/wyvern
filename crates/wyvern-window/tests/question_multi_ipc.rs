//! Multi-select question IPC: comma-joined answers per question-contract-examples.md.
//!
//! Separate binary from `question_ipc` because winit allows one EventLoop per process.

mod support;

fn main() {
    println!("running 1 test");
    let result = std::panic::catch_unwind(|| {
        question_ipc_multi_select_comma_joined();
    });
    match result {
        Ok(()) => {
            println!("test question_ipc_multi_select_comma_joined ... ok");
            println!();
            println!("test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out");
        }
        Err(_) => {
            println!("test question_ipc_multi_select_comma_joined ... FAILED");
            println!();
            println!(
                "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out"
            );
            std::process::exit(101);
        }
    }
}

/// Multi-select stdout contract per question-contract-examples.md (REQ-0062).
fn question_ipc_multi_select_comma_joined() {
    let result = support::open_multi_select_question_with_injected_ipc(
        r#"{"kind":"question_submitted","answers":{"Pick tools":"JSON, Plain"}}"#,
    )
    .expect("multi-select question window should complete with Ok(CommandResult)");
    support::assert_question_submitted(&result, "Pick tools", "JSON, Plain");

    let wire: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&result).expect("serialize")).expect("json");
    assert!(wire.get("button").is_none());
    assert_eq!(wire["answers"]["Pick tools"], "JSON, Plain");
    assert_eq!(wire["response"], "");
    assert_eq!(wire["questions"][0]["question"], "Pick tools");
    assert_eq!(wire["questions"][0]["multiSelect"], true);
}
