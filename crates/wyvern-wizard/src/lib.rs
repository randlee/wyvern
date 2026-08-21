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

mod history;
/// Static lint analysis for wizard packages (`wyvern wizard lint`).
///
/// All public items are pure — no file I/O.
pub mod lint;
mod session;

#[doc(inline)]
pub use lint::{
    extract_local_script_srcs, extract_next_hops, has_back_button, has_cancel_button,
    has_nav_region, has_next_button, has_wizard_chrome_script, is_terminal_page, lint_page,
    LintCode, LintFinding, PageHop, PageInfo, PageRole,
};
#[doc(inline)]
pub use session::{NavigateOutcome, WizardError, WizardSession, WizardSnapshot};
