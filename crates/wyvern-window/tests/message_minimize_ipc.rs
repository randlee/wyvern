//! Integration: modal MessageApp `window_minimize` is a no-op (no early dismiss).
//!
//! Custom harness (`harness = false`) so `main` runs on the process main
//! thread — required by winit on macOS (and safest cross-platform).
//!
//! Pattern mirrors `chrome_minimize_ipc`: inject minimize, then auto-dismiss
//! finishes the run. If minimize incorrectly dismissed (or fell through to
//! malformed fail-safe), the harness would still complete — but without
//! auto-dismiss scheduling after a true no-op the run would hang. Completing
//! via auto-dismiss proves minimize did not end the run early by itself.

mod support;

fn main() {
    println!("running 1 test");
    let result = std::panic::catch_unwind(|| {
        message_window_minimize_noop_then_auto_dismiss();
    });
    match result {
        Ok(()) => {
            println!("test message_window_minimize_noop_then_auto_dismiss ... ok");
            println!();
            println!("test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out");
        }
        Err(_) => {
            println!("test message_window_minimize_noop_then_auto_dismiss ... FAILED");
            println!();
            println!(
                "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out"
            );
            std::process::exit(101);
        }
    }
}

fn message_window_minimize_noop_then_auto_dismiss() {
    // window_minimize must not complete the run by itself; auto-dismiss finishes
    // with the normal dismissed message result (no early/malformed-only path).
    let result = support::open_message_inject_then_auto_dismiss(r#"{"kind":"window_minimize"}"#)
        .expect("message window_minimize + auto-dismiss should complete");
    support::assert_dismissed(&result);
}
