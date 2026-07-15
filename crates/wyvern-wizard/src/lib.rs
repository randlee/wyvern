//! Wyvern wizard navigation state machine.
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
mod session;

#[doc(inline)]
pub use session::{WizardError, WizardSession, WizardSnapshot};
