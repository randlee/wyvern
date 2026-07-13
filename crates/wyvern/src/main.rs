//! Wyvern CLI — load command input from argv or stdin.

mod error;
mod input;

use std::io::{self, IsTerminal};
use std::process::ExitCode;

use error::{emit_load_error, LoadError};
use input::{load_command_input, usage_message};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // No positional args on a TTY: print usage instead of blocking on stdin.
    if args.is_empty() && io::stdin().is_terminal() {
        eprintln!("{}", usage_message());
        return ExitCode::from(1);
    }

    match load_command_input(&args, io::stdin()) {
        Ok(_value) => {
            // Validation and window execution arrive in later sprints.
            ExitCode::SUCCESS
        }
        Err(LoadError::Usage { message }) => {
            eprintln!("{message}");
            ExitCode::from(1)
        }
        Err(err) => {
            eprintln!("{}", emit_load_error(&err));
            ExitCode::from(1)
        }
    }
}
