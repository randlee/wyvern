# Wyvern — MVP Project Plan

A sprint is a single testable deliverable that fits within one AI context window (~200k tokens) and represents 1–5 days of focused work. Each sprint has explicit acceptance criteria that must pass before the next sprint begins.

**sc-lint-boundary** is a planning activity applied from Phase 2 onwards — architectural boundary rules are reviewed at sprint planning, not implemented as a sprint.

---

## Phase 1 — Foundation

**Phase goal:** A working binary with a native window, HTML chrome frame, and validated JSON I/O. Nothing useful yet — but everything subsequent phases build on.

**Phase acceptance criteria:** `wyvern '{"type":"message","title":"test","message":"hello","buttons":"ok"}'` opens a correctly-framed window and returns `{"button":"ok"}` to stdout.

---

### S1.1a — Rust project scaffold

Set up the Cargo workspace, add core dependencies (`wry`, `winit`, `serde`, `serde_json`, `strsim`), and confirm a `cargo build` produces a binary on macOS.

**Acceptance criteria:**
- `cargo build` succeeds with no warnings
- All dependencies resolve at pinned versions
- Workspace structure established for future crate separation

---

### S1.1b — Native window opens and closes

Wire up the `winit` event loop and `wry` WebView. Open a blank window; close it cleanly on OS × button or programmatic exit.

**Acceptance criteria:**
- `cargo run` opens a blank native window on macOS
- Window closes without panic or resource leak
- Transparent title bar + full-size content view active (ADR-0010)
- `-webkit-app-region: drag` wired on the title bar element

---

### S1.2a — CLI arg detection and JSON loading

Detect and load input from three sources: inline JSON string arg, `.json` file path, `.md` file path, and stdin.

**Acceptance criteria:**
- All four input modes correctly load their payload
- `.md` extension → markdown type shorthand
- `.json` extension → dialog command
- No arg + stdin → reads from stdin
- Ambiguous/missing input prints usage to stderr and exits non-zero

---

### S1.2b — JSON schema validation and error output

Validate all input against the full schema for all types. Write structured errors to stderr.

**Acceptance criteria:**
- Unknown fields → `{ "error": "validation", "field": "...", "message": "..." }` on stderr
- Missing required field → explicit named error
- Wrong enum value → lists valid options + Levenshtein suggestion (distance ≤ 2)
- Wrong type → `"expected boolean, got string"`
- Cross-field rules: `custom` buttons without `custom_buttons`, `multiline` + file mode
- Exit code non-zero on any validation failure
- Unit tests cover all validation rules

---

### S1.3a — HTML chrome frame structure

Implement the consistent HTML shell: title bar, content area, status bar (optional), button bar. Render static placeholder content in each zone.

**Acceptance criteria:**
- Frame renders correctly in the wry webview
- Title bar occupies full width with 72px left safe zone (macOS traffic lights)
- Status bar hides cleanly when not provided
- Button bar renders 1–5 buttons from a hardcoded array
- Auto-sizing: window fits content with defined max width/height

---

### S1.3b — Platform window chrome (close/minimize buttons)

Wire platform-specific window controls. macOS: traffic lights float over HTML. Windows/Linux: HTML-rendered close + minimize buttons via IPC.

**Acceptance criteria:**
- macOS: traffic light buttons visible and functional
- Windows + Linux: HTML close and minimize buttons call window actions via IPC
- Window draggable via title bar on all platforms
- Modal types (message, input, markdown, question): minimize disabled
- `{"button":"dismissed"}` returned when window closed via any OS mechanism

---

### S1.4 — sc-observability integration

Integrate the `sc-observability` structured logging library. Define logging conventions and usage guidelines for the Wyvern codebase.

**Acceptance criteria:**
- `sc-observability` added as dependency from `../sc-observability`
- Structured log output on: process start, command received, window open/close, result emitted, error
- `WYVERN_LOG` env var controls log level
- Usage guidelines documented in `docs/observability.md`
- All existing code updated to use structured logging

---

### S1.5 — sc-lint integration

Integrate the `sc-lint` lint tooling. Define lint rules and enforce them in CI.

