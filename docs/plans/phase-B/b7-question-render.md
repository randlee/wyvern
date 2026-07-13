---
id: b.7
title: Question cards, radio and checkbox rendering
status: pending
branch: feature/phase-B-b7-question-render
target: integrate/phase-B
---

# Sprint b.7 — `question` cards, radio and checkbox rendering

## Goal

- Executable `type: "question"` for card rendering and answer collection.
- REQ-0061/REQ-0062 validation; radio (single-select) and checkbox (multi-select) groups.
- Preview field deferred to b.8; normal completion returns AskUserQuestion response shape.

## Hard Dependencies

- b.6 markdown complete (shared chrome, markdown for option descriptions if needed)

## Exact Targets

- `crates/wyvern-schema/src/command.rs` — `Command::Question { questions: Vec<QuestionCard> }`
- `crates/wyvern-schema/src/validate.rs` — question contract rules
- `crates/wyvern-schema/src/result.rs` — `QuestionResult`, `CommandResult::Question`
- `crates/wyvern-schema/tests/validation_question.rs`
- `crates/wyvern-window/src/question/` — template, render, IPC submit handler
- `crates/wyvern-window/src/run.rs` — dispatch `Command::Question`

## Deliverables

- Questions render as cards with `header` (≤12 chars) and `question` prompt
- `multiSelect: false` → radio buttons; `true` → checkboxes
- `description` below each `options[].label`
- `options[].preview` present → validation pass; **not rendered at b.7** (no preview slot in template; field preserved in passthrough `questions` array)
- Submit returns `{ "questions": [...], "answers": { "<question>": "<label>" }, "response": "" }` per [question-contract-examples.md](question-contract-examples.md)
- Multi-select answers comma-join labels (REQ-0062)
- Force close shape deferred to b.8 acceptance tests (REQ-0068)
- `question` executable for render+submit; preview compliance in b.8

## Required Work — question render behavior (authoritative)

### Validation (REQ-0062)

| Rule | Constraint |
|------|------------|
| `questions` length | 1–4 entries |
| `questions[].options` | 2–4 entries each |
| `questions[].header` | max 12 characters |
| `questions[].question` | required non-empty string |
| `multiSelect` | boolean, required per card |
| `preview` on option | allowed in schema; **not rendered** at b.7 (no template slot) |

### Render

- One card per `questions[]` entry
- Radio/checkbox `name` scoped per card
- Submit button (not preset button bar) — sends `question_submitted` IPC; normal completion has no stdout `button` field
- `questions` array echoed verbatim in stdout response

### IPC

- Submit control sends `question_submitted` per [ipc-dialog-contract.md](ipc-dialog-contract.md) (not `button_pressed`)
- Page validates every card has a selection before send
- Host builds `answers` map from IPC payload; echoes input `questions` verbatim in stdout
- Multi-select: comma-join selected `options[].label` with `", "` (REQ-0062)

### ADR-0007

- Wyvern envelope: `type: "question"` at top level
- Inner fields match public AskUserQuestion names (`question`, `header`, `options`, `multiSelect`, `description`)

## Explicit Code Samples

```rust
// crates/wyvern-schema/src/command.rs
pub struct QuestionOption {
    pub label: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preview: Option<String>, // accepted; not rendered until b.8
}

pub struct QuestionCard {
    pub question: String,
    pub header: String, // max 12 chars
    pub options: Vec<QuestionOption>, // 2–4 entries
    pub multi_select: bool, // JSON: "multiSelect"
}

pub enum Command {
    // ...
    Question { questions: Vec<QuestionCard> }, // 1–4 cards
}

#[derive(Serialize)]
pub struct QuestionResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub button: Option<ButtonLabel>, // None at b.7 normal completion; Some(dismissed) deferred to b.8
    pub questions: Vec<serde_json::Value>, // verbatim passthrough
    pub answers: std::collections::HashMap<String, String>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub response: String,
}
```

```json
{ "kind": "question_submitted", "answers": { "Output format?": "JSON" } }
```

See [question-contract-examples.md](question-contract-examples.md) for minimal single-select and multi-select stdout shapes.

## This Sprint Does Not Close

- `preview` HTML rendering (b.8)
- Full AskUserQuestion compliance audit (b.8)
- REQ-0068 force-close `button: dismissed` extension tests (b.8)
- `wizard` type

## Acceptance Criteria

- 1–4 question cards render with correct controls
- `multiSelect: false` → exactly one answer per card in `answers` map
- `multiSelect: true` → comma-joined labels in `answers`
- `header` and `description` render correctly
- `questions` array in stdout matches input verbatim
- `response` field present as empty string on normal completion
- Validation rejects 0 or 5 questions, 1 option, header > 12 chars
- `preview` field accepted in validation but not rendered (no layout slot at b.7)

## Required Validation

- `cargo test --workspace -- --test-threads=1`
- `cargo test -p wyvern-schema -- validation_question`
- IPC integration test: `question_submitted` → `QuestionResult` without `button`
- Compare stdout against examples in question-contract-examples.md (minus preview case)
