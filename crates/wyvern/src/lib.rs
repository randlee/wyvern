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
mod examples_cmd;
mod embedded_ui;
mod error;
/// Bundled example discovery from README frontmatter.
pub mod examples;
/// CLI extension registry, match, preexec, and expand (ADR-0022).
pub mod extensions;
mod input;
mod observability;
mod pipeline;
mod viewer_spawn;
mod wizard_cmd;
/// Wizard workflow hooks and `next_wizard` chain (REQ-0124–0126).
pub mod workflow;

#[doc(inline)]
pub use browsers_cmd::{browsers_usage_message, run_browsers_command, BrowsersError};
#[doc(inline)]
pub use examples_cmd::{
    examples_usage_message, run_examples_command, ExamplesCmdError,
};
#[doc(inline)]
pub use cli_args::{apply_host_overrides, parse_cli_args, usage_message, CliArgs};
#[doc(inline)]
pub use error::{
    emit_extension_error, emit_fatal_internal, emit_host_error, emit_io_error, emit_parse_error,
    emit_stdout, emit_usage_error, emit_usage_message, emit_validation_error,
    emit_wizard_lint_stage_error, emit_workflow_error, BuiltinDomain, EmitError, LoadError,
    UsageErrorKind,
};
#[doc(inline)]
pub use extensions::{
    emit_near_miss, ExtensionError, ExtensionMatch, ExtensionRegistry, NearMissKind,
    PreexecFailureKind,
};
#[doc(inline)]
pub use input::load_command_input;
#[doc(inline)]
pub use observability::set_pipeline_correlation_id;
#[doc(inline)]
pub use pipeline::{run_from_loaded, run_wizard_workflow_loop, PipelineError};
#[doc(inline)]
pub use viewer_spawn::{resolve_viewer_bin, spawn_embedded_viewer, ViewerSpawnError};
#[doc(inline)]
pub use wizard_cmd::{
    run_wizard_command, wizard_usage_message, WizardCmdError, WizardCmdResult, WizardLintStageError,
};
#[doc(inline)]
pub use workflow::{
    check_chain_depth, merge_wizard_config, resolve_next_wizard, Allowlist, NextInvocation,
    WorkflowError, WorkflowRunner, NEXT_WIZARD_MAX_DEPTH, WORKFLOW_SCRIPT_TIMEOUT,
};
