//! Integration: OS close / auto-dismiss yields REQ-0068 question shape.
//!
//! Separate binary because winit allows one EventLoop per process.

mod support;

fn main() {
    println!("running 1 test");
    let result = std::panic::catch_unwind(|| {
        question_ipc_dismissed_includes_button();
    });
    match result {
        Ok(()) => {
            println!("test question_ipc_dismissed_includes_button ... ok");
            println!();
            println!("test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out");
        }
        Err(_) => {
            println!("test question_ipc_dismissed_includes_button ... FAILED");
            println!();
            println!(
                "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out"
            );
            std::process::exit(101);
        }
    }
}

/// Force close stdout contract per question-contract-examples.md (REQ-0068).
fn question_ipc_dismissed_includes_button() {
    let result = support::open_question_auto_dismiss()
        .expect("question window should complete with Ok(CommandResult)");
    support::assert_question_dismissed(&result);

    let wire: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&result).expect("serialize")).expect("json");
    assert_eq!(wire["button"], "dismissed");
    assert_eq!(wire["answers"], serde_json::json!({}));
    assert_eq!(wire["response"], "");
    assert_eq!(wire["questions"][0]["question"], "Output format?");
    assert!(wire["questions"][0]["options"][0].get("preview").is_some());
}
