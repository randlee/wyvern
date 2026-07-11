# Wyvern PRD
### What You View, Engine Renders Natively

---

## Overview

Wyvern is a lightweight CLI tool that opens native webview windows for user interaction and returns structured JSON results. It is designed to be MCP-compatible from the ground up — the JSON schema for CLI input maps 1:1 to MCP tool parameters with no restructuring required.

All UI is rendered as HTML/CSS/JS within a consistent HTML chrome frame. Wyvern has no domain knowledge — it is a dumb host that manages window lifecycle, navigation state, and JSON I/O.

---

## Architecture

- **Webview engine**: `wry` (Tauri team) — wraps OS-native webviews
  - macOS: WebKit (system-native, ~30–50MB, near-instant load)
  - Windows: WebView2 (pre-installed Win11, optional Win10)
  - Linux: WebKitGTK (~100–150MB)
- **Shell**: Rust CLI binary
- **UI**: All chrome (title bar, status bar, buttons) rendered as HTML — no OS-native widgets
- **IPC**: Bidirectional JSON messaging between page JS and Rust host
- **Protocol**: JSON in / JSON out — stdin, file, or inline arg

---

## CLI Invocation

### Single-shot (blocking)
```bash
wyvern '{"type": "message", ...}'     # inline JSON string
wyvern input.json                      # JSON file (auto-detected by extension)
wyvern my-doc.md                       # Markdown viewer (auto-detected by extension)
echo '{...}' | wyvern                  # stdin
```

Extension detection order: `.md` → markdown viewer, `.json` → dialog, otherwise parse as inline JSON string.

### Interactive mode (non-blocking loop)
```bash
wyvern --interactive
```

Opens the window and enters a read loop on stdin. Each line is a JSON command. Stays alive until `{"action": "exit"}` is received or the user closes the window. Responses are written to stdout as JSON lines.

```bash
# Example session
{"type": "markdown", "content": "## Agent started"}
{"type": "image", "file": "chart.png"}
{"type": "question", ...}              # blocks loop until answered, prints result to stdout
{"action": "hide"}                     # hide window, keep process alive
{"action": "show"}                     # restore window
{"action": "exit"}                     # close window and terminate
```

`--persistent` is an alias for `--interactive`.

### Background shell usage (no MCP)

Any agent or script can drive Wyvern interactively by spawning it as a background process and holding the stdin/stdout handles open:

```bash
wyvern --interactive &
WYVERN_PID=$!
echo '{"type": "markdown", "content": "## Step 1 complete"}' >&${WYVERN_STDIN}
```

This is the same pattern as background PowerShell job IPC — no MCP required, works in any shell environment including Claude Code.

---

## HTML Chrome Frame

All dialog types share a consistent HTML frame:

```
┌─────────────────────────────────┐
│  [icon]  Title                  │  ← title bar (HTML)
├─────────────────────────────────┤
│                                 │
│         Content Area            │  ← type-specific
│                                 │
├─────────────────────────────────┤
│  Status bar text                │  ← status bar (HTML, optional)
├─────────────────────────────────┤
│              [ Button ] [ Btn ] │  ← button bar (HTML)
└─────────────────────────────────┘
```

- Window auto-sizes to content with word-wrap and a sensible max width/height
- All elements are HTML — fully themeable
- Explicit `width`/`height` overrides accepted on wizard type

---

## Icon Image System

Wyvern ships a curated set of images in standard web-renderable formats (SVG, PNG, WebP). Icons are used in:
- The title bar (alongside title text)
- The `message` type `level` field (info, warning, error, question)
- The `image` field (custom decorative image in message body)

Image sets are cycleable — multiple variants per semantic role, selectable by index or name. Custom images can be supplied via file path or base64.

```json
"icon": "warning"          // named icon from built-in set
"icon": "warning:2"        // second variant in the warning set
"icon": "/path/to/img.svg" // custom file path
"icon": "data:image/..."   // base64 inline
```

