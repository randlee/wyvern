# Wyvern PRD
### What You View, Engine Renders Natively

---

## Overview

Wyvern is a lightweight CLI tool that opens native webview windows for user interaction and returns structured JSON results. It is designed to be MCP-compatible from the ground up — the JSON schema for CLI input maps 1:1 to MCP tool parameters with no restructuring required.

All UI is rendered as HTML/CSS/JS within a consistent HTML chrome frame. Wyvern has no domain knowledge — it is a dumb host that manages window lifecycle, navigation state, and JSON I/O.

The MVP surface is intentionally small:
- Blocking dialog commands: `message`, `input`, `markdown`, `question`, `wizard`
- `--interactive` lifecycle actions: `show`, `hide`, `exit`

If something feels complicated, treat that first as a documentation, contract, or scoping problem. Review and hardening should simplify toward the smallest coherent API before introducing new command types.

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

### Interactive mode (persistent stdin loop)
```bash
wyvern --interactive
```

Opens the window and enters a read loop on stdin. Each line is a JSON command. Stays alive until `{"action": "exit"}` is received or the user closes the window. Blocking dialog commands keep their normal modal behavior inside the loop. Responses are written to stdout as JSON lines.

```bash
# Example session
{"type": "message", "title": "Continue?", "message": "Ready for the next step?", "buttons": "yes_no"}
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
# write JSON lines to the process's stdin handle
```

This is the same pattern as background PowerShell job IPC — no MCP required, works in any shell environment including Claude Code.

### MCP mode

```bash
wyvern --mcp
```

Starts Wyvern as an MCP server over stdio. In MVP, the public MCP tool surface is the blocking dialog commands only; lifecycle actions remain part of `--interactive`.

#### Lifecycle action commands

These commands are valid only in `--interactive`:

```json
{ "action": "show" }
```

```json
{ "action": "hide" }
```

```json
{ "action": "exit" }
```

Successful action result:

```json
{ "action": "show | hide | exit", "ok": true }
```

### Deferred fire-and-forget path

MVP does not overload `message` with modeless semantics. If Wyvern needs an ephemeral update surface later, it should be introduced as a separate `notification` command.

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

A blocking modal dialog with title, message body, and standard button combinations.

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
  "content": "string (optional — exactly one of file or content)",
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

A multi-page wizard with browser-history navigation. The host is domain-agnostic — it only understands page descriptors, explicit navigation pointers, and opaque page data.

**Input:**
```json
{
  "type": "wizard",
  "page": {
    "id": "string",
    "title": "string",
    "html": "path/to/wizard.html"
  },
  "config": { },
  "width": 800,
  "height": 600
}
```

`config` is passed to the wizard HTML on load as opaque data. The host never inspects page-specific `data`.

#### Minimal Page Descriptor

```json
{
  "id": "layout-picker",
  "title": "Choose Layout",
  "html": "pages/layout-picker.html"
}
```

- `id`: stable page identity used for browser-style history restoration
- `title`: human-readable page title for the window chrome and user recognition
- `html`: relative or absolute path to the page HTML

#### Navigation Contract

**Page → Host** (on any user action):
```json
{ "action": "back", "page": { }, "data": { } }
```

```json
{ "action": "next", "page": { }, "data": { }, "next": { } }
```

```json
{ "action": "finish", "page": { }, "data": { } }
```

```json
{ "action": "cancel" }
```

**Host → Page** (on page load):
```json
{ "page": { }, "page_data": { }, "stack": [ ] }
```

- `page`: the current page descriptor
- `page_data`: this page's previously collected data (populated on back-navigation restore)
- `stack`: full history array of all prior page entries as `{ "page": { }, "data": { } }` — readable by JS for context-aware rendering
- `data` fields are opaque to the host — stored and passed through, never interpreted

#### History Model

Browser-style cursor over a history array:

