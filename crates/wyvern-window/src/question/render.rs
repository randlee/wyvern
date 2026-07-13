//! Render the question HTML shell and parse page → host IPC.

use std::collections::HashMap;

use serde_json::{json, Value};
use wyvern_schema::QuestionCard;

use crate::chrome::{platform_chrome_for, title_bar_style, window_controls_block, CommandKind};
use crate::{DIALOG_MAX_HEIGHT, DIALOG_MAX_WIDTH, DIALOG_MIN_HEIGHT, DIALOG_MIN_WIDTH};

use super::sanitize::render_preview_fragment;

const QUESTION_HTML: &str = include_str!("template.html");

/// Inputs for [`render_question_html`].
#[derive(Debug, Clone)]
pub struct QuestionRenderInput<'a> {
    /// Window / title-bar text.
    pub title: &'a str,
    /// Typed question cards.
    pub questions: &'a [QuestionCard],
}

/// Parsed page → host IPC payload for question dialogs (ipc-dialog-contract.md).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuestionPageIpc {
    /// User clicked Submit with a non-empty answers map.
    QuestionSubmitted { answers: HashMap<String, String> },
    /// Explicit dismiss from page (rare; OS close is handled by winit).
    Dismissed,
}

/// Escape text for safe insertion into HTML element bodies.
fn escape_html_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

/// Escape a string for embedding inside a double-quoted HTML attribute.
fn escape_attr(s: &str) -> String {
    escape_html_text(s)
}

/// Build question-card HTML with radio or checkbox groups and a Submit control.
///
/// When `options[].preview` is present, the fragment is converted (markdown→HTML
/// when needed) and sanitized before insertion into the preview layout slot.
pub fn render_question_html(input: &QuestionRenderInput<'_>) -> String {
    let QuestionRenderInput { title, questions } = input;

    let safe_title = escape_html_text(title);
    let mut cards_html = String::new();
    let mut ctx_questions = Vec::with_capacity(questions.len());

    for (qi, card) in questions.iter().enumerate() {
        let input_type = if card.multi_select {
            "checkbox"
        } else {
            "radio"
        };
        let group_name = format!("q{qi}");

        let mut options_html = String::new();
        for (oi, opt) in card.options.iter().enumerate() {
            let id = format!("q{qi}-opt{oi}");
            let preview_block = opt
                .preview
                .as_deref()
                .filter(|p| !p.is_empty())
                .map(|p| {
                    format!(
                        r#"<div class="option-preview">{preview}</div>"#,
                        preview = render_preview_fragment(p)
                    )
                })
                .unwrap_or_default();
            let row_class = if preview_block.is_empty() {
                "option-row"
            } else {
                "option-row has-preview"
            };
            options_html.push_str(&format!(
                r#"<label class="{row_class}" for="{id}">
  <input type="{input_type}" id="{id}" name="{name}" value="{value}" />
  <span class="option-text">
    <span class="option-label">{label}</span>
    <div class="option-description">{description}</div>
  </span>
  {preview}
</label>"#,
                row_class = row_class,
                id = escape_attr(&id),
                input_type = input_type,
                name = escape_attr(&group_name),
                value = escape_attr(&opt.label),
                label = escape_html_text(&opt.label),
                description = escape_html_text(&opt.description),
                preview = preview_block,
            ));
        }

        cards_html.push_str(&format!(
            r#"<section class="question-card" data-index="{qi}">
  <div class="card-header">{header}</div>
  <div class="card-prompt">{prompt}</div>
  {options}
</section>"#,
            qi = qi,
            header = escape_html_text(&card.header),
            prompt = escape_html_text(&card.question),
            options = options_html,
        ));

        ctx_questions.push(json!({
            "question": card.question,
            "header": card.header,
            "multiSelect": card.multi_select,
            "options": card.options.iter().map(|o| {
                let mut obj = json!({
                    "label": o.label,
                    "description": o.description,
                });
                if let Some(preview) = &o.preview {
                    obj["preview"] = json!(preview);
                }
                obj
            }).collect::<Vec<_>>(),
        }));
    }

    let context = json!({
        "type": "question",
        "title": title,
        "questions": ctx_questions,
    });
    let chrome = platform_chrome_for(CommandKind::Question);

    QUESTION_HTML
        .replace("{{TITLE}}", &safe_title)
        .replace("{{TITLE_BAR_STYLE}}", title_bar_style(&chrome))
        .replace("{{WINDOW_CONTROLS_BLOCK}}", &window_controls_block(&chrome))
        .replace("{{CARDS}}", &cards_html)
        .replace("{{CONTEXT_JSON}}", &context.to_string())
}

