//! Integration: inject `button_pressed` IPC and assert MessageResult wire shape.
//!
//! Custom harness (`harness = false`) so `main` runs on the process main
//! thread — required by winit on macOS (and safest cross-platform).

mod support;

fn main() {
    println!("running 1 test");
    let result = std::panic::catch_unwind(|| {
        message_ipc_button_pressed_ok();
    });
    match result {
        Ok(()) => {
            println!("test message_ipc_button_pressed_ok ... ok");
            println!();
            println!("test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out");
        }
        Err(_) => {
            println!("test message_ipc_button_pressed_ok ... FAILED");
            println!();
            println!(
                "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out"
            );
            std::process::exit(101);
        }
    }
}

fn message_ipc_button_pressed_ok() {
    let result =
        support::open_message_with_injected_ipc(r#"{"kind":"button_pressed","label":"ok"}"#)
            .expect("message window should complete with Ok(CommandResult)");
    support::assert_message_button(&result, "ok");

    let wire = serde_json::to_string(&result).expect("serialize");
    assert_eq!(wire, r#"{"button":"ok"}"#);
}
