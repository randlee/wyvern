# `wyvern-schema` — Requirements

*Part of the [principal requirements](../requirements.md).*

---

## Validation (REQ-0050 – REQ-0057)

**REQ-0050** — Validate all input JSON before opening any window.

**REQ-0051** — Write validation errors to stderr as structured JSON: `{ "error": "validation", "field": "...", "message": "..." }`.

**REQ-0052** — Exit with non-zero code on validation failure.

**REQ-0053** — Unknown fields → error (not silently ignored).

**REQ-0054** — Wrong enum value → error listing all valid options; suggest closest match (Levenshtein distance ≤ 2).

**REQ-0055** — `buttons: custom` without `custom_buttons` array → explicit error.

**REQ-0056** — `custom_buttons` with non-`custom` `buttons` value → explicit error.

**REQ-0057** — `mode: file` or `mode: folder` combined with `multiline: true` → explicit error.

---

## Return Values (REQ-0060 – REQ-0065)

**REQ-0060** — All dialog types write their result to stdout as a single JSON line on completion.

**REQ-0061** — OS window close (× button) → all types return `{ "button": "dismissed" }`.

**REQ-0062** — `message` and `markdown` → `{ "button": "<label>" }`.

**REQ-0063** — `input` → `{ "button": "<label>", "input": "<value>" }`. Multi-file → `input` as array.

**REQ-0064** — `wizard` → `{ "button": "finish|cancel|dismissed", "data": {}, "stack": [] }`.

**REQ-0065** — `question` → Claude AskUserQuestion response schema: `{ "questions": [], "answers": {}, "response": "" }`.
