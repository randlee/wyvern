//! Folder-mode picker IPC with `WYVERN_MOCK_PICKER_PATH` (ADR-0014).

mod support;

fn main() {
    println!("running 1 test");
    let result = std::panic::catch_unwind(|| {
        input_folder_picker_mock_ok();
    });
    match result {
        Ok(()) => {
            println!("test input_folder_picker_mock_ok ... ok");
            println!();
            println!("test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out");
        }
        Err(_) => {
            println!("test input_folder_picker_mock_ok ... FAILED");
            println!();
            println!(
                "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out"
            );
            std::process::exit(101);
        }
    }
}

fn input_folder_picker_mock_ok() {
    let result = support::open_folder_picker_with_mock("/tmp/wyvern-dir")
        .expect("folder picker should complete");
    support::assert_input_result(&result, "ok", Some("/tmp/wyvern-dir"));
}
