//! Wyvern CLI library — load → validate → run → emit pipeline.
//!
//! `main.rs` is a thin binary wrapper around [`pipeline::run_from_loaded`].

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

mod browsers_cmd;
mod cli_args;
mod embedded_ui;
mod error;
/// CLI extension registry, match, preexec, and expand (ADR-0022).
pub mod extensions;
mod input;
mod observability;
mod pipeline;
mod viewer_spawn;

#[doc(inline)]
pub use browsers_cmd::{run_browsers_command, BrowsersError};
#[doc(inline)]
pub use cli_args::{apply_host_overrides, parse_cli_args, usage_message, CliArgs};
#[doc(inline)]
pub use error::{
    emit_extension_error, emit_fatal_internal, emit_host_error, emit_io_error, emit_parse_error,
    emit_stdout, emit_usage_error, emit_usage_message, emit_validation_error, EmitError, LoadError,
};
#[doc(inline)]
pub use extensions::{ExtensionError, ExtensionMatch, ExtensionRegistry};
#[doc(inline)]
pub use input::load_command_input;
#[doc(inline)]
pub use pipeline::{run_from_loaded, PipelineError};
#[doc(inline)]
pub use viewer_spawn::{resolve_viewer_bin, spawn_embedded_viewer, ViewerSpawnError};
