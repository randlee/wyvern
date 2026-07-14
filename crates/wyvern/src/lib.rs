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

mod error;
mod input;
pub mod observability;
mod pipeline;

#[doc(inline)]
pub use error::{
    emit_fatal_internal, emit_io_error, emit_parse_error, emit_run_error, emit_stdout,
    emit_validation_error, EmitError, LoadError,
};
#[doc(inline)]
pub use input::{load_command_input, usage_message};
#[doc(inline)]
pub use pipeline::{run_from_loaded, PipelineError};
