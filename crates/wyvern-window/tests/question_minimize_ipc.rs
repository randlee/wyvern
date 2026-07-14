//! Integration: modal QuestionApp `window_minimize` is a no-op (no early dismiss).

mod support;

fn main() {
    println!("running 1 test");
    let result = std::panic::catch_unwind(|| {
        question_window_minimize_noop_then_auto_dismiss();
    });
    match result {
        Ok(()) => {
            println!("test question_window_minimize_noop_then_auto_dismiss ... ok");
            println!();
            println!("test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out");
        }
        Err(_) => {
            println!("test question_window_minimize_noop_then_auto_dismiss ... FAILED");
            println!();
            println!(
                "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out"
            );
            std::process::exit(101);
        }
    }
}

fn question_window_minimize_noop_then_auto_dismiss() {
    let result = support::open_question_inject_then_auto_dismiss(r#"{"kind":"window_minimize"}"#)
        .expect("question window_minimize + auto-dismiss should complete");
    support::assert_question_dismissed(&result);
}
