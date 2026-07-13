//! Wyvern CLI — thin wrapper around load → pipeline (validate → run → emit).

use std::io::{self, IsTerminal, Write};
use std::process::ExitCode;

use wyvern::{emit_load_error, load_command_input, run_from_loaded, usage_message, LoadError};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // No positional args on a TTY: print usage instead of blocking on stdin.
    if args.is_empty() && io::stdin().is_terminal() {
        eprintln!("{}", usage_message());
        return ExitCode::from(1);
    }

    let value = match load_command_input(&args, io::stdin()) {
        Ok(value) => value,
        Err(LoadError::Usage { message }) => {
            eprintln!("{message}");
            return ExitCode::from(1);
        }
        Err(err) => {
            eprintln!("{}", emit_load_error(&err));
            return ExitCode::from(1);
        }
    };

    match run_from_loaded(value) {
        Ok(stdout) => {
            let mut out = io::stdout().lock();
            let _ = writeln!(out, "{stdout}");
            ExitCode::SUCCESS
        }
        Err((stderr_json, code)) => {
            eprintln!("{stderr_json}");
            ExitCode::from(u8::try_from(code).unwrap_or(1))
        }
    }
}
