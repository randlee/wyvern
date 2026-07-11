# `wyvern-window` — Requirements

*Part of the [principal requirements](../requirements.md).*

---

## Dialog Types (REQ-0010 – REQ-0018)

**REQ-0010** — Support `message` type: `type`, `title`, `message`, `markdown`, `status`, `level`, `icon`, `image`, `buttons`, `custom_buttons`, `default_button`.

**REQ-0011** — `message` button presets: `ok`, `ok_cancel`, `yes_no`, `yes_no_cancel`, `retry_cancel`, `custom`.

**REQ-0012** — `message` `level` values: `info`, `warning`, `error`, `question` — each maps to a distinct icon.

**REQ-0013** — Support `input` type: `type`, `title`, `message`, `markdown`, `status`, `icon`, `multiline`, `placeholder`, `default`, `mode`, `filter`, `multiple`, `start_path`, `buttons`.

**REQ-0014** — `input` `mode` values: `text`, `file`, `folder`.

**REQ-0015** — `input` `mode: file` supports `filter` (extension patterns) and `multiple` (multi-file selection).

**REQ-0016** — Support `markdown` type: renders `.md` file or inline markdown in a styled HTML viewer.

**REQ-0017** — Support `wizard` type: loads caller-supplied HTML, passes `config` object on load.

**REQ-0018** — Support `question` type: input and output schemas match Claude `AskUserQuestion` API exactly.

---

## Icon & Image System (REQ-0030 – REQ-0032)

**REQ-0030** — Ship a built-in icon set in web-renderable formats (SVG, PNG, WebP), organized by semantic role with multiple variants per role.

**REQ-0031** — Icons selectable by: name (`"warning"`), name + variant index (`"warning:2"`), file path, or base64 data URI.

**REQ-0032** — `message` type supports optional `image` field for a decorative body image, specified the same way as `icon`.

---

## HTML Chrome Frame (REQ-0040 – REQ-0042)

**REQ-0040** — All dialog types render within a consistent HTML chrome frame: title bar, content area, optional status bar, button bar.

**REQ-0041** — Window auto-sizes to content with word-wrapping and a sensible maximum width and height.

**REQ-0042** — `wizard` type accepts explicit `width` and `height` overrides.

---

## Window Chrome — Platform (REQ-0080 – REQ-0087)

**REQ-0080** — macOS: transparent title bar with full-size content view; HTML content fills entire window including title bar area.

**REQ-0081** — HTML title bar reserves ~72px left safe zone on macOS for native traffic light buttons.

**REQ-0082** — HTML title bar element is draggable via `-webkit-app-region: drag`.

**REQ-0083** — Modal types (`message`, `input`, `markdown`, `question`) disable minimize and maximize/fullscreen.

**REQ-0084** — `wizard` type and `--interactive` mode enable minimize and allow window resizing.

**REQ-0085** — Windows and Linux: `decorations: false` with HTML-rendered close and minimize buttons via IPC.

**REQ-0086** — HTML close and minimize buttons invoke window actions via IPC on all platforms.

**REQ-0087** — Window draggable via `-webkit-app-region: drag` on all platforms.
