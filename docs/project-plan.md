# Wyvern — MVP Project Plan

---

## Phase 1: Project Scaffold

Goal: working Rust binary that opens a blank wry window and exits cleanly.

- [ ] Initialize Rust project (`cargo new wyvern`)
- [ ] Add dependencies: `wry`, `winit`, `serde`, `serde_json`, `strsim`
- [ ] Wire up `winit` event loop + `wry` WebView
- [ ] Confirm window opens and closes on macOS, Windows, Linux
- [ ] CI: GitHub Actions build matrix (mac/win/linux)

**Exit criteria:** `cargo run` opens a blank window and exits cleanly on all platforms.

---

## Phase 2: JSON I/O & Validation

Goal: parse and validate all input schemas; return structured errors.

- [ ] Define Rust enums/structs for all dialog types (`message`, `input`, `markdown`, `wizard`, `question`)
- [ ] Implement CLI arg parsing — inline JSON string, `.json` file, `.md` file, stdin
- [ ] Implement JSON schema validation (REQ-0050 – REQ-0057)
  - Unknown fields → error
  - Wrong enum value → error + closest match suggestion (Levenshtein via `strsim`)
  - Missing required fields → error
  - Cross-field rules (`custom` buttons, `multiline` + `mode`, etc.)
- [ ] Write validation error output to stderr as structured JSON
- [ ] Exit with non-zero code on failure
- [ ] Unit tests for all validation rules

**Exit criteria:** all invalid inputs produce clear, actionable stderr JSON; all valid inputs parse without error.

---

## Phase 3: HTML Chrome Frame

Goal: consistent HTML shell rendered in the webview for all dialog types.

- [ ] Design and implement the HTML chrome template (title bar, content area, status bar, button bar)
- [ ] Implement auto-sizing logic (content-driven width/height with max bounds)
- [ ] Implement IPC bridge: JS → Rust message passing
- [ ] Implement button bar rendering from schema (`buttons` presets + `custom_buttons`)
- [ ] Wire `default_button` focus
- [ ] Return `{ "button": "dismissed" }` on OS window close (REQ-0061)

**Exit criteria:** a hardcoded `message` dialog renders correctly with buttons that return JSON on click.

---

## Phase 4: `message` Type

Goal: fully functional modal message dialog.

- [ ] Render `title`, `message`, `status` in chrome frame
- [ ] Implement `markdown: true` — render message body as markdown (marked.js or similar)
- [ ] Implement `level` icons (info / warning / error / question) from built-in image set placeholder
- [ ] Implement `icon` field (named, path, base64)
- [ ] Implement `image` field (decorative body image)
- [ ] Write result to stdout on button press
- [ ] Integration tests for all button presets

**Exit criteria:** `wyvern '{"type":"message",...}'` works end-to-end.

---

## Phase 5: `input` Type

Goal: text entry and file/folder chooser dialogs.

- [ ] Render single-line text input in chrome frame
- [ ] Implement `multiline` toggle (textarea)
- [ ] Implement `placeholder` and `default` value pre-fill
- [ ] Implement `mode: file` — trigger OS file picker, return path(s)
- [ ] Implement `mode: folder` — trigger OS folder picker
- [ ] Implement `filter` for file picker (extension patterns)
- [ ] Implement `multiple` for multi-file selection (return array)
- [ ] Return `{ "button": "...", "input": "..." }` on submit
- [ ] Integration tests for all modes

**Exit criteria:** `wyvern '{"type":"input",...}'` works for text, file, and folder modes.

---

## Phase 6: `markdown` Type

Goal: styled markdown viewer dialog.

- [ ] Implement `.md` file auto-detection from CLI arg (REQ-0003)
- [ ] Render markdown file content in styled HTML viewer
- [ ] Support inline `content` field as alternative to `file`
- [ ] Apply chrome frame with configurable `buttons`

**Exit criteria:** `wyvern my-doc.md` opens a readable markdown viewer.

---

## Phase 7: `question` Type

