# Dialog IPC Contract (Phase B)

Authoritative contract for bidirectional messaging between dialog page JavaScript and the Rust host in `wyvern-window`. Wizard IPC (Phase D) is a separate contract.

## Transport

- **Mechanism:** `wry` IPC / custom protocol handler (same bridge pattern as future wizard pages)
- **Encoding:** JSON strings only — one object per message
- **Direction:** Page → host (user actions); host → page (initial state injection only in Phase B)

## Page → host messages

All messages include `"kind"` (string discriminator).

### `button_pressed`

User clicked a button in the HTML button bar.

```json
{ "kind": "button_pressed", "label": "ok" }
```

- `label` — wire value written to stdout `button` field (see mapping table below)
- Host closes window and completes `run()` with `CommandResult`

### `input_submitted`

User confirmed an `input` dialog.

```json
{ "kind": "input_submitted", "button": "ok", "value": "user text" }
```

- `value` — required for `mode: text` (or omitted) when `button` is `ok`; omitted when `button` is `cancel`
- **`mode: file` / `mode: folder` (b.4):** page sends `{ "kind": "input_submitted", "button": "ok" }` **without** `value`. Host opens native picker via `rfd` synchronously on receive; on selection, host completes with `InputResult.input` set to path string (or path array when `multiple: true`). Picker cancel leaves dialog open (no stdout yet); user may retry or press Cancel.
- Text field is hidden for file/folder modes; message prompt and button bar remain visible per b.4 UX flow.

### `question_submitted`

User clicked Submit on a `question` dialog (b.7+). Question dialogs use a dedicated Submit control — not the preset `buttons` mapping table.

```json
{
  "kind": "question_submitted",
  "answers": { "Output format?": "JSON", "Pick tools": "JSON, Plain" }
}
```

- `answers` — map keyed by each card's `question` string; values are selected `options[].label` (comma-joined when `multiSelect: true` per REQ-0062)
- Host closes window and completes with `QuestionResult` **without** top-level `button` field (REQ-0067)
- Host echoes input `questions` array verbatim in stdout result
- Validation of answer completeness (every card answered) is enforced in page JS before send; host rejects empty `answers` with **REQ-0068** fail-safe: `{ "button": "dismissed", "questions": [...], "answers": {}, "response": "" }`

### `dismissed`

User closed via OS chrome (×) or equivalent.

```json
{ "kind": "dismissed" }
```

- Host maps to `ButtonLabel::dismissed()` for `message`/`markdown`/`input`
- `question` uses extended shape per REQ-0068 (includes `button`, `questions`, `answers`, `response`) — same shape for empty-`answers` fail-safe above

## Host → page injection (on load)

Host injects initial render context via embedded JSON in the HTML shell (not a live IPC channel in Phase B):

```json
{
  "type": "message",
  "title": "...",
  "message": "...",
  "buttons": ["OK", "Cancel"],
  "default_button": 0
}
```

Exact shape is an implementation detail of `render_*_html()` — must include everything the static page needs to render without further host round-trips until user action.

## Button label mapping (stdout `button` field)

| `buttons` preset | Rendered labels (in order) | stdout `button` values |
|------------------|---------------------------|------------------------|
| `ok` | `["OK"]` | `ok` |
| `ok_cancel` | `["OK", "Cancel"]` | `ok`, `cancel` |
| `yes_no` | `["Yes", "No"]` | `yes`, `no` |
| `yes_no_cancel` | `["Yes", "No", "Cancel"]` | `yes`, `no`, `cancel` |
| `retry_cancel` | `["Retry", "Cancel"]` | `retry`, `cancel` |
| `custom` | `custom_buttons[]` verbatim | each string as-is |

`default_button` is a **0-based index** into the rendered label array for the active preset (or `custom_buttons` when `buttons: custom`). Out-of-range → validation error at schema layer.

## Error handling

- Malformed IPC JSON from page → log via observability `log_error`, treat as `dismissed` (fail-safe close)
- Unknown `kind` → same fail-safe
- Host never panics on IPC; invalid messages must not hang the event loop

## Testing

- Unit-test label mapping in `wyvern-schema` or `wyvern-window` (no webview)
- Integration tests inject IPC messages via test harness hook on `ChromeApp` successor
- CI: button tests run under xvfb single-threaded (same as Phase A)
