# Chrome Window Control IPC (Phase C)

Extends [Phase B dialog IPC](../phase-B/ipc-dialog-contract.md) with **HTML window chrome** messages for Windows and Linux (ADR-0010a, REQ-0085–REQ-0087).

macOS does **not** use this contract — native traffic lights handle close/minimize (ADR-0010).

## Transport

Same as dialog IPC: `wry` custom protocol, JSON strings, page → host only for user-initiated window actions.

IPC parsing lives in `crates/wyvern-window/src/chrome/ipc.rs` (`pub(crate) enum ChromeIpc`, `pub(crate) fn parse_chrome_ipc`). Dispatch remains in each app's `handle_ipc` (`run.rs` for Message/Input/Markdown/Chrome; `question/handler.rs` for Question). No top-level `src/ipc/` module tree — chrome IPC types stay under `chrome/` to avoid QuestionApp ↔ run module cycles.

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

- **Non-modal** (`chrome`): call winit minimize on the window handle; **do not** complete `run()` or write stdout
- **Modal** (`message`, `input`, `markdown`, `question`): explicit **no-op** in `handle_ipc` — return immediately without dismiss, stdout, or error. Must not fall through to malformed-IPC fail-safe (which would incorrectly dismiss)

Modal types also omit or hide the minimize button in HTML (`PlatformChrome.show_minimize = false`); the host no-op is defense-in-depth if IPC is injected anyway.

## HTML integration

Window control buttons live in `#window-controls` inside `#title-bar`:

- `-webkit-app-region: no-drag` on controls container and buttons
- `-webkit-app-region: drag` on `#title-bar` / title text (REQ-0087)
- `#btn-minimize` omitted or `hidden` for modal dialog renders
- Win/Linux: no 72px left padding on `#title-bar`; controls aligned **right** (see c.3 `PlatformChrome`)

Button click handlers post JSON via the same IPC bridge as `button_pressed`.

Authoritative JS wiring (each template with `#window-controls`):

```html
<script>
  (function () {
    var controls = document.getElementById("window-controls");
    if (!controls) return;
    controls.addEventListener("click", function (ev) {
      var btn = ev.target.closest("button[data-action]");
      if (!btn) return;
      var action = btn.getAttribute("data-action");
      if (action === "close") {
        window.ipc.postMessage(JSON.stringify({ kind: "window_close" }));
      } else if (action === "minimize") {
        window.ipc.postMessage(JSON.stringify({ kind: "window_minimize" }));
      }
    });
  })();
</script>
```

Buttons use `data-action="close"` / `data-action="minimize"` on `#btn-close` / `#btn-minimize`.

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
- **`window_minimize` on modal types is not an error** — silent no-op (distinct from unknown kind)

## Testing

See [Phase C README](README.md) CI validation section for matrix policy. Sprint-specific checks:

- Unit-test mapping: `window_close` → `ButtonLabel::dismissed()` / question REQ-0068 shape
- Unit-test: modal `window_minimize` → no stdout, no dismiss (MessageApp, InputApp, MarkdownApp, QuestionApp)
- Integration test: ChromeApp + `WYVERN_INJECT_IPC='{"kind":"window_close"}'` → `{"button":"dismissed"}`
- Integration test: inject `window_minimize` on chrome, assert window minimized flag without stdout
- CI: run under xvfb single-threaded on Linux; Win/macOS matrix per [Phase C README](README.md#ci-validation-authoritative)
- Modal render test: minimize button not present in HTML for `message` type

## Platform scope

| Platform | HTML window controls | Native controls |
|----------|---------------------|-----------------|
| macOS | Not rendered | Traffic lights (close/minimize/zoom) |
| Windows | Close + minimize (non-modal) | None (`decorations: false`) |
| Linux | Close + minimize (non-modal) | None (`decorations: false`) |
