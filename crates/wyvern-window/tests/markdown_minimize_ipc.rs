//! Integration: modal MarkdownApp `window_minimize` is a no-op (no early dismiss).

mod support;

fn main() {
    println!("running 1 test");
    let result = std::panic::catch_unwind(|| {
        markdown_window_minimize_noop_then_auto_dismiss();
    });
    match result {
        Ok(()) => {
            println!("test markdown_window_minimize_noop_then_auto_dismiss ... ok");
            println!();
            println!("test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out");
        }
        Err(_) => {
            println!("test markdown_window_minimize_noop_then_auto_dismiss ... FAILED");
            println!();
            println!(
                "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out"
            );
            std::process::exit(101);
        }
    }
}

fn markdown_window_minimize_noop_then_auto_dismiss() {
    let result = support::open_markdown_inject_then_auto_dismiss(r#"{"kind":"window_minimize"}"#)
        .expect("markdown window_minimize + auto-dismiss should complete");
    support::assert_dismissed(&result);
}