---

## Dialog Types

### 1. `message`

A modal dialog with title, message body, and standard button combinations. Blocks until dismissed.

**Input:**
```json
{
  "type": "message",
  "title": "string",
  "message": "string",
  "markdown": false,
  "status": "string (optional)",
  "level": "info | warning | error | question",
  "icon": "string (optional — named, path, or base64)",
  "image": "string (optional — decorative body image)",
  "buttons": "ok | ok_cancel | yes_no | yes_no_cancel | retry_cancel | custom",
  "custom_buttons": ["string"],
  "default_button": 0
}
```

**Return:**
```json
{ "button": "ok | cancel | yes | no | retry | <custom_label> | dismissed" }
```

`dismissed` is returned when the user closes the window via the OS (× button).
`markdown: true` renders `message` as markdown via a built-in HTML markdown renderer.

---

### 2. `input`

A modal dialog with a text entry field or file/folder chooser.

**Input:**
```json
{
  "type": "input",
  "title": "string",
  "message": "string",
  "markdown": false,
  "status": "string (optional)",
  "icon": "string (optional)",
  "multiline": false,
  "placeholder": "string (optional)",
  "default": "string (optional)",
  "mode": "text | file | folder",
  "filter": ["*.json", "*.txt"],
  "multiple": false,
  "start_path": "string (optional)",
  "buttons": "ok_cancel"
}
```

**Return:**
```json
{ "button": "ok | cancel | dismissed", "input": "string | path" }
```

For `multiple: true` file selection:
```json
{ "button": "ok", "input": ["/path/a", "/path/b"] }
```

---

### 3. `markdown` (shorthand)

Renders a `.md` file or inline markdown string in a styled HTML viewer within the standard frame.

**CLI shorthand:** `wyvern my-doc.md`

**JSON equivalent:**
```json
{
  "type": "markdown",
  "file": "path/to/doc.md",
  "title": "string (optional — defaults to filename)",
  "status": "string (optional)",
  "buttons": "ok"
}
```

**Return:**
```json
{ "button": "ok | dismissed" }
```

---

### 4. `wizard`

A multi-page wizard with browser-history navigation. The host is domain-agnostic — all content, flow logic, and state interpretation live in the HTML/JS/JSON config supplied by the caller.

**Input:**
```json
{
  "type": "wizard",
  "title": "string",
  "html": "path/to/wizard.html",
  "config": { },
  "width": 800,
  "height": 600
}
```

`config` is passed to the wizard HTML on load as opaque data. The host never inspects it.

#### Wizard Page Config (`wizard.json`)

```json
{
  "pages": [
    {
      "id": "layout-picker",
      "html": "pages/layout-picker.html",
      "buttons": [
        { "label": "Orchestrator → Dev → QA Loop", "next": "orch-agent-1" },
        { "label": "Explorer + Reporter",           "next": "explorer-agent-1" }
      ]
    },
    {
      "id": "orch-agent-1",
      "html": "pages/orch-agent-1.html",
      "buttons": [
        { "label": "Next", "next": "orch-agent-2" }
      ]
    }
  ]
}
```

#### Navigation Contract

**Page → Host** (on any user action):
```json
{ "action": "next | back | finish | cancel", "button": "label", "data": { } }
```

**Host → Page** (on page load):
```json
{ "page_data": { }, "stack": [ ] }
```

- `page_data`: this page's previously collected data (populated on back-navigation restore)
- `stack`: full history array of all prior pages' `{ id, data }` entries — readable by JS for context-aware rendering
- `data` fields are opaque to the host — stored and passed through, never interpreted

#### History Model

Browser-style cursor over a history array:

| Action | Effect |
|--------|--------|
| Forward (button press) | Push page + data, advance cursor |
| Back | Move cursor back — forward history preserved |
| Forward again, same next page | Restore cached page + data |
| Forward again, different next page | Truncate forward history, push new page |