| Action | Effect |
|--------|--------|
| Forward (explicit `next`) | Push page + data, advance cursor |
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
  "stack": [ { "page": { }, "data": { } } ]
}
```

---

## `question` Type — AskUserQuestion Compatibility

Wyvern's `question` command is based on Claude's public `AskUserQuestion` API. Wyvern keeps its standard `type: "question"` command envelope and reuses the public AskUserQuestion fields and behavior inside that envelope.

**Input:**
```json
{
  "type": "question",
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

- `questions`: 1–4 questions
- `header`: short label, max 12 characters
- `options`: 2–4 per question; `preview` is optional HTML or markdown fragment
- `multiSelect`: if true, user may select multiple options
- Multi-step or page-based questionnaires are wizard flows, not `question`

**Return:**
```json
{
  "questions": [ ... ],
  "answers": {
    "How should I format the output?": "Summary",
    "Which sections?": "Introduction, Conclusion"
  },
  "response": "optional freeform reply if user bypasses structured options"
}
```

On force close, Wyvern returns:

```json
{
  "button": "dismissed",
  "questions": [ ... ],
  "answers": { },
  "response": ""
}
```

This `button` field is a Wyvern-specific extension for abnormal termination.

In `--interactive` mode, `question` blocks the read loop until answered, then writes result to stdout and resumes. This is normal loop behavior, not a special transport-specific semantic.

---

## Validation

Wyvern validates all input JSON before opening any window. Errors written to stderr as structured JSON; process exits with non-zero code.

**Error format:**
```json
{ "error": "validation", "field": "buttons", "message": "got 'ok-cancel', expected one of: ok, ok_cancel, yes_no, yes_no_cancel, retry_cancel, custom" }
```

Other error kinds:

```json
{ "error": "parse", "message": "expected JSON object" }
```

```json
{ "error": "io", "field": "file", "message": "could not read path 'missing.md'" }
```

```json
{ "error": "state", "field": "action", "message": "show is only valid in --interactive mode" }
```

**Rules:**
- Unknown fields → error (not silently ignored)
- Missing required field → `"missing required field 'type'"`
- Wrong enum value → list valid options; suggest closest match (Levenshtein distance ≤ 2)
- Wrong type → `"field 'multiline' expected boolean, got string"`
- `buttons: custom` without `custom_buttons` array → explicit error
- `custom_buttons` with non-custom `buttons` value → explicit error
- `mode: file` or `mode: folder` with `multiline: true` → explicit error
- `markdown` with both `file` and `content`, or with neither → explicit error
- `filter` or `multiple` outside `mode: file` → explicit error
- `placeholder` or `default` outside `mode: text` → explicit error
- `show` / `hide` / `exit` outside `--interactive` → explicit state error

---

## Return Values Summary

| Command | Return |
|------|--------|
| `message` | `{ "button": "..." }` |
| `input` | `{ "button": "...", "input": "..." }` |
| `markdown` | `{ "button": "..." }` |
| `wizard` | `{ "button": "...", "data": {}, "stack": [] }` |
| `question` (normal completion) | `{ "questions": [...], "answers": {}, "response": "" }` |
| `question` (force close) | `{ "button": "dismissed", "questions": [...], "answers": {}, "response": "" }` |
| `show` / `hide` / `exit` in `--interactive` | `{ "action": "...", "ok": true }` |

---

## MCP Compatibility

Each dialog type maps directly to an MCP tool. JSON field names are identical — no restructuring required.

1. **CLI**: `wyvern input.json` → stdout JSON result
2. **MCP tool**: same JSON as tool parameters → same JSON as tool result

### MCP Mode

When running as an MCP server, Wyvern is a persistent background process:
- Window survives across tool calls instead of launching per call
- Blocking dialog tools keep the same modal semantics they have in the CLI
- State persists for the lifetime of the MCP server process

---

## Open Questions / TBDs

- Icon image set: final list of named icons and number of variants per role
- Default window dimensions for `message` and `input` types

---

## Post-MVP

- **AI-generated icons**: Since all UI is HTML, message boxes can display any image. Post-MVP, integrate an image generation AI to produce custom icon sets on demand, cached as standard web assets (SVG/PNG/WebP). MVP ships with a curated static bundle.
