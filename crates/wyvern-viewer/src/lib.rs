//! Library surface for `wyvern-viewer` unit/integration tests.
//!
//! The binary (`main.rs`) remains the user-facing entrypoint; this crate root
//! exposes viewport wire helpers (d.6) and dismiss stack builders (d.8).

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

pub mod dismiss;
pub mod viewport;

#[doc(inline)]
pub use dismiss::{is_wizard_dialog_url, post_dismissed, wizard_dismiss_finish_body};
#[doc(inline)]
pub use viewport::{HiddenUntilResize, ViewportBounds, FALLBACK_VIEWPORT};
