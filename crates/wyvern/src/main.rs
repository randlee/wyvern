//! Wyvern CLI — thin wrapper around load → pipeline (validate → run → emit).

use std::io::{self, IsTerminal, Write};
use std::process::ExitCode;

use wyvern::observability;
use wyvern::{
    emit_fatal_internal, emit_io_error, emit_parse_error, load_command_input, run_from_loaded,
    usage_message, EmitError, LoadError, PipelineError,
};
use wyvern_schema::SerializeError;

fn main() -> ExitCode {
    if let Err(err) = observability::init() {
        eprintln!("wyvern: {err}");
    }
    observability::log_process_start();

    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.len() == 1 && (args[0] == "--version" || args[0] == "-V") {
        println!("wyvern {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }

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
        Err(err) => return emit_load_stage_failure(&err),
    };

    match run_from_loaded(value) {
        Ok(stdout) => {
            let mut out = io::stdout().lock();
            let _ = writeln!(out, "{stdout}");
            ExitCode::SUCCESS
        }
        Err(PipelineError::Stage { stderr, exit_code }) => {
            eprintln!("{stderr}");
            ExitCode::from(u8::try_from(exit_code).unwrap_or(1))
        }
        Err(PipelineError::Emit(e)) => emit_fatal_internal(&e),
    }
}

fn emit_load_stage_failure(err: &LoadError) -> ExitCode {
    debug_assert!(matches!(
        err,
        LoadError::Parse { .. } | LoadError::Io { .. }
    ));
    let emit_result = match err {
        LoadError::Parse { .. } => emit_parse_error(err),
        LoadError::Io { .. } => emit_io_error(err),
        LoadError::Usage { .. } => {
            emit_fatal_internal(&EmitError::Serialize(SerializeError {
                message: "miswired Usage in emit_load_stage_failure".into(),
            }));
        }
    };
    match emit_result {
        Ok(stderr) => {
            eprintln!("{stderr}");
            ExitCode::from(u8::try_from(err.exit_code()).unwrap_or(1))
        }
        Err(e) => emit_fatal_internal(&e),
    }
}
