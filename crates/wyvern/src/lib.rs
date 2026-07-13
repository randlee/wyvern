//! Wyvern CLI library — load → validate → run → emit pipeline.
//!
//! `main.rs` is a thin binary wrapper around [`pipeline::run_from_loaded`].

mod error;
mod input;
pub mod observability;
mod pipeline;

#[doc(inline)]
pub use error::{
    emit_load_error, emit_run_error, emit_stdout, emit_validation_error, handle_run_failure,
    LoadError,
};
#[doc(inline)]
pub use input::{load_command_input, usage_message};
#[doc(inline)]
pub use pipeline::run_from_loaded;
