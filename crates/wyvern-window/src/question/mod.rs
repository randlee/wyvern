//! HTML question-card dialog template and rendering.

mod render;

pub use render::{
    estimate_question_window_size, parse_question_page_ipc, render_question_html, QuestionPageIpc,
    QuestionRenderInput,
};
