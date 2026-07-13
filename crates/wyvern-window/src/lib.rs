//! Wyvern native webview window, IPC bridge, and HTML chrome frame.
//!
//! The sole public entry point is [`run`]. Size constants describe chrome
//! (Phase A fixed) and modal dialog (Phase B auto-size) geometry.

mod chrome;
mod error;
mod markdown;
mod message;
mod run;
mod window;

/// Default chrome open width in logical pixels (REQ Phase A fixed size).
pub const CHROME_DEFAULT_WIDTH: f64 = 480.0;
/// Default chrome open height in logical pixels.
pub const CHROME_DEFAULT_HEIGHT: f64 = 360.0;
/// Maximum chrome width in logical pixels.
pub const CHROME_MAX_WIDTH: f64 = 800.0;
/// Maximum chrome height in logical pixels.
pub const CHROME_MAX_HEIGHT: f64 = 600.0;

/// Modal dialog minimum width (REQ-0041 / Phase B).
pub const DIALOG_MIN_WIDTH: f64 = 320.0;
/// Modal dialog minimum height (REQ-0041 / Phase B).
pub const DIALOG_MIN_HEIGHT: f64 = 200.0;
/// Modal dialog maximum width (REQ-0041 / Phase B).
pub const DIALOG_MAX_WIDTH: f64 = 800.0;
/// Modal dialog maximum height (REQ-0041 / Phase B).
pub const DIALOG_MAX_HEIGHT: f64 = 600.0;

#[doc(inline)]
pub use error::RunError;
#[doc(inline)]
pub use markdown::markdown_to_html;
#[doc(inline)]
pub use run::run;
