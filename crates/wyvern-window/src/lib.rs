//! Wyvern native webview window, IPC bridge, and HTML chrome frame.

mod error;
mod window;

pub use error::RunError;

#[doc(hidden)]
pub use window::open_blank_window;
