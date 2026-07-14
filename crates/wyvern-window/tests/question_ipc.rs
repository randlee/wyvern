//! Integration: inject `question_submitted` IPC and assert QuestionResult wire shape.
//!
//! Custom harness (`harness = false`) so `main` runs on the process main
//! thread — required by winit on macOS (and safest cross-platform).

mod support;

fn main() {
    println!("running 1 test");
    let result = std::panic::catch_unwind(|| {
        question_ipc_submitted_omits_button();
    });
    match result {
        Ok(()) => {
            println!("test question_ipc_submitted_omits_button ... ok");
            println!();
            println!("test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out");
        }
        Err(_) => {
            println!("test question_ipc_submitted_omits_button ... FAILED");
            println!();
            println!(
                "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out"
            );
            std::process::exit(101);
        }
    }
}

fn question_ipc_submitted_omits_button() {
    let result = support::open_question_with_injected_ipc(
        r#"{"kind":"question_submitted","answers":{"Output format?":"JSON"}}"#,
    )
    .expect("question window should complete with Ok(CommandResult)");
    support::assert_question_submitted(&result, "Output format?", "JSON");

    let wire: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&result).expect("serialize")).expect("json");
    assert!(wire.get("button").is_none());
    assert_eq!(wire["answers"]["Output format?"], "JSON");
    assert_eq!(wire["response"], "");
    assert_eq!(wire["questions"][0]["question"], "Output format?");
    assert_eq!(wire["questions"][0]["multiSelect"], false);
}