```
A → B → C        history: [A, B, C]  cursor=2
back             history: [A, B, C]  cursor=1
back             history: [A, B, C]  cursor=0
→ B (same)       history: [A, B, C]  cursor=1  (B's data restored)
→ C (same)       history: [A, B, C]  cursor=2  (C's data restored)

back to A        history: [A, B, C]  cursor=0
→ D (different)  history: [A, D]     cursor=1  (B, C truncated)
```

**Wizard return:**
```json
{
  "button": "finish | cancel | dismissed",
  "data": { },
  "stack": [ { "id": "page-id", "data": { } } ]
}
```

---

## `question` Type — AskUserQuestion Compatibility

Matches the Claude `AskUserQuestion` JSON API exactly — same schema in, same schema out. No translation layer.

**Input:**
```json
{
  "questions": [
    {
      "question": "How should I format the output?",
      "header": "Format",
      "multiSelect": false,
      "options": [
        { "label": "Summary",  "description": "Brief overview", "preview": "<div>...</div>" },
        { "label": "Detailed", "description": "Full explanation" }
      ]
    }
  ]
}
```

- `header`: short label, max 12 characters
- `options`: 2–4 per question; `preview` is optional HTML or markdown fragment
- `multiSelect`: if true, user may select multiple options

**Return:**
```json
{
  "questions": [ ],
  "answers": {
    "How should I format the output?": "Summary",
    "Which sections?": ["Introduction", "Conclusion"]
  },
  "response": "optional freeform reply if user bypasses structured options"
}
```

In `--interactive` mode, `question` blocks the read loop until answered, then writes result to stdout and resumes.

---

## Validation

Wyvern validates all input JSON before opening any window. Errors written to stderr as structured JSON; process exits with non-zero code.

**Error format:**
```json
{ "error": "validation", "field": "buttons", "message": "got 'ok-cancel', expected one of: ok, ok_cancel, yes_no, yes_no_cancel, retry_cancel, custom" }
```

**Rules:**
- Unknown fields → error (not silently ignored)
- Missing required field → `"missing required field 'type'"`
- Wrong enum value → list valid options; suggest closest match (Levenshtein distance ≤ 2)
- Wrong type → `"field 'multiline' expected boolean, got string"`
- `buttons: custom` without `custom_buttons` array → explicit error
- `custom_buttons` with non-custom `buttons` value → explicit error
- `mode: file` or `mode: folder` with `multiline: true` → explicit error

---

## Return Values Summary

| Type | Return |
|------|--------|
| `message` | `{ "button": "..." }` |
| `input` | `{ "button": "...", "input": "..." }` |
| `markdown` | `{ "button": "..." }` |
| `wizard` | `{ "button": "...", "data": {}, "stack": [] }` |
| `question` | `{ "questions": [], "answers": {}, "response": "" }` |
| Any (force close) | `{ "button": "dismissed" }` |

---

## MCP Compatibility

Each dialog type maps directly to an MCP tool. JSON field names are identical — no restructuring required.

1. **CLI**: `wyvern input.json` → stdout JSON result
2. **MCP tool**: same JSON as tool parameters → same JSON as tool result

### MCP Mode

When running as an MCP server, Wyvern is a persistent background process:
- Window survives across tool calls — `show` / `hide` instead of launch / kill
- `question` calls block the MCP tool call until answered
- All other types are fire-and-forget display commands
- State persists for the lifetime of the MCP server process

---

## Open Questions / TBDs

- Icon image set: final list of named icons and number of variants per role
- Default window dimensions for `message` and `input` types
- Save vs. open mode for `file` input (currently assumes open)
- `filter` on folder chooser (N/A on most OSes — no-op or error?)

---

## Post-MVP

- **AI-generated icons**: Since all UI is HTML, message boxes can display any image. Post-MVP, integrate an image generation AI to produce custom icon sets on demand, cached as standard web assets (SVG/PNG/WebP). MVP ships with a curated static bundle.
