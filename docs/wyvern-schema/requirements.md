# `wyvern-schema` — Requirements

*Part of the [principal requirements](../requirements.md).*

---

## Phase A Executable Surface (REQ-0049)

**REQ-0049** — Phase A accepts exactly one executable command type: `chrome` with required `title` and optional `status`. All other `type` values are validation errors until their implementation phase ships.

---

## Validation (REQ-0050 – REQ-0057)

**REQ-0050** — Validate all input JSON before starting the dialog host (or opening any viewer). Validation scope matches the current phase's executable surface.

**REQ-0051** — Write validation errors to stderr as structured JSON: `{ "error": "validation", "field": "...", "message": "..." }`.

**REQ-0052** — Exit with non-zero code on validation failure.

**REQ-0053** — Unknown fields → error (not silently ignored).

**REQ-0054** — Wrong enum value → error listing all valid options; suggest closest match (Levenshtein distance ≤ 2). *Phase A: applies when enum fields are introduced per type.*

**REQ-0055** — `buttons: custom` without `custom_buttons` array → explicit error. *Phase B+ when `message` ships.*

**REQ-0056** — `custom_buttons` with non-`custom` `buttons` value → explicit error. *Phase B+.*

**REQ-0057** — `mode: file` or `mode: folder` combined with `multiline: true` → explicit error. *Phase B+ when `input` ships.*

---

## Command & Field Validation (REQ-0058 – REQ-0060)

**REQ-0058** — `markdown` requires exactly one of `file` or `content`. *Phase B+.*

**REQ-0059** — `input` cross-field validation rules. *Phase B+ (b.3 text subset; b.4 complete).*

- **`filter` or `multiple`** — only valid when `mode` is `file`. With `mode: text` (or omitted, which defaults to text) or `mode: folder` → explicit validation error on the offending field.
- **`placeholder` or `default`** — valid for any `input` mode (including `file` and `folder`) to pre-fill or hint the path field. With `mode: text` or omitted, same as before.
- **`start_path`** — only valid when `mode` is `file` or `mode` is `folder`. With `mode: text` or omitted → explicit validation error.

**Related:** **REQ-0057** — `mode: file` or `mode: folder` combined with `multiline: true` → explicit error (independent of REQ-0059; enforced when file/folder modes ship in b.4).

**REQ-0060** — `show`, `hide`, and `exit` are invalid outside `--interactive`; using them elsewhere produces a structured state error. Enforced from Phase A even though lifecycle actions ship in Phase E.

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

**Rust types:** [HTTP-TYPES.md](../plans/phase-C/HTTP-TYPES.md) (`CommandResult`, per-variant result structs).

---

## Icon fields (amendment c.9 — HTTP delivery)

**REQ-0030** and **REQ-0031** (Rust built-in icon catalog, named-icon validation) are **deprecated** when `icons.rs` is deleted in c.9.

- `icon` and `image` on `message` / `input` remain optional **opaque strings** (path, URL, or template hint).
- Templates in `ui/` interpret `level`, `icon`, and `image` — not `wyvern-schema`.
- Unknown named icons are **not** validation errors on the HTTP path (contrast with historical c.2 behavior).

---

## Error Model (REQ-0069 – REQ-0072)

Structured stderr uses the shared [`StderrError`](../../crates/wyvern-schema/src/stderr.rs) envelope. In addition to the historical `error` slug + `message` (and `field` when applicable), emit helpers attach:

| Field | When present | Purpose |
|-------|--------------|---------|
| `code` | always | Stable SCREAMING_SNAKE_CASE machine code (`PARSE_ERROR`, …) |
| `cause` | when set by emit helper | Why the failure occurred |
| `recovery` | non-empty array | Actionable recovery steps |
| `docs` | when set | Repo-relative requirements / architecture pointer |

Empty optional fields are omitted from JSON (`skip_serializing_if`).

**REQ-0069** — JSON parse failures: `LoadError::Parse` in `crates/wyvern` → stderr `{ "error": "parse", "code": "PARSE_ERROR", "message": "...", "cause": "...", "recovery": [...], "docs": "..." }` via `emit_parse_error`, exit `2`.

**REQ-0070** — Schema/cross-field failures: `ValidationError::Validation` → stderr `{ "error": "validation", "code": "VALIDATION_ERROR", "field": "...", "message": "...", "cause": "...", "recovery": [...], "docs": "..." }` via `emit_validation_error`, exit `4`.

**REQ-0071** — File/path load failures: `LoadError::Io` in `crates/wyvern` → stderr `{ "error": "io", "code": "IO_ERROR", "field": "...", "message": "...", "cause": "...", "recovery": [...], "docs": "..." }` via `emit_io_error`, exit `3`.

**REQ-0072** — Mode/state failures: `ValidationError::State` → stderr `{ "error": "state", "code": "STATE_ERROR", "field": "...", "message": "...", "cause": "...", "recovery": [...], "docs": "..." }` via `emit_validation_error`, exit `5`.

**REQ-0073** — Host/run failures (c.10+): `HostError` in `wyvern-host` → stderr `{ "error": "host_error" | "host_bind" | "host_viewer", "code": "HOST_ERROR" | "HOST_BIND_ERROR" | "HOST_VIEWER_ERROR", ... }` via `emit_host_error` in `crates/wyvern`, exit `6` / `7` per [HTTP-TYPES.md](../plans/phase-C/HTTP-TYPES.md).

**REQ-0073a (historical — wyvern-window, pre-c.9)** — `RunError` in `wyvern-window` → `window_create` / `event_loop` slugs. Removed with crate deletion; do not implement in new code.

**REQ-0078** — Emit-stage failures: when stdout or stderr JSON serialization fails at the CLI boundary (`EmitError::Serialize`), Wyvern emits `{ "error": "internal", "code": "INTERNAL_ERROR", "message": "...", "cause": "...", "recovery": [...], "docs": "..." }` (static JSON via `emit_fatal_internal`; no recursive serialize) and exits `8`. Applies only to emit helpers in `crates/wyvern`; does not change load/validate/run slugs. (Distinct from MCP **REQ-0074** in `docs/wyvern-mcp/requirements.md`.)
