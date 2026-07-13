//! Integration: blank winit+wry window opens and dismisses cleanly.
//!
//! Custom harness (`harness = false`) so `main` runs on the process main
//! thread — required by winit on macOS (and safest cross-platform).

mod support;

fn main() {
    println!("running 1 test");
    let result = std::panic::catch_unwind(|| {
        blank_window_dismisses();
    });
    match result {
        Ok(()) => {
            println!("test blank_window_dismisses ... ok");
            println!();
            println!("test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out");
        }
        Err(_) => {
            println!("test blank_window_dismisses ... FAILED");
            println!();
            println!(
                "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out"
            );
            std::process::exit(101);
        }
    }
}

fn blank_window_dismisses() {
    let result = support::open_blank_window_for_test();
    assert!(
        result.is_ok(),
        "blank window should dismiss with Ok(()), got {result:?}"
    );
}
