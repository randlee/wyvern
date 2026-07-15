//! Library surface for `wyvern-viewer` unit/integration tests.
//!
//! The binary (`main.rs`) remains the user-facing entrypoint; this crate root
//! exposes viewport wire helpers used by golden tests (d.6).

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

pub mod viewport;

#[doc(inline)]
pub use viewport::{ViewportBounds, FALLBACK_VIEWPORT};
