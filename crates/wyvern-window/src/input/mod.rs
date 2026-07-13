//! HTML input dialog template and rendering.

mod render;

pub use render::{
    estimate_input_window_size, parse_input_page_ipc, render_input_html, InputPageIpc,
    InputRenderInput,
};
