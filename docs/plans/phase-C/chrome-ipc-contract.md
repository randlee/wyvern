# Chrome Window Control IPC (Phase C)

Extends [Phase B dialog IPC](../phase-B/ipc-dialog-contract.md) with **HTML window chrome** messages for Windows and Linux (ADR-0010a, REQ-0085–REQ-0087).

macOS does **not** use this contract — native traffic lights handle close/minimize (ADR-0010).

## Transport

Same as dialog IPC: `wry` custom protocol, JSON strings, page → host only for user-initiated window actions.

## Page → host messages

All messages include `"kind"`.

### `window_close`

User clicked HTML close button (Win/Linux title bar).

```json
{ "kind": "window_close" }
```

**Host behavior:**

| Command type | Result |
|--------------|--------|
| `chrome` | `CommandResult::Chrome { button: "dismissed" }` |
| `message`, `input`, `markdown` | Same as dialog `{ "kind": "dismissed" }` |
| `question` | REQ-0068 extended dismiss shape |

Equivalent to OS chrome close — must not double-emit.

### `window_minimize`

User clicked HTML minimize button (non-modal types only).

```json
{ "kind": "window_minimize" }
```

**Host behavior:**

- Call winit minimize on the window handle
- **Do not** complete `run()` or write stdout
- Ignored (no-op) when window attributes disallow minimize (modal types per REQ-0083)

## HTML integration

Window control buttons live in `#window-controls` inside `#title-bar`:

- `-webkit-app-region: no-drag` on controls container and buttons
- `-webkit-app-region: drag` on `#title-bar` / title text (REQ-0087)
- `#btn-minimize` omitted or `hidden` for modal dialog renders

Button click handlers post JSON via the same IPC bridge as `button_pressed`.

## Relationship to dialog IPC

| Message | Closes window? | Emits stdout? |
|---------|----------------|---------------|
| `button_pressed` | Yes | Yes |
| `input_submitted` | Yes (after picker if applicable) | Yes |
| `question_submitted` | Yes | Yes |
| `dismissed` | Yes | Yes |
| `window_close` | Yes | Yes (same mapping as dismissed) |
| `window_minimize` | No | No |

## Error handling

Same fail-safe as dialog contract:

- Malformed JSON → log + treat as `window_close` / dismissed
- Unknown `kind` → log + dismissed
- Host never panics on IPC

## Testing

See [Phase C README](README.md) CI validation section for matrix policy. Sprint-specific checks:

- Unit-test mapping: `window_close` → `ButtonLabel::dismissed()` / question REQ-0068 shape
- Integration test: inject `window_minimize`, assert window minimized flag without stdout
- CI: run under xvfb single-threaded on Linux; Win/macOS matrix per Phase A README
- Modal render test: minimize button not present in HTML for `message` type

## Platform scope

| Platform | HTML window controls | Native controls |
|----------|---------------------|-----------------|
| macOS | Not rendered | Traffic lights (close/minimize/zoom) |
| Windows | Close + minimize (non-modal) | None (`decorations: false`) |
| Linux | Close + minimize (non-modal) | None (`decorations: false`) |
