# Wyvern — Requirements

Requirements are categorized as functional (**REQ**) or non-functional (**NFR**). Each is numbered sequentially and linked to relevant ADRs where applicable.

---

## Functional Requirements

### CLI Invocation

**REQ-0001** — The CLI shall accept a JSON command as an inline string argument.
`wyvern '{"type": "message", ...}'`

**REQ-0002** — The CLI shall accept a `.json` file path and load it as the command.
`wyvern input.json`

**REQ-0003** — The CLI shall accept a `.md` file path and open it as a markdown viewer.
`wyvern my-doc.md`

**REQ-0004** — The CLI shall accept JSON via stdin when no argument is provided.
`echo '{...}' | wyvern`

**REQ-0005** — The CLI shall support `--interactive` (alias `--persistent`) to enter a readline loop on stdin, processing one JSON command per line until `{"action": "exit"}` or window close.

---

### Dialog Types

**REQ-0010** — The CLI shall support a `message` dialog type with the following fields:
`type`, `title`, `message`, `markdown`, `status`, `level`, `icon`, `image`, `buttons`, `custom_buttons`, `default_button`

**REQ-0011** — The `message` type shall support button presets: `ok`, `ok_cancel`, `yes_no`, `yes_no_cancel`, `retry_cancel`, and `custom`.

**REQ-0012** — The `message` type shall support `level` values: `info`, `warning`, `error`, `question` — each mapped to a distinct icon from the built-in image set.

**REQ-0013** — The CLI shall support an `input` dialog type with the following fields:
`type`, `title`, `message`, `markdown`, `status`, `icon`, `multiline`, `placeholder`, `default`, `mode`, `filter`, `multiple`, `start_path`, `buttons`

**REQ-0014** — The `input` type shall support `mode` values: `text`, `file`, `folder`.

**REQ-0015** — The `input` type with `mode: file` shall support `filter` (file extension patterns) and `multiple` (multi-file selection).

**REQ-0016** — The CLI shall support a `markdown` dialog type that renders a `.md` file or inline markdown string in a styled HTML viewer.

**REQ-0017** — The CLI shall support a `wizard` dialog type that loads caller-supplied HTML and passes a `config` object to it on load.

**REQ-0018** — The CLI shall support a `question` dialog type whose input and output schemas match the Claude `AskUserQuestion` API exactly. *(See ADR-0007)*

---

### Wizard Navigation

**REQ-0020** — The wizard host shall maintain a browser-history model: a cursor over an array of visited pages. *(See ADR-0005)*

**REQ-0021** — Back navigation shall move the cursor back without discarding forward history.

**REQ-0022** — Forward navigation to the same next-page as the cached entry shall restore that page's previously collected data.

**REQ-0023** — Forward navigation to a different next-page shall truncate all entries after the cursor and push the new page.

**REQ-0024** — On page load, the host shall inject `{ "page_data": {}, "stack": [] }` into the page via IPC, where `stack` contains all prior pages' `{ id, data }` entries.

**REQ-0025** — Pages shall signal navigation via IPC: `{ "action": "next|back|finish|cancel", "button": "label", "data": {} }`. The host shall treat `data` as opaque. *(See ADR-0006)*

---

### Icon & Image System

**REQ-0030** — Wyvern shall ship a built-in set of icons in web-renderable formats (SVG, PNG, WebP), organized by semantic role with multiple variants per role.

**REQ-0031** — Icons shall be selectable by name (`"warning"`), by name and variant index (`"warning:2"`), by file path, or by base64 data URI.

**REQ-0032** — The `message` type shall support an optional `image` field for a decorative body image, specified the same way as `icon`.

---

### HTML Chrome Frame

**REQ-0040** — All dialog types shall render within a consistent HTML chrome frame comprising: title bar, content area, optional status bar, and button bar.

**REQ-0041** — The window shall auto-size to content with word-wrapping and a sensible maximum width and height.

**REQ-0042** — The `wizard` type shall accept explicit `width` and `height` overrides.

---

### Validation & Errors

**REQ-0050** — The CLI shall validate all input JSON before opening any window.

**REQ-0051** — Validation errors shall be written to stderr as structured JSON: `{ "error": "validation", "field": "...", "message": "..." }`.

**REQ-0052** — The process shall exit with a non-zero code on validation failure.

**REQ-0053** — Unknown fields shall produce an error (not be silently ignored).

**REQ-0054** — Wrong enum values shall produce an error listing all valid options and suggesting the closest match when Levenshtein distance ≤ 2.

**REQ-0055** — `buttons: custom` without a `custom_buttons` array shall produce an explicit error.

**REQ-0056** — `custom_buttons` paired with a non-`custom` `buttons` value shall produce an explicit error.

**REQ-0057** — `mode: file` or `mode: folder` combined with `multiline: true` shall produce an explicit error.

---

### Return Values

**REQ-0060** — All dialog types shall write their result to stdout as a single JSON line on completion.

**REQ-0061** — When the user closes the window via the OS (× button), all types shall return `{ "button": "dismissed" }`.

**REQ-0062** — `message` and `markdown` types shall return `{ "button": "<label>" }`.

**REQ-0063** — `input` type shall return `{ "button": "<label>", "input": "<value>" }`. Multi-file selection shall return `input` as an array.

**REQ-0064** — `wizard` type shall return `{ "button": "finish|cancel|dismissed", "data": {}, "stack": [] }`.

**REQ-0065** — `question` type shall return the Claude `AskUserQuestion` response schema: `{ "questions": [], "answers": {}, "response": "" }`.

---

### Interactive & MCP Mode

**REQ-0070** — In `--interactive` mode, display commands (`message`, `markdown`, `image`) shall be fire-and-forget — the loop immediately awaits the next command.

**REQ-0071** — In `--interactive` mode, `question` commands shall block the loop until the user answers, then write the result to stdout before continuing.

**REQ-0072** — The `{"action": "show"}` and `{"action": "hide"}` commands shall show and hide the window without terminating the process.

**REQ-0073** — When running as an MCP server, Wyvern shall operate as a persistent background process. The window shall survive across tool calls. *(See ADR-0009)*

---

## Non-Functional Requirements

**NFR-0001** — On macOS, the window shall open in under 500ms from process launch.

**NFR-0002** — On macOS, resident memory shall not exceed 80MB under normal operation.

**NFR-0003** — The compiled binary shall not exceed 10MB on macOS.

**NFR-0004** — Wyvern shall not require a browser to be installed on the host system. *(See ADR-0001)*

**NFR-0005** — The CLI shall run on macOS (WebKit), Windows (WebView2), and Linux (WebKitGTK).

**NFR-0006** — The JSON schema for all dialog types shall map 1:1 to MCP tool parameters with no field renaming or restructuring required. *(See ADR-0004)*

**NFR-0007** — Validation error messages shall be human-readable and actionable — sufficient for a developer to fix the error without consulting documentation.

**NFR-0008** — The host shall never inspect or interpret wizard page data. All domain logic shall reside in caller-supplied HTML/JS. *(See ADR-0006)*

**NFR-0009** — The `question` type shall remain backward-compatible with the Claude `AskUserQuestion` API schema at all times. Extensions shall be additive only. *(See ADR-0007)*

**NFR-0010** — Interactive mode shall support concurrent use from a background shell process with stdin/stdout handles held open, with no additional setup required.
