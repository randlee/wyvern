//! Wyvern embedded viewer — navigate to a dialog URL; POST dismissed on OS close.

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

use std::process::ExitCode;

mod platform;
mod run;

fn main() -> ExitCode {
    match run::run_from_env_and_args(std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("wyvern-viewer: {err}");
            ExitCode::from(1)
        }
    }
}
