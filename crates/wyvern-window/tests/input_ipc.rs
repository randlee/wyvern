//! Integration: inject `input_submitted` IPC and assert InputResult wire shape.
//!
//! Custom harness (`harness = false`) so `main` runs on the process main
//! thread — required by winit on macOS (and safest cross-platform).
//!
//! Only one EventLoop per process is possible on macOS; cancel omit-input is
//! covered by `input::render` unit tests. File/folder picker paths live in
//! separate harness binaries (`input_file_ipc`, `input_folder_ipc`).

mod support;

fn main() {
    println!("running 1 test");
    let result = std::panic::catch_unwind(|| {
        input_ipc_submitted_ok_with_value();
    });
    match result {
        Ok(()) => {
            println!("test input_ipc_submitted_ok_with_value ... ok");
            println!();
            println!("test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out");
        }
        Err(_) => {
            println!("test input_ipc_submitted_ok_with_value ... FAILED");
            println!();
            println!(
                "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out"
            );
            std::process::exit(101);
        }
    }
}

fn input_ipc_submitted_ok_with_value() {
    let result = support::open_input_with_injected_ipc(
        r#"{"kind":"input_submitted","button":"ok","value":"Ada Lovelace"}"#,
    )
    .expect("input window should complete with Ok(CommandResult)");
    support::assert_input_result(&result, "ok", Some("Ada Lovelace"));

    let wire = serde_json::to_string(&result).expect("serialize");
    assert_eq!(wire, r#"{"button":"ok","input":"Ada Lovelace"}"#);
}
