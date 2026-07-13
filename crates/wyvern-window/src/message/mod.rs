//! HTML message dialog template and rendering.

pub(crate) mod media;
mod render;

pub use render::{
    estimate_message_window_size, parse_page_ipc, render_message_html, MessageRenderInput, PageIpc,
};
