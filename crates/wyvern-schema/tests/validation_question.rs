//! Integration tests for `type: "question"` validation (REQ-0061 / REQ-0062).

use serde_json::json;
use wyvern_schema::{validate, Command, QuestionCard, ValidationError};

fn minimal_card() -> serde_json::Value {
    json!({
        "question": "Output format?",
        "header": "Format",
        "options": [
            { "label": "JSON", "description": "Structured" },
            { "label": "Plain", "description": "Text only" }
        ],
        "multiSelect": false
    })
}

#[test]
fn validation_question_minimal_single_select_passes() {
    let input = json!({
        "type": "question",
        "questions": [minimal_card()]
    });
    let cmd = validate(&input).expect("valid question");
    match cmd {
        Command::Question {
            questions,
            questions_raw,
        } => {
            assert_eq!(questions.len(), 1);
            assert_eq!(questions_raw.len(), 1);
            let QuestionCard {
                question,
                header,
                options,
                multi_select,
            } = &questions[0];
            assert_eq!(question, "Output format?");
            assert_eq!(header, "Format");
            assert!(!multi_select);
            assert_eq!(options.len(), 2);
            assert_eq!(options[0].label, "JSON");
            assert_eq!(options[0].description, "Structured");
            assert!(options[0].preview.is_none());
            assert_eq!(questions_raw[0], minimal_card());
        }
        other => panic!("expected Question, got {other:?}"),
    }
}

#[test]
fn validation_question_multi_select_passes() {
    let cmd = validate(&json!({
        "type": "question",
        "questions": [{
            "question": "Pick tools",
            "header": "Tools",
            "options": [
                { "label": "JSON", "description": "A" },
                { "label": "Plain", "description": "B" }
            ],
            "multiSelect": true
        }]
    }))
    .expect("valid");
    match cmd {
        Command::Question { questions, .. } => {
            assert!(questions[0].multi_select);
        }
        other => panic!("expected Question, got {other:?}"),
    }
}

#[test]
fn validation_question_preview_accepted_not_required() {
    let cmd = validate(&json!({
        "type": "question",
        "questions": [{
            "question": "Output format?",
            "header": "Format",
            "options": [
                {
                    "label": "JSON",
                    "description": "Structured",
                    "preview": "<pre>{\"ok\":true}</pre>"
                },
                { "label": "Plain", "description": "Text only" }
            ],
            "multiSelect": false
        }]
    }))
    .expect("preview allowed");
    match cmd {
        Command::Question {
            questions,
            questions_raw,
        } => {
            assert_eq!(
                questions[0].options[0].preview.as_deref(),
                Some("<pre>{\"ok\":true}</pre>")
            );
            assert!(questions_raw[0]["options"][0].get("preview").is_some());
        }
        other => panic!("expected Question, got {other:?}"),
    }
}

#[test]
fn validation_question_four_cards_pass() {
    let cards: Vec<_> = (0..4)
        .map(|i| {
            json!({
                "question": format!("Q{i}?"),
                "header": format!("H{i}"),
                "options": [
                    { "label": "A", "description": "a" },
                    { "label": "B", "description": "b" }
                ],
                "multiSelect": false
            })
        })
        .collect();
    let cmd = validate(&json!({ "type": "question", "questions": cards })).expect("4 cards ok");
    match cmd {
        Command::Question { questions, .. } => assert_eq!(questions.len(), 4),
        other => panic!("expected Question, got {other:?}"),
    }
}

#[test]
fn validation_question_zero_cards_fails() {
    let err = validate(&json!({ "type": "question", "questions": [] })).expect_err("empty");
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "questions");
            assert!(message.contains("empty array") || message.contains("1–4"));
        }
        other => panic!("expected Validation, got {other:?}"),
    }
}

#[test]
fn validation_question_five_cards_fails() {
    let cards: Vec<_> = (0..5)
        .map(|i| {
            json!({
                "question": format!("Q{i}?"),
                "header": "Hdr",
                "options": [
                    { "label": "A", "description": "a" },
                    { "label": "B", "description": "b" }
                ],
                "multiSelect": false
            })
        })
        .collect();
    let err = validate(&json!({ "type": "question", "questions": cards })).expect_err("5 cards");
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "questions");
            assert!(message.contains("max 4"));
        }
        other => panic!("expected Validation, got {other:?}"),
    }
}

#[test]
fn validation_question_one_option_fails() {
    let err = validate(&json!({
        "type": "question",
        "questions": [{
            "question": "Only one?",
            "header": "One",
            "options": [{ "label": "A", "description": "a" }],
            "multiSelect": false
        }]
    }))
    .expect_err("1 option");
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "questions[0].options");
            assert!(message.contains("min 2"));
        }
        other => panic!("expected Validation, got {other:?}"),
    }
}

#[test]
fn validation_question_header_over_12_fails() {
    let err = validate(&json!({
        "type": "question",
        "questions": [{
            "question": "Q?",
            "header": "1234567890123",
            "options": [
                { "label": "A", "description": "a" },
                { "label": "B", "description": "b" }
            ],
            "multiSelect": false
        }]
    }))
    .expect_err("header too long");
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "questions[0].header");
            assert!(message.contains("max 12"));
        }
        other => panic!("expected Validation, got {other:?}"),
    }
}

#[test]
fn validation_question_header_exactly_12_passes() {
    validate(&json!({
        "type": "question",
        "questions": [{
            "question": "Q?",
            "header": "123456789012",
            "options": [
                { "label": "A", "description": "a" },
                { "label": "B", "description": "b" }
            ],
            "multiSelect": false
        }]
    }))
    .expect("12 chars ok");
}

#[test]
fn validation_question_empty_question_fails() {
    let err = validate(&json!({
        "type": "question",
        "questions": [{
            "question": "",
            "header": "H",
            "options": [
                { "label": "A", "description": "a" },
                { "label": "B", "description": "b" }
            ],
            "multiSelect": false
        }]
    }))
    .expect_err("empty question");
    match err {
        ValidationError::Validation { field, .. } => {
            assert_eq!(field, "questions[0].question");
        }
        other => panic!("expected Validation, got {other:?}"),
    }
}

#[test]
fn validation_question_missing_multiselect_fails() {
    let err = validate(&json!({
        "type": "question",
        "questions": [{
            "question": "Q?",
            "header": "H",
            "options": [
                { "label": "A", "description": "a" },
                { "label": "B", "description": "b" }
            ]
        }]
    }))
    .expect_err("missing multiSelect");
    match err {
        ValidationError::Validation { field, .. } => {
            assert_eq!(field, "questions[0].multiSelect");
        }
        other => panic!("expected Validation, got {other:?}"),
    }
}

#[test]
fn validation_question_unknown_field_fails() {
    let err = validate(&json!({
        "type": "question",
        "questions": [minimal_card()],
        "extra": true
    }))
    .expect_err("unknown");
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "extra");
            assert!(message.contains("unknown field"));
        }
        other => panic!("expected Validation, got {other:?}"),
    }
}

#[test]
fn validation_question_type_near_miss_suggests() {
    let err = validate(&json!({ "type": "questio", "questions": [] })).expect_err("typo");
    match err {
        ValidationError::Validation { message, .. } => {
            assert!(message.contains("did you mean 'question'"));
        }
        other => panic!("expected Validation, got {other:?}"),
    }
}
