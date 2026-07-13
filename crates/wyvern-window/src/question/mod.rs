//! HTML question-card dialog template, rendering, and event-loop handler.

mod handler;
mod render;
mod sanitize;

pub(crate) use handler::run_question;
pub use render::{
    estimate_question_window_size, parse_question_page_ipc, render_question_html, QuestionPageIpc,
    QuestionRenderInput,
};