**Acceptance criteria:**
- `sc-lint` added and configured from `../sc-lint`
- Lint passes on all existing code with zero warnings
- CI fails on lint errors
- Lint configuration documented in `docs/linting.md`
- sc-lint-boundary rules identified and noted for Phase 2 planning

---

## Phase 2 — Core Dialogs (MVP)

**Phase goal:** All four dialog types (`message`, `input`, `markdown`, `question`) work end-to-end from the CLI. This is the first genuinely useful version of Wyvern.

**Phase acceptance criteria:** A developer can replace any `zenity`/`osascript` dialog call with a `wyvern` command and get a richer, JSON-returning equivalent.

*sc-lint-boundary applied at sprint planning from this phase forward.*

---

### S2.1a — `message` type: structure and buttons

Render `title`, `message`, `status` in the chrome frame. Wire all button presets to return the correct JSON.

**Acceptance criteria:**
- All button presets render and return: `ok`, `ok_cancel`, `yes_no`, `yes_no_cancel`, `retry_cancel`
- `custom_buttons` array renders correctly
- `default_button` index is focused on open
- Returns `{"button":"<label>"}` to stdout on press
- Returns `{"button":"dismissed"}` on OS close

---

### S2.1b — `message` type: icons, images, markdown

Add `level` icon rendering, `icon` field, decorative `image` field, and `markdown` flag.

**Acceptance criteria:**
- `level` values (`info`, `warning`, `error`, `question`) render distinct placeholder icons
- `icon` field accepts named icon, file path, and base64 data URI
- `image` field renders a decorative body image
- `markdown: true` renders the `message` field as formatted markdown
- All combinations of fields render without layout breakage

---

### S2.2a — `input` type: text mode

Render a single-line or multiline text input with placeholder and default value.

**Acceptance criteria:**
- Single-line input renders and returns value on `ok`
- `multiline: true` renders a textarea
- `placeholder` displays as hint text
- `default` pre-fills the field
- Returns `{"button":"ok","input":"<value>"}` or `{"button":"cancel"}`

---

### S2.2b — `input` type: file and folder picker

Trigger the OS native file/folder chooser from the input dialog.

**Acceptance criteria:**
- `mode: file` opens the OS file picker; returns selected path
- `mode: folder` opens the OS folder picker; returns selected path
- `filter` restricts file picker to matching extensions
- `multiple: true` enables multi-file selection; returns JSON array of paths
- `start_path` sets the initial picker directory
- `multiline: true` with file/folder mode → validation error (REQ-0057)

---

### S2.3a — `markdown` type: file rendering

Load and render a `.md` file in a styled HTML viewer within the chrome frame.

**Acceptance criteria:**
- `wyvern my-doc.md` shorthand opens the viewer
- `{"type":"markdown","file":"path.md"}` JSON form works identically
- Markdown renders with headings, code blocks, tables, lists
- `title` defaults to filename when not provided
- `buttons: "ok"` default; returns `{"button":"ok"}` or `{"button":"dismissed"}`

---

### S2.3b — `markdown` type: inline content and styling

Support inline `content` field and apply a polished default stylesheet.

**Acceptance criteria:**
- `content` field renders inline markdown string (no file required)
- Stylesheet: readable typography, code highlighting, responsive to window width
- `status` bar renders below content when provided
- Content area scrolls for long documents without resizing the window

---

### S2.4a — `question` type: option rendering

Render question cards with radio (single-select) and checkbox (multi-select) option groups.

**Acceptance criteria:**
- All questions in the `questions` array render as cards
- `multiSelect: false` → radio buttons; `multiSelect: true` → checkboxes
- `header` renders as card label; `question` as card prompt
- `description` renders below each option label
- Returns correct `answers` map keyed by `question` text

---

### S2.4b — `question` type: preview, freeform, and schema compliance

Add `preview` field rendering, freeform "Other" input, and full Claude AskUserQuestion response compliance.

