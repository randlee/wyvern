//! Wyvern CLI — thin wrapper around load → pipeline (validate → run → emit).

#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable,
        clippy::todo,
        clippy::unimplemented
    )
)]

use std::io::{self, IsTerminal, Write};
use std::process::ExitCode;

use wyvern::extensions::{
    build_match_context, build_skill_record, expand_and_validate, format_skill_card,
    match_extension_help, run_extensions_command, ExtensionError, ExtensionRegistry,
    ExtensionsCmdError, PathRequiresProbe,
};
use wyvern::{
    apply_host_overrides, emit_extension_error, emit_fatal_internal, emit_io_error,
    emit_parse_error, emit_usage_error, emit_usage_message, load_command_input, parse_cli_args,
    run_browsers_command, run_from_loaded, usage_message, BrowsersError, LoadError, PipelineError,
};

mod main_observability;

fn main() -> ExitCode {
    if let Err(err) = main_observability::init() {
        main_observability::emit_init_error(&err);
    }
    main_observability::log_process_start();

    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.first().map(String::as_str) == Some("browsers") {
        return match run_browsers_command(&args[1..]) {
            Ok(stdout) => {
                print!("{stdout}");
                ExitCode::SUCCESS
            }
            Err(BrowsersError::Usage { kind, message }) => {
                match emit_usage_error(&LoadError::Usage { kind, message }) {
                    Ok(stderr) => {
                        eprintln!("{stderr}");
                        ExitCode::from(2)
                    }
                    Err(e) => emit_fatal_internal(&e),
                }
            }
            Err(BrowsersError::Stage { stderr, exit_code }) => {
                eprintln!("{stderr}");
                ExitCode::from(u8::try_from(exit_code).unwrap_or(1))
            }
            Err(BrowsersError::Emit(e)) => emit_fatal_internal(&e),
        };
    }

    if args.first().map(String::as_str) == Some("extensions") {
        return match run_extensions_command(&args[1..]) {
            Ok(stdout) => {
                print!("{stdout}");
                ExitCode::SUCCESS
            }
            Err(ExtensionsCmdError::Usage { kind, message }) => {
                match emit_usage_error(&LoadError::Usage { kind, message }) {
                    Ok(stderr) => {
                        eprintln!("{stderr}");
                        ExitCode::from(2)
                    }
                    Err(e) => emit_fatal_internal(&e),
                }
            }
            Err(ExtensionsCmdError::Stage { stderr, exit_code }) => {
                eprintln!("{stderr}");
                ExitCode::from(u8::try_from(exit_code).unwrap_or(1))
            }
            Err(ExtensionsCmdError::Emit(e)) => emit_fatal_internal(&e),
        };
    }

    let mut cli = match parse_cli_args(&args) {
        Ok(cli) => cli,
        Err(err) => return emit_load_stage_failure(&err),
    };

    if cli.positionals.len() == 1
        && (cli.positionals[0] == "--version" || cli.positionals[0] == "-V")
    {
        println!("wyvern {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }

    // No positional args on a TTY: emit structured usage JSON instead of blocking on stdin.
    if cli.positionals.is_empty() && io::stdin().is_terminal() {
        return match emit_usage_message(&usage_message()) {
            Ok(stderr) => {
                eprintln!("{stderr}");
                ExitCode::from(2)
            }
            Err(e) => emit_fatal_internal(&e),
        };
    }

    if cli
        .positionals
        .first()
        .is_some_and(|token| token == "--help" || token == "-h" || token == "help")
    {
        print!("{}", usage_message());
        return ExitCode::SUCCESS;
    }

    let registry = match ExtensionRegistry::load_default() {
        Ok(registry) => registry,
        Err(err) => return emit_extension_stage_failure(&err),
    };

    if let Some(ext) = match_extension_help(&registry, &cli.positionals) {
        print!(
            "{}",
            format_skill_card(&build_skill_record(ext, &PathRequiresProbe))
        );
        return ExitCode::SUCCESS;
    }

    if let Some(matched) = registry.match_argv(&cli.positionals) {
        let ctx = build_match_context(&matched, matched.extension());
        let expanded = match expand_and_validate(matched.extension(), &ctx) {
            Ok(expanded) => expanded,
            Err(err) => return emit_extension_stage_failure(&err),
        };
        apply_host_overrides(&mut cli.host, &expanded.host_overrides);
        let result = run_from_loaded(expanded.command, cli.host);
        // `expanded.temp_guard` drops after host exit (success or stage error).
        drop(expanded.temp_guard);
        return emit_pipeline_result(result);
    }

    let value = match load_command_input(&cli.positionals, io::stdin()) {
        Ok(value) => value,
        Err(err) => return emit_load_stage_failure(&err),
    };

    emit_pipeline_result(run_from_loaded(value, cli.host))
}

fn emit_pipeline_result(result: Result<String, PipelineError>) -> ExitCode {
    match result {
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

fn emit_extension_stage_failure(err: &ExtensionError) -> ExitCode {
    match emit_extension_error(err) {
        Ok(stderr) => {
            eprintln!("{stderr}");
            ExitCode::from(u8::try_from(err.exit_code()).unwrap_or(1))
        }
        Err(e) => emit_fatal_internal(&e),
    }
}

fn emit_load_stage_failure(err: &LoadError) -> ExitCode {
    let emit_result = match err {
        LoadError::Parse { .. } => emit_parse_error(err),
        LoadError::Io { .. } => emit_io_error(err),
        LoadError::Usage { .. } => emit_usage_error(err),
    };
    match emit_result {
        Ok(stderr) => {
            eprintln!("{stderr}");
            ExitCode::from(u8::try_from(err.exit_code()).unwrap_or(1))
        }
        Err(e) => emit_fatal_internal(&e),
    }
}
