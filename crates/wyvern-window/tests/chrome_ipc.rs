//! Integration: ChromeApp HTML window chrome IPC (window_close / window_minimize).
//!
//! Custom harness (`harness = false`) so `main` runs on the process main
//! thread — required by winit on macOS (and safest cross-platform).

mod support;

fn main() {
    println!("running 2 tests");
    let mut failed = 0usize;

    let result = std::panic::catch_unwind(|| {
        chrome_ipc_window_close_dismissed();
    });
    match result {
        Ok(()) => println!("test chrome_ipc_window_close_dismissed ... ok"),
        Err(_) => {
            println!("test chrome_ipc_window_close_dismissed ... FAILED");
            failed += 1;
        }
    }

    let result = std::panic::catch_unwind(|| {
        chrome_ipc_window_minimize_no_stdout_then_auto_dismiss();
    });
    match result {
        Ok(()) => {
            println!("test chrome_ipc_window_minimize_no_stdout_then_auto_dismiss ... ok")
        }
        Err(_) => {
            println!("test chrome_ipc_window_minimize_no_stdout_then_auto_dismiss ... FAILED");
            failed += 1;
        }
    }

    println!();
    if failed == 0 {
        println!("test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out");
    } else {
        println!(
            "test result: FAILED. {} passed; {failed} failed; 0 ignored; 0 measured; 0 filtered out",
            2 - failed
        );
        std::process::exit(101);
    }
}

fn chrome_ipc_window_close_dismissed() {
    let result = support::open_chrome_with_injected_ipc(r#"{"kind":"window_close"}"#)
        .expect("chrome window_close should complete with Ok(CommandResult)");
    support::assert_dismissed(&result);

    let wire = serde_json::to_string(&result).expect("serialize");
    assert_eq!(wire, r#"{"button":"dismissed"}"#);
}

fn chrome_ipc_window_minimize_no_stdout_then_auto_dismiss() {
    // window_minimize must not complete the run by itself; auto-dismiss finishes
    // the harness with the normal dismissed chrome result (no early/malformed path).
    let result = support::open_chrome_inject_then_auto_dismiss(r#"{"kind":"window_minimize"}"#)
        .expect("chrome window_minimize + auto-dismiss should complete");
    support::assert_dismissed(&result);
}