**Acceptance criteria:**
- `preview` HTML/markdown fragment renders alongside option when present
- "Other" freeform input appended after options; user text used as answer value
- `response` field populated when user dismisses structured options and types freely
- `questions` array passed through verbatim in response (REQ-0065)
- Tested against sample AskUserQuestion payloads from Claude Agent SDK docs

---

## Phase 3 — Release v0.1.0

**Phase goal:** Wyvern ships as a usable, cross-platform CLI tool. Icon set complete. Binaries available for download.

**Phase acceptance criteria:** `brew install wyvern` (or equivalent) works; a developer can run all Phase 2 dialog types on macOS, Windows, and Linux from a released binary.

---

### S3.1a — Icon image set (semantic roles)

Source or generate icons for all semantic roles in web-renderable formats.

**Acceptance criteria:**
- Roles covered: `info`, `warning`, `error`, `question`, `success`, `loading`
- Minimum 2 variants per role in SVG or PNG/WebP
- Assets bundled into binary via `include_bytes!`
- Named icon resolution works: `"warning"` → variant 1

---

### S3.1b — Icon variant selection

Implement full icon field resolution: named, indexed variant, file path, base64.

**Acceptance criteria:**
- `"warning"` → first variant
- `"warning:2"` → second variant
- `"/path/to/icon.svg"` → file loaded from disk
- `"data:image/..."` → base64 inline rendered
- Unknown named icon → validation error with list of valid names

---

### S3.2a — Windows and Linux full-size content view

Implement `decorations: false` + HTML close/minimize buttons on Windows and Linux.

**Acceptance criteria:**
- Windows: borderless window with HTML close + minimize buttons functional
- Linux: borderless window with HTML close + minimize buttons functional
- Window draggable on both platforms via `-webkit-app-region: drag`
- All Phase 2 dialog types render correctly on Windows and Linux

---

### S3.2b — Cross-platform validation and NFR pass

Verify performance targets and fix cross-platform rendering issues.

**Acceptance criteria:**
- NFR-0001: window opens < 500ms on macOS
- NFR-0002: resident memory < 80MB on macOS under normal operation
- NFR-0003: binary < 10MB on macOS
- No rendering regressions on Windows or Linux
- All Phase 2 acceptance criteria pass on all three platforms

---

### S3.3 — Release tooling and v0.1.0

GitHub Actions builds and publishes binaries. README quickstart complete.

**Acceptance criteria:**
- GitHub Actions matrix builds mac/win/linux binaries on tag push
- Release artifacts attached to GitHub release automatically
- README quickstart: install + 3 example commands runnable in < 5 minutes
- `CHANGELOG.md` entry for v0.1.0
- Tag `v0.1.0` pushed and release published

---

## Phase 4 — Wizard

**Phase goal:** Multi-page wizards with branching navigation and data persistence across pages.

**Phase acceptance criteria:** The example DAG layout-picker wizard completes a full flow with branching, back-navigation, data restoration, and returns the correct stack JSON.

---

### S4.1a — Wizard host: HTML load and config injection

Load caller-supplied HTML into the webview and inject `config` on load.

**Acceptance criteria:**
- `{"type":"wizard","html":"path/to/wizard.html","config":{}}` opens the HTML file
- `config` object injected into the page as `window.wyvern.config` on load
- Wizard window uses explicit `width`/`height` when provided
- Minimize enabled for wizard windows

---

### S4.1b — Wizard IPC contract

Implement bidirectional IPC between wizard pages and the Rust host.

**Acceptance criteria:**
- Page can send: `{"action":"next","button":"label","data":{}}` → host advances
- Page can send: `{"action":"back"}` → host navigates back
- Page can send: `{"action":"finish","data":{}}` → host closes + returns result
- Page can send: `{"action":"cancel"}` → host closes + returns `{"button":"cancel"}`
- Host sends on page load: `{"page_data":{},"stack":[]}`

---

### S4.2a — Browser-history navigation model

Implement the cursor-over-array history (ADR-0005).

**Acceptance criteria:**
- Forward navigation pushes page + data, advances cursor
- Back moves cursor back without truncating forward history
- Forward on same next-page restores cached page data
- Forward on different next-page truncates forward history and pushes new page
- History state verified by unit tests covering all four cases

