# `wyvern-schema` — Requirements

*Part of the [principal requirements](../requirements.md).*

---

## Phase 1 Executable Surface (REQ-0049)

**REQ-0049** — Phase 1 accepts exactly one executable command type: `chrome` with required `title` and optional `status`. All other `type` values are validation errors until their implementation phase ships.

---

## Validation (REQ-0050 – REQ-0057)

**REQ-0050** — Validate all input JSON before opening any window. Validation scope matches the current phase's executable surface (Phase 1: `chrome` only).

**REQ-0051** — Write validation errors to stderr as structured JSON: `{ "error": "validation", "field": "...", "message": "..." }`.

**REQ-0052** — Exit with non-zero code on validation failure.

**REQ-0053** — Unknown fields → error (not silently ignored).

**REQ-0054** — Wrong enum value → error listing all valid options; suggest closest match (Levenshtein distance ≤ 2). *Phase 1: applies when enum fields are introduced per type.*

**REQ-0055** — `buttons: custom` without `custom_buttons` array → explicit error. *Phase 2+ when `message` ships.*

**REQ-0056** — `custom_buttons` with non-`custom` `buttons` value → explicit error. *Phase 2+.*

**REQ-0057** — `mode: file` or `mode: folder` combined with `multiline: true` → explicit error. *Phase 2+ when `input` ships.*

---

## Command & Field Validation (REQ-0058 – REQ-0060)

**REQ-0058** — `markdown` requires exactly one of `file` or `content`. *Phase 2+.*

**REQ-0059** — `input` cross-field validation rules. *Phase 2+.*

**REQ-0060** — `show`, `hide`, and `exit` are invalid outside `--interactive`; using them elsewhere produces a structured state error. Enforced from Phase 1 even though lifecycle actions ship in Phase 5.

---

## Question Contract (REQ-0061 – REQ-0062)

**REQ-0061** — `question` input uses Wyvern's standard `type: "question"` command envelope while preserving the public Claude `AskUserQuestion` field names and meanings inside that command.

**REQ-0062** — `question.questions` contains 1–4 entries. Each question has a `question` string, `header` of at most 12 characters, `options` with 2–4 entries, optional `preview`, and `multiSelect` boolean. Multi-select answers are serialized as comma-joined labels to match the public API.

---

## Return Values (REQ-0063 – REQ-0068)

**REQ-0063** — Every successful command writes exactly one JSON result line to stdout on completion.

**REQ-0063a** — `chrome` (Phase A) → `CommandResult::Chrome(ChromeResult { button: "dismissed" })` serializes to `{ "button": "dismissed" }` on OS close (see a.4 `#[serde(untagged)]` contract).

**REQ-0064** — `message` and `markdown` → `{ "button": "<label>" }`.

**REQ-0065** — `input` → `{ "button": "<label>", "input": "<value>" }`. Multi-file → `input` as array.

**REQ-0066** — `wizard` → `{ "button": "finish|cancel|dismissed", "data": {}, "stack": [] }`.

**REQ-0067** — `question` on normal completion → Claude AskUserQuestion response schema: `{ "questions": [], "answers": {}, "response": "" }`. `response` is optional.

**REQ-0068** — Force close behavior:
- `message`, `input`, `markdown`, `wizard` return `{ "button": "dismissed", ... }` in their normal shape
- `question` returns `{ "button": "dismissed", "questions": [...], "answers": {}, "response": "" }` as an explicit Wyvern extension for abnormal termination

---

## Error Model (REQ-0069 – REQ-0072)

**REQ-0069** — JSON parse failures: `LoadError::Parse` in `crates/wyvern` → stderr `{ "error": "parse", "message": "..." }`, exit non-zero.

**REQ-0070** — Schema/cross-field failures: `ValidationError::Validation` → stderr `{ "error": "validation", "field": "...", "message": "..." }`, exit non-zero.

**REQ-0071** — File/path load failures: `LoadError::Io` in `crates/wyvern` → stderr `{ "error": "io", "field": "...", "message": "..." }`, exit non-zero.

**REQ-0072** — Mode/state failures: `ValidationError::State` → stderr `{ "error": "state", "field": "...", "message": "..." }`, exit non-zero.

**REQ-0073** — Window/run failures: `RunError` in `wyvern-window` → stderr `{ "error": "window_create" | "event_loop", "message": "..." }` via `emit_run_error` in `crates/wyvern`, exit non-zero.
