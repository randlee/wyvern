//! HTML input dialog template and rendering.

mod picker;
mod render;

pub use picker::{pick_file, pick_folder};
pub use render::{
    estimate_input_window_size, parse_input_page_ipc, render_input_html, InputPageIpc,
    InputRenderInput,
};
