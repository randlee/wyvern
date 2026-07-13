//! File-mode picker IPC with `WYVERN_MOCK_PICKER_PATH` (ADR-0014).
//!
//! Separate harness so macOS gets a fresh EventLoop (one per process).

mod support;

fn main() {
    println!("running 1 test");
    let result = std::panic::catch_unwind(|| {
        input_file_picker_mock_ok();
    });
    match result {
        Ok(()) => {
            println!("test input_file_picker_mock_ok ... ok");
            println!();
            println!("test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out");
        }
        Err(_) => {
            println!("test input_file_picker_mock_ok ... FAILED");
            println!();
            println!(
                "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out"
            );
            std::process::exit(101);
        }
    }
}

fn input_file_picker_mock_ok() {
    let result = support::open_file_picker_with_mock("/tmp/wyvern-fixture.txt", false)
        .expect("file picker should complete");
    support::assert_input_result(&result, "ok", Some("/tmp/wyvern-fixture.txt"));
    let wire = serde_json::to_string(&result).expect("serialize");
    assert_eq!(wire, r#"{"button":"ok","input":"/tmp/wyvern-fixture.txt"}"#);
}