Goal: drop-in native renderer for Claude AskUserQuestion.

- [ ] Render question cards from `questions` array
- [ ] Implement radio (single) and checkbox (multi-select) option groups
- [ ] Render `preview` field as HTML or markdown fragment
- [ ] Support freeform "Other" text input per question
- [ ] Return response in Claude AskUserQuestion schema (REQ-0065)
- [ ] Integration test against sample AskUserQuestion payloads

**Exit criteria:** `wyvern '{"questions":[...]}'` renders correctly and returns valid AskUserQuestion response JSON.

---

## Phase 8: `wizard` Type

Goal: multi-page wizard with browser-history navigation.

- [ ] Load caller-supplied HTML into webview
- [ ] Inject `config` object on load via IPC
- [ ] Implement navigation IPC contract (page → host: `next/back/finish/cancel`)
- [ ] Implement browser-history model (ADR-0005, REQ-0020 – REQ-0025)
  - Cursor-over-array history
  - Back: move cursor, preserve forward
  - Forward same: restore cached page data
  - Forward different: truncate, push new
- [ ] Inject `{ page_data, stack }` into each page on load
- [ ] Return full stack on `finish`/`cancel`
- [ ] Build example DAG layout-picker wizard (validation/demo)

**Exit criteria:** a multi-page wizard with branching navigates correctly, preserves data on back, and returns the full stack.

---

## Phase 9: Icon Image Set

Goal: built-in curated icon set in web-renderable formats.

- [ ] Define icon roles: `info`, `warning`, `error`, `question`, `success`, `loading`
- [ ] Source or generate 2+ variants per role (SVG/PNG/WebP)
- [ ] Bundle assets into binary (via `include_bytes!` or embedded asset map)
- [ ] Implement icon resolution: named → variant index → file path → base64

**Exit criteria:** `"icon": "warning:2"` renders the correct bundled asset.

---

## Phase 10: Interactive Mode

Goal: `--interactive` persistent loop driven by stdin.

- [ ] Implement stdin readline loop on `--interactive` flag
- [ ] Route each JSON line to the appropriate dialog handler
- [ ] Fire-and-forget for display commands (`markdown`, `image`)
- [ ] Block loop on `question` until answered; write result to stdout
- [ ] Implement `show`, `hide`, `exit` action commands
- [ ] Test background shell usage (spawn + hold handles)

**Exit criteria:** `wyvern --interactive` accepts a stream of commands, displays content, answers questions, and exits cleanly.

---

## Phase 11: MCP Server

Goal: Wyvern as a persistent MCP server.

- [ ] Implement MCP server wrapper (stdio transport)
- [ ] Map each dialog type to an MCP tool with identical JSON schema
- [ ] Implement persistent window lifecycle (`show`/`hide` vs launch/kill)
- [ ] `question` tool call blocks until user answers
- [ ] Test with Claude Code MCP integration

**Exit criteria:** Wyvern registered as an MCP server responds to tool calls and renders native dialogs.

---

## Phase 12: Polish & MVP Release

- [ ] Cross-platform testing (macOS, Windows, Linux)
- [ ] Error message quality pass (all validation errors human-readable)
- [ ] Performance: confirm NFR-0001 (< 500ms open on macOS), NFR-0002 (< 80MB)
- [ ] README quickstart with install instructions
- [ ] Binary release via GitHub Actions (mac/win/linux)
- [ ] Tag `v0.1.0`

---

## Dependency Map

```
Phase 1 (scaffold)
  └─ Phase 2 (validation)
       └─ Phase 3 (chrome frame)
            ├─ Phase 4 (message)
            ├─ Phase 5 (input)
            ├─ Phase 6 (markdown)
            ├─ Phase 7 (question)
            └─ Phase 8 (wizard)
                 └─ Phase 9 (icons) ──── can run in parallel with 4-8
Phase 10 (interactive) ── depends on 4-8
Phase 11 (MCP) ────────── depends on 10
Phase 12 (release) ─────── depends on all
```
