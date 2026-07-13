//! Integration: ChromeApp `window_minimize` does not complete stdout alone.
//!
//! Custom harness (`harness = false`) so `main` runs on the process main
//! thread — required by winit on macOS (and safest cross-platform).
//!
//! Separate from `chrome_ipc` so macOS gets a fresh EventLoop (one per process).

mod support;

fn main() {
    println!("running 1 test");
    let result = std::panic::catch_unwind(|| {
        chrome_ipc_window_minimize_no_stdout_then_auto_dismiss();
    });
    match result {
        Ok(()) => {
            println!("test chrome_ipc_window_minimize_no_stdout_then_auto_dismiss ... ok");
            println!();
            println!("test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out");
        }
        Err(_) => {
            println!("test chrome_ipc_window_minimize_no_stdout_then_auto_dismiss ... FAILED");
            println!();
            println!(
                "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out"
            );
            std::process::exit(101);
        }
    }
}

fn chrome_ipc_window_minimize_no_stdout_then_auto_dismiss() {
    // window_minimize must not complete the run by itself; auto-dismiss finishes
    // the harness with the normal dismissed chrome result (no early/malformed path).
    let result = support::open_chrome_inject_then_auto_dismiss(r#"{"kind":"window_minimize"}"#)
        .expect("chrome window_minimize + auto-dismiss should complete");
    support::assert_dismissed(&result);
}
