//! HTML chrome shell template and rendering.

mod ipc;
mod platform;
mod render;

pub(crate) use ipc::{parse_chrome_ipc, ChromeIpc};
pub(crate) use platform::CommandKind;
pub use platform::{platform_chrome_for, PlatformChrome};
pub use render::render_chrome_html;
pub(crate) use render::{title_bar_style, window_controls_block};
