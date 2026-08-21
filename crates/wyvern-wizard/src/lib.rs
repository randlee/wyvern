//! Wyvern wizard navigation state machine and static lint analysis.
//!
//! Pure stack + cursor logic (ADR-0005 / ADR-0007). No I/O.

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

/// Dataflow lint rules WIZARD-LINT-005–008 (`config.dataflow`).
pub mod dataflow;
mod history;
/// Static lint analysis for wizard packages (`wyvern wizard lint`).
///
/// All public items are pure — no file I/O.
pub mod lint;
mod session;

#[doc(inline)]
pub use dataflow::{
    add_edge, extract_data_reads, extract_next_wizard_refs, lint_dataflow,
    merge_html_dataflow_overlay, parse_dataflow_from_json, parse_dataflow_value, DataflowLintInput,
    DataflowSpec, GraphPage, NextWizardRef, PageDataflow, WizardPageGraph,
};
#[doc(inline)]
pub use lint::{
    extract_local_script_srcs, extract_next_hops, has_back_button, has_cancel_button,
    has_nav_region, has_next_button, has_wizard_chrome_script, is_terminal_page, lint_page,
    LintCode, LintFinding, PageHop, PageInfo, PageRole,
};
#[doc(inline)]
pub use session::{NavigateOutcome, WizardError, WizardSession, WizardSnapshot};
