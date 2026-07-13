//! Wyvern native webview window, IPC bridge, and HTML chrome frame.
//!
//! The sole public entry point is [`run`]. Size constants describe the Phase A
//! fixed chrome geometry.

mod chrome;
mod error;
mod run;
mod window;

/// Default chrome open width in logical pixels (REQ Phase A fixed size).
pub const CHROME_DEFAULT_WIDTH: f64 = 480.0;
/// Default chrome open height in logical pixels.
pub const CHROME_DEFAULT_HEIGHT: f64 = 360.0;
/// Maximum chrome width in logical pixels until content auto-size (Phase B).
pub const CHROME_MAX_WIDTH: f64 = 800.0;
/// Maximum chrome height in logical pixels until content auto-size (Phase B).
pub const CHROME_MAX_HEIGHT: f64 = 600.0;

#[doc(inline)]
pub use error::RunError;
#[doc(inline)]
pub use run::run;
