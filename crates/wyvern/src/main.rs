//! Wyvern CLI — load → validate (window execution arrives in a.5).

mod error;
mod input;

use std::io::{self, IsTerminal};
use std::process::ExitCode;

use error::{emit_load_error, emit_validation_error, LoadError};
use input::{load_command_input, usage_message};
use wyvern_schema::validate;

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

    match validate(&value) {
        Ok(_command) => {
            // a.4 interim: valid chrome exits 0 without opening a window.
            // a.5 wires wyvern_window::run and stdout emission.
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("{}", emit_validation_error(&err));
            ExitCode::from(1)
        }
    }
}