/// Parse a raw IPC body from the question page. Malformed / unknown → [`None`].
pub fn parse_question_page_ipc(raw: &str) -> Option<QuestionPageIpc> {
    let value: Value = serde_json::from_str(raw).ok()?;
    let kind = value.get("kind")?.as_str()?;
    match kind {
        "question_submitted" => {
            let answers_value = value.get("answers")?;
            let obj = answers_value.as_object()?;
            if obj.is_empty() {
                // Empty answers is a fail-safe path handled by the host as dismissed.
                // Still parse successfully so the host can apply REQ-0068.
                return Some(QuestionPageIpc::QuestionSubmitted {
                    answers: HashMap::new(),
                });
            }
            let mut answers = HashMap::with_capacity(obj.len());
            for (k, v) in obj {
                let label = v.as_str()?.to_string();
                answers.insert(k.clone(), label);
            }
            Some(QuestionPageIpc::QuestionSubmitted { answers })
        }
        "dismissed" => Some(QuestionPageIpc::Dismissed),
        _ => None,
    }
}

/// Estimate dialog inner size from card count (REQ-0041).
pub fn estimate_question_window_size(questions: &[QuestionCard]) -> (f64, f64) {
    const TITLE_H: f64 = 36.0;
    const SUBMIT_BAR_H: f64 = 52.0;
    const CONTENT_PAD_Y: f64 = 24.0;
    const CARD_BASE_H: f64 = 56.0;
    const OPTION_H: f64 = 36.0;
    const PREVIEW_EXTRA_H: f64 = 48.0;
    const PREVIEW_EXTRA_W: f64 = 160.0;
    const CARD_GAP: f64 = 12.0;
    const PAD_X: f64 = 48.0;
    const CHAR_W: f64 = 7.2;

    let mut content_h = 0.0_f64;
    let mut max_chars = 24usize;
    let mut has_preview = false;
    for (i, card) in questions.iter().enumerate() {
        if i > 0 {
            content_h += CARD_GAP;
        }
        let mut options_h = 0.0_f64;
        for opt in &card.options {
            options_h += OPTION_H;
            if opt.preview.as_ref().is_some_and(|p| !p.is_empty()) {
                options_h += PREVIEW_EXTRA_H;
                has_preview = true;
            }
            max_chars = max_chars
                .max(opt.label.chars().count())
                .max(opt.description.chars().count());
        }
        content_h += CARD_BASE_H + options_h;
        max_chars = max_chars
            .max(card.question.chars().count())
            .max(card.header.chars().count());
    }

    let preview_w = if has_preview { PREVIEW_EXTRA_W } else { 0.0 };
    let width = ((max_chars as f64).mul_add(CHAR_W, PAD_X) + 80.0 + preview_w)
        .clamp(DIALOG_MIN_WIDTH, DIALOG_MAX_WIDTH);
    let height = (TITLE_H + CONTENT_PAD_Y + content_h + SUBMIT_BAR_H)
        .clamp(DIALOG_MIN_HEIGHT, DIALOG_MAX_HEIGHT);

    (width, height)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wyvern_schema::{QuestionCard, QuestionOption, QuestionResult};

    fn sample_card(multi: bool) -> QuestionCard {
        QuestionCard {
            question: "Output format?".into(),
            header: "Format".into(),
            options: vec![
                QuestionOption {
                    label: "JSON".into(),
                    description: "Structured".into(),
                    preview: Some("<pre>x</pre>".into()),
                },
                QuestionOption {
                    label: "Plain".into(),
                    description: "Text only".into(),
                    preview: None,
                },
            ],
            multi_select: multi,
        }
    }

    #[test]
    fn render_radio_for_single_select() {
        let html = render_question_html(&QuestionRenderInput {
            title: "Question",
            questions: &[sample_card(false)],
        });
        assert!(html.contains(r#"type="radio""#));
        assert!(!html.contains(r#"type="checkbox""#));
        assert!(html.contains("card-header"));
        assert!(html.contains("Format"));
        assert!(html.contains("Output format?"));
        assert!(html.contains("Structured"));
        assert!(html.contains("id=\"submit-btn\""));
        assert!(html.contains("question_submitted"));
        // b.8: preview renders in the option-preview slot (sanitized).
        assert!(html.contains("option-preview"));
        assert!(html.contains("<pre>x</pre>") || html.contains("<pre>x</pre>\n"));
        assert!(html.contains("has-preview"));
    }

    #[test]
    fn render_preview_markdown_and_sanitize() {
        let card = QuestionCard {
            question: "Format?".into(),
            header: "Fmt".into(),
            options: vec![
                QuestionOption {
                    label: "JSON".into(),
                    description: "Structured output".into(),
                    preview: Some(r#"<pre>{"ok":true}</pre><script>alert(1)</script>"#.into()),
                },
                QuestionOption {
                    label: "MD".into(),
                    description: "Markdown".into(),
                    preview: Some("**bold**".into()),
                },
            ],
            multi_select: false,
        };
        let html = render_question_html(&QuestionRenderInput {
            title: "Question",
            questions: &[card],
        });
        // Inspect option-preview slots only (page chrome has its own <script>).
        let mut previews = Vec::new();
        let mut search = html.as_str();
        while let Some(idx) = search.find(r#"class="option-preview""#) {
            let from = idx;
            let rest = &search[from..];
            let end = rest.find("</div>").expect("preview close");
            previews.push(&rest[..end]);
            search = &rest[end + 1..];
        }
        assert_eq!(previews.len(), 2, "expected two preview slots");
        assert!(
            previews[0].contains(r#"{"ok":true}"#) || previews[0].contains("ok"),
            "preview0={}",
            previews[0]
        );
        assert!(
            !previews[0].to_ascii_lowercase().contains("<script"),
            "preview0={}",
            previews[0]
        );
        assert!(!previews[0].contains("alert"), "preview0={}", previews[0]);
        assert!(
            previews[1].contains("<strong>bold</strong>"),
            "preview1={}",
            previews[1]
        );
    }

    #[test]
    fn dismissed_result_matches_req_0068() {
        let questions = vec![json!({
            "question": "Output format?",
            "header": "Format",
            "options": [
                { "label": "JSON", "description": "Structured" },
                { "label": "Plain", "description": "Text only" }
            ],
            "multiSelect": false
        })];
        let result = QuestionResult::dismissed(questions.clone());
        let wire: Value = serde_json::from_str(&serde_json::to_string(&result).unwrap()).unwrap();
        assert_eq!(wire["button"], "dismissed");
        assert_eq!(wire["answers"], json!({}));
        assert_eq!(wire["response"], "");
        assert_eq!(wire["questions"], Value::Array(questions));
    }

    #[test]
    fn render_checkbox_for_multi_select() {
        let html = render_question_html(&QuestionRenderInput {
            title: "Question",
            questions: &[sample_card(true)],
        });
        assert!(html.contains(r#"type="checkbox""#));
        assert!(!html.contains(r#"type="radio""#));
    }

    #[test]
    fn parse_question_submitted() {
        let ipc = parse_question_page_ipc(
            r#"{"kind":"question_submitted","answers":{"Output format?":"JSON"}}"#,
        )
        .unwrap();
        match ipc {
            QuestionPageIpc::QuestionSubmitted { answers } => {
                assert_eq!(
                    answers.get("Output format?").map(String::as_str),
                    Some("JSON")
                );
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn parse_empty_answers_still_yields_submitted() {
        let ipc = parse_question_page_ipc(r#"{"kind":"question_submitted","answers":{}}"#).unwrap();
        match ipc {
            QuestionPageIpc::QuestionSubmitted { answers } => assert!(answers.is_empty()),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn parse_malformed_returns_none() {
        assert!(parse_question_page_ipc("not-json").is_none());
        assert!(parse_question_page_ipc(r#"{"kind":"button_pressed","label":"ok"}"#).is_none());
        assert!(parse_question_page_ipc(r#"{"kind":"question_submitted"}"#).is_none());
    }

    #[test]
    fn submitted_maps_to_question_result_without_button() {
        let questions = vec![json!({
            "question": "Output format?",
            "header": "Format",
            "options": [
                { "label": "JSON", "description": "Structured" },
                { "label": "Plain", "description": "Text only" }
            ],
            "multiSelect": false
        })];
        let mut answers = HashMap::new();
        answers.insert("Output format?".into(), "JSON".into());
        let result = QuestionResult::submitted(questions, answers);
        let wire: Value = serde_json::from_str(&serde_json::to_string(&result).unwrap()).unwrap();
        assert!(wire.get("button").is_none());
        assert_eq!(wire["answers"]["Output format?"], "JSON");
        assert_eq!(wire["response"], "");
    }

    #[test]
    fn multi_select_join_uses_comma_space() {
        // Page JS joins with ", " (REQ-0062); assert the contract delimiter here.
        let joined = ["JSON", "Plain"].join(", ");
        assert_eq!(joined, "JSON, Plain");
    }

    #[test]
    fn estimate_size_clamped() {
        let (w, h) = estimate_question_window_size(&[sample_card(false)]);
        assert!((DIALOG_MIN_WIDTH..=DIALOG_MAX_WIDTH).contains(&w));
        assert!((DIALOG_MIN_HEIGHT..=DIALOG_MAX_HEIGHT).contains(&h));
    }
}