---

### S4.2b — Stack injection and data restoration

Inject full history stack into each page on load; restore page data on back-navigation.

**Acceptance criteria:**
- `stack` array in host→page message contains all prior `{id, data}` entries
- `page_data` populated with this page's previously collected data on restore
- JS on any page can access `window.wyvern.stack` to read prior answers
- Data round-trips correctly through JSON serialization

---

### S4.3a — Example DAG layout-picker wizard

Build a working demo wizard: layout selection → N agent configuration pages.

**Acceptance criteria:**
- Step 1: layout cards rendered from `config.layouts` array
- Each layout card shows label + agent count
- Selecting a layout navigates to the first of N agent pages
- Each agent page collects a name and description
- `finish` returns full stack with layout selection + all agent configs

---

### S4.3b — Wizard polish and edge cases

Handle edge cases and improve wizard UX.

**Acceptance criteria:**
- First page: back button hidden or disabled
- Last page: next button label changes to "Finish"
- Empty `data` on a page handled gracefully (no undefined errors)
- Wizard with a single page (N=1) works correctly
- OS close on any wizard page returns `{"button":"dismissed","stack":[...]}`

---

## Phase 5 — Interactive & MCP

**Phase goal:** Wyvern runs as a persistent process, driveable by agents over stdin or as an MCP server.

**Phase acceptance criteria:** A Claude Code agent can open Wyvern in `--interactive` mode from a background shell, push markdown status updates, ask a question, receive the answer, and exit — with no MCP required.

---

### S5.1a — `--interactive` stdin loop and display commands

Implement the `--interactive` readline loop for fire-and-forget display commands.

**Acceptance criteria:**
- `wyvern --interactive` opens window and enters read loop on stdin
- `{"type":"markdown","content":"..."}` updates window content immediately
- `{"type":"image","file":"..."}` displays image in window
- `{"action":"hide"}` and `{"action":"show"}` toggle window visibility
- Loop continues waiting after each display command (no block)

---

### S5.1b — Blocking `question` in interactive mode

Implement blocking question handling and `exit` in the interactive loop.

**Acceptance criteria:**
- `{"type":"question",...}` blocks the loop until user answers
- Answer written to stdout as JSON line; loop resumes
- `{"action":"exit"}` closes window and terminates process cleanly
- Window close by user also terminates process and loop
- `--persistent` accepted as alias for `--interactive`

---

### S5.2a — MCP server wrapper and tool mapping

Implement Wyvern as an MCP server (stdio transport). Map each dialog type to an MCP tool.

**Acceptance criteria:**
- Wyvern starts as MCP server with `wyvern --mcp`
- Each type (`message`, `input`, `markdown`, `question`, `wizard`) registered as an MCP tool
- Tool parameter schemas identical to CLI JSON schemas (no renaming)
- MCP tool calls invoke the correct dialog and return result as tool response

---

### S5.2b — MCP persistent window and integration testing

Implement persistent window lifecycle for MCP mode; test with Claude Code.

**Acceptance criteria:**
- Window persists across MCP tool calls (`show`/`hide` semantics)
- `question` tool call blocks until user answers; result returned as tool response
- Display commands (`markdown`, `image`) are fire-and-forget in MCP context
- Tested end-to-end as registered MCP server in Claude Code
- `docs/mcp-setup.md` documents how to register Wyvern as an MCP server

---

## Phase Summary

| Phase | Sprints | Ships |
|-------|---------|-------|
| 1 — Foundation | 10 | Working binary, nothing useful |
| 2 — Core Dialogs | 8 | **MVP — all dialog types usable** |
| 3 — Release | 5 | **v0.1.0 on mac/win/linux** |
| 4 — Wizard | 6 | Multi-page wizard with branching |
| 5 — Interactive & MCP | 4 | Agent-driveable status viewer + MCP |

## Dependency Map

```
Phase 1
  └─ Phase 2 ──────────────────── sc-lint-boundary applied from here
       └─ Phase 3 (v0.1.0 release)
            └─ Phase 4 (wizard)
                 └─ Phase 5 (interactive + MCP)
```
