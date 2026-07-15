//! Automated coverage for `docs/plans/phase-B/question-contract-examples.md`.

use std::collections::HashMap;

use serde_json::json;
use wyvern_schema::{validate, Command, CommandResult, QuestionResult, ValidationError};

#[test]
fn contract_minimal_single_select_input_and_stdout() {
    let input = json!({
        "type": "question",
        "questions": [
            {
                "question": "Output format?",
                "header": "Format",
                "options": [
                    { "label": "JSON", "description": "Structured" },
                    { "label": "Plain", "description": "Text only" }
                ],
                "multiSelect": false
            }
        ]
    });
    let cmd = validate(&input).expect("minimal single-select");
    let Command::Question { questions_raw, .. } = cmd else {
        panic!("expected Question");
    };

    let mut answers = HashMap::new();
    answers.insert("Output format?".into(), "JSON".into());
    let result = CommandResult::Question(QuestionResult::submitted(questions_raw, answers));
    let wire: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&result).unwrap()).unwrap();

    // Normal completion — no top-level button (REQ-0067).
    assert!(wire.get("button").is_none());
    assert_eq!(wire["answers"]["Output format?"], "JSON");
    assert_eq!(wire["response"], "");
    assert_eq!(wire["questions"][0]["question"], "Output format?");
    assert_eq!(wire["questions"][0]["header"], "Format");
    assert_eq!(wire["questions"][0]["multiSelect"], false);
}

#[test]
fn contract_multi_select_comma_joined_labels() {
    let input = json!({
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
    });
    let cmd = validate(&input).expect("multi-select");
    let Command::Question { questions_raw, .. } = cmd else {
        panic!("expected Question");
    };
    let mut answers = HashMap::new();
    answers.insert("Pick tools".into(), "JSON, Plain".into());
    let result = CommandResult::Question(QuestionResult::submitted(questions_raw, answers));
    let wire: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&result).unwrap()).unwrap();
    assert!(wire.get("button").is_none());
    assert_eq!(wire["answers"]["Pick tools"], "JSON, Plain");
    assert_eq!(wire["response"], "");
}

#[test]
fn contract_force_close_req_0068() {
    let questions = vec![json!({
        "question": "Output format?",
        "header": "Format",
        "options": [
            { "label": "JSON", "description": "Structured" },
            { "label": "Plain", "description": "Text only" }
        ],
        "multiSelect": false
    })];
    let result = CommandResult::Question(QuestionResult::dismissed(questions));
    let wire: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&result).unwrap()).unwrap();
    assert_eq!(wire["button"], "dismissed");
    assert_eq!(wire["answers"], json!({}));
    assert_eq!(wire["response"], "");
    assert!(wire["questions"].is_array());
}

#[test]
fn contract_preview_field_accepted() {
    let input = json!({
        "type": "question",
        "questions": [{
            "question": "Output format?",
            "header": "Format",
            "options": [
                {
                    "label": "JSON",
                    "description": "Structured output",
                    "preview": "<pre>{\"ok\":true}</pre>"
                },
                { "label": "Plain", "description": "Text only" }
            ],
            "multiSelect": false
        }]
    });
    let cmd = validate(&input).expect("preview option");
    let Command::Question {
        questions,
        questions_raw,
        ..
    } = cmd
    else {
        panic!("expected Question");
    };
    assert_eq!(
        questions[0].options[0].preview.as_deref(),
        Some(r#"<pre>{"ok":true}</pre>"#)
    );
    assert_eq!(
        questions_raw[0]["options"][0]["preview"],
        r#"<pre>{"ok":true}</pre>"#
    );
}

#[test]
fn contract_validation_failures() {
    // 0 questions
    let err = validate(&json!({ "type": "question", "questions": [] })).unwrap_err();
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "questions");
            assert!(message.contains("empty array") || message.contains("1–4"));
        }
        other => panic!("unexpected {other:?}"),
    }

    // 5 questions
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
    let err = validate(&json!({ "type": "question", "questions": cards })).unwrap_err();
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "questions");
            assert!(message.contains("max 4"));
        }
        other => panic!("unexpected {other:?}"),
    }

    // 1 option
    let err = validate(&json!({
        "type": "question",
        "questions": [{
            "question": "Only one?",
            "header": "One",
            "options": [{ "label": "A", "description": "a" }],
            "multiSelect": false
        }]
    }))
    .unwrap_err();
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "questions[0].options");
            assert!(message.contains("min 2"));
        }
        other => panic!("unexpected {other:?}"),
    }

    // header 13 chars
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
    .unwrap_err();
    match err {
        ValidationError::Validation { field, message } => {
            assert_eq!(field, "questions[0].header");
            assert!(message.contains("max 12"));
        }
        other => panic!("unexpected {other:?}"),
    }
}
