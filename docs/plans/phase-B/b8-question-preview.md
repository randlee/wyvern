---
id: b.8
title: Question preview and AskUserQuestion compliance
status: pending
branch: feature/phase-B-b8-question-preview
target: integrate/phase-B
---

# Sprint b.8 — `question` preview and AskUserQuestion compliance

## Goal

- Complete Phase B: `preview` field rendering and full public AskUserQuestion compliance.
- REQ-0068 force-close extension with `button: "dismissed"`.
- Phase B acceptance criteria #4 and #5 pass.

## Hard Dependencies

- b.7 question render

## Exact Targets

- `crates/wyvern-window/src/question/render.rs` — preview HTML fragments
- `crates/wyvern-window/src/question/template.html` — preview layout slot
- `crates/wyvern-schema/src/result.rs` — dismissed question result variant
- `crates/wyvern-schema/tests/validation_question.rs` — preview field acceptance
- `crates/wyvern/tests/` — AskUserQuestion fixture integration tests
- `docs/plans/phase-B/question-contract-examples.md` — reference (no change required unless gaps found)

## Deliverables

- `options[].preview` renders as HTML fragment beside option (markdown fragments converted to HTML at render time)
- Normal completion: no top-level `button` field (REQ-0067)
- Force close: `{ "button": "dismissed", "questions": [...], "answers": {}, "response": "" }` (REQ-0068)
- `response` field behavior matches public AskUserQuestion contract (optional, default empty)
- Tested against sample payloads from [question-contract-examples.md](question-contract-examples.md) and Claude Agent SDK public docs
- `question` type fully executable; all four dialog types complete per README

## Required Work — preview and compliance (authoritative)

### Preview rendering

```json
{
  "label": "JSON",
  "description": "Structured output",
  "preview": "<pre>{\"ok\":true}</pre>"
}
```

- Preview HTML sanitized before render (authoritative policy):
  - Strip `<script>` and `<style>` tags and their contents
  - Remove all `on*` event attributes from remaining tags
  - Reject `javascript:` URLs in `href`/`src`; allow `data:` URIs for inline images only
  - Allow semantic HTML tags (`pre`, `code`, `table`, `p`, `strong`, etc.) needed for preview fragments
- Markdown string in `preview` → converted via shared markdown renderer, then same sanitization pass applied
- Layout: preview column or block adjacent to option label; must not break card scroll

### Normal vs force-close stdout

| Scenario | `button` field | `answers` | `questions` |
|----------|----------------|-----------|-------------|
| Submit | absent | populated map | verbatim input |
| OS close | `"dismissed"` | `{}` | verbatim input |

Document in tests: `button` is **not** present on normal completion (Wyvern extension only for abnormal termination per REQ-0068).

### Compliance checklist

- Field names match public AskUserQuestion (`question`, `header`, `options`, `label`, `description`, `multiSelect`, `preview`)
- Constraints: 1–4 questions, 2–4 options, header ≤12 chars
- Multi-select comma-join semantics (REQ-0062)
- `type: "wizard"` still validation error (Phase D) — README AC #5

## Explicit Code Samples

```json
// Normal completion — from question-contract-examples.md
{
  "questions": [ { "question": "Output format?", "header": "Format", "options": [...], "multiSelect": false } ],
  "answers": { "Output format?": "JSON" },
  "response": ""
}

// Force close — REQ-0068
{
  "button": "dismissed",
  "questions": [ "..." ],
  "answers": {},
  "response": ""
}
```

```rust
// Dismissed question result
QuestionResult {
    button: Some(ButtonLabel::dismissed()), // only on force close
    questions: input_questions.clone(),
    answers: HashMap::new(),
    response: String::new(),
}
```

## This Sprint Does Not Close

- `wizard` type (Phase D)
- `--interactive` / MCP (Phase E)
- Win/Linux decoration polish (Phase C)

## Acceptance Criteria

- Preview sanitization: `<script>` stripped; `on*` attributes removed (unit tests with malicious fragment)
- All question-contract-examples.md cases pass in automated tests
- Normal submit: AskUserQuestion shape without `button`
- OS close: REQ-0068 extended shape with `button: "dismissed"`
- README phase acceptance #4 and #5 pass
- Phase B complete: all four dialog types executable

## Required Validation

- `cargo test --workspace -- --test-threads=1`
- Integration tests: fixtures from question-contract-examples.md
- Manual smoke optional: multi-card + preview layout on macOS
- `sc-lint check native --config .sc-lint.toml`
- Full README phase acceptance criteria #1–#5
