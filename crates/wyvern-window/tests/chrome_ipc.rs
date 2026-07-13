//! Integration: ChromeApp `window_close` IPC → `{"button":"dismissed"}`.
//!
//! Custom harness (`harness = false`) so `main` runs on the process main
//! thread — required by winit on macOS (and safest cross-platform).
//!
//! Separate from `chrome_minimize_ipc` so macOS gets a fresh EventLoop
//! (one per process).

mod support;

fn main() {
    println!("running 1 test");
    let result = std::panic::catch_unwind(|| {
        chrome_ipc_window_close_dismissed();
    });
    match result {
        Ok(()) => {
            println!("test chrome_ipc_window_close_dismissed ... ok");
            println!();
            println!("test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out");
        }
        Err(_) => {
            println!("test chrome_ipc_window_close_dismissed ... FAILED");
            println!();
            println!(
                "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out"
            );
            std::process::exit(101);
        }
    }
}

fn chrome_ipc_window_close_dismissed() {
    let result = support::open_chrome_with_injected_ipc(r#"{"kind":"window_close"}"#)
        .expect("chrome window_close should complete with Ok(CommandResult)");
    support::assert_dismissed(&result);

    let wire = serde_json::to_string(&result).expect("serialize");
    assert_eq!(wire, r#"{"button":"dismissed"}"#);
}
