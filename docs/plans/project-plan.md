# Wyvern — MVP Project Plan

A sprint is a single testable deliverable that fits within one AI context window (~200k tokens) and represents 1–5 days of focused work. Each sprint has explicit acceptance criteria that must pass before the next sprint begins.

**sc-lint-boundary** is a planning activity applied from Phase B onwards — architectural boundary rules are reviewed at sprint planning, not implemented as a sprint.

**Review and hardening principle:** If something feels complicated, assume the design is unclear or overspecified before assuming more API is needed. Reviews should attack complication directly by collapsing semantic drift, clarifying contracts, and defending the smallest coherent command surface.

**Integration branch map:**

| Integration branch | Project plan phase | Sprint docs |
|---|---|---|
| `integrate/phase-A` | Phase A — Foundation | `docs/plans/phase-A/` |
| `integrate/phase-B` | Phase B — Core Dialogs | `docs/plans/phase-B/` |
| `integrate/phase-C` | Phase C — Polish & Icons | `docs/plans/phase-C/` |
| `integrate/phase-D` | Phase D — Wizard | `docs/plans/phase-D/` |
| `integrate/phase-E` | Phase E — Persistent & MCP | `docs/plans/phase-E/` |

Phase A sprint PRs target `integrate/phase-A`. Sprint authority: `docs/plans/phase-A/` (sprints **a.1–a.7**).

---

## Phase A — Foundation

**Phase goal:** Cross-platform foundation binary with HTML chrome frame and validated JSON I/O on a **single direct path**. Only `type: "chrome"` is executable. Win/Linux decoration polish deferred to Phase C.

**Execution model:** `load (LoadError) → validate (ValidationError) → Command → run (RunError) → CommandResult → stdout`. One `type` → one handler. No CLI flags, no stub handlers.

**Phase acceptance criteria:**

1. `wyvern '{"type":"message",...}'` → validation stderr, exit ≠ 0, no window
2. `wyvern '{"type":"chrome","title":"Foundation"}'` → chrome opens; OS close → `{"button":"dismissed"}`
3. `wyvern '{"type":"unknown"}'` → validation stderr on `type`, exit ≠ 0, no window

**Platform:** Cross-platform code patterns + CI `cargo test --workspace` on ubuntu, macos, and windows. Win/Linux validation is CI-automated only (no manual E2E). Optional macOS manual chrome smoke during dev. Win/Linux decoration polish → Phase C.

**Sprints:** seven active (**a.1–a.7**). See [docs/plans/phase-A/README.md](phase-A/README.md).

| Sprint | Title | Doc |
|--------|-------|-----|
| a.1 | Workspace scaffold | [a1-scaffold.md](phase-A/a1-scaffold.md) |
| a.2 | Native window (tests) | [a2-window.md](phase-A/a2-window.md) |
| a.3 | JSON loading | [a3-json-io.md](phase-A/a3-json-io.md) |
| a.4 | Validation (`chrome`) | [a4-validation.md](phase-A/a4-validation.md) |
| a.5 | Chrome E2E | [a5-chrome-frame.md](phase-A/a5-chrome-frame.md) |
| a.6 | sc-observability | [a6-sc-observability.md](phase-A/a6-sc-observability.md) |
| a.7 | sc-lint | [a7-sc-lint.md](phase-A/a7-sc-lint.md) |

---

## Phase B — Core Dialogs (MVP)

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

### S2.4b — `question` type: preview and schema compliance

Add `preview` field rendering and schema compliance within Wyvern's `type: "question"` command envelope.

**Acceptance criteria:**
- `preview` HTML/markdown fragment renders alongside option when present
- Public AskUserQuestion field set and constraints are documented and enforced
- `response` field behavior matches the public AskUserQuestion contract
- `questions` array passed through verbatim in response
- Tested against sample AskUserQuestion payloads from Claude Agent SDK docs

---

## Phase C — Release v0.1.0

**Phase goal:** Wyvern ships as a usable, cross-platform CLI tool. Icon set complete. Binaries available for download.

**Phase acceptance criteria:** `brew install wyvern` (or equivalent) works; a developer can run all Phase B dialog types on macOS, Windows, and Linux from a released binary.

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

### S3.2a — Windows and Linux platform chrome

Implement `decorations: false` + HTML close/minimize buttons on Windows and Linux. Deferred from Phase A (was never in a.1–a.7 scope).

**Acceptance criteria:**
- Windows: borderless window with HTML close + minimize buttons functional via IPC
- Linux: borderless window with HTML close + minimize buttons functional via IPC
- Window draggable on both platforms via `-webkit-app-region: drag`
- All Phase B dialog types render correctly on Windows and Linux
- `chrome` foundation command still returns `{"button":"dismissed"}` on OS close on all platforms

---

### S3.2b — Cross-platform validation and NFR pass

Verify performance targets and fix cross-platform rendering issues.

**Acceptance criteria:**
- NFR-0001: window opens < 500ms on macOS
- NFR-0002: resident memory < 80MB on macOS under normal operation
- NFR-0003: binary < 10MB on macOS
- No rendering regressions on Windows or Linux
- All Phase B acceptance criteria pass on all three platforms

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

## Phase D — Wizard

**Phase goal:** Multi-page wizards with branching navigation and data persistence across pages.

**Phase acceptance criteria:** The example DAG layout-picker wizard completes a full flow with branching, back-navigation, data restoration, and returns the correct stack JSON.

---

### S4.1a — Wizard host: HTML load and config injection

Load caller-supplied HTML into the webview and inject the initial page descriptor plus `config` on load.

**Acceptance criteria:**
- `{"type":"wizard","page":{"id":"start","title":"Start","html":"path/to/wizard.html"},"config":{}}` opens the initial HTML file
- `config` object injected into the page as `window.wyvern.config` on load
- Wizard window uses explicit `width`/`height` when provided
- Minimize enabled for wizard windows

---

### S4.1b — Wizard IPC contract

Implement bidirectional IPC between wizard pages and the Rust host using explicit page descriptors.

**Acceptance criteria:**
- Page can send: `{"action":"next","page":{...},"data":{},"next":{...}}` → host advances
- Page can send: `{"action":"back","page":{...},"data":{}}` → host navigates back
- Page can send: `{"action":"finish","page":{...},"data":{}}` → host closes + returns result
- Page can send: `{"action":"cancel"}` → host closes + returns `{"button":"cancel"}`
- Host sends on page load: `{"page":{},"page_data":{},"stack":[]}`

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
- `stack` array in host→page message contains all prior `{page, data}` entries
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

## Phase E — Interactive & MCP

**Phase goal:** Wyvern runs as a persistent process, driveable by agents over stdin or as an MCP server.

**Phase acceptance criteria:** A Claude Code agent can open Wyvern in `--interactive` mode from a background shell, issue multiple blocking dialog commands against one persistent process, receive the JSON results, and exit — with no MCP required.

---

### S5.1a — `--interactive` stdin loop and lifecycle actions

Implement the `--interactive` readline loop and persistent-process lifecycle actions.

**Acceptance criteria:**
- `wyvern --interactive` opens window and enters read loop on stdin
- `{"action":"hide"}` and `{"action":"show"}` toggle window visibility
- Lifecycle actions return `{"action":"...","ok":true}`
- Loop remains alive after lifecycle actions and continues waiting for the next command

---

### S5.1b — Blocking dialogs and `exit` in interactive mode

Implement blocking dialog handling and `exit` in the interactive loop.

**Acceptance criteria:**
- Blocking dialog commands return their normal JSON result on stdout; loop resumes afterward
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
- Blocking dialog tools keep their normal CLI semantics and return their normal JSON result as the tool response
- Tested end-to-end as registered MCP server in Claude Code
- `docs/mcp-setup.md` documents how to register Wyvern as an MCP server

---

## Phase Summary

| Phase | Sprints | Ships |
|-------|---------|-------|
| Phase A — Foundation | 7 | Working binary, `chrome` command |
| Phase B — Core Dialogs | 8 | **MVP — all dialog types usable** |
| 3 — Release | 5 | **v0.1.0 on mac/win/linux** |
| 4 — Wizard | 6 | Multi-page wizard with branching |
| 5 — Interactive & MCP | 4 | Agent-driveable status viewer + MCP |

## Dependency Map

```
Phase A
  └─ Phase B ──────────────────── sc-lint-boundary applied from here
       └─ Phase C (v0.1.0 release)
            └─ Phase D (wizard)
                 └─ Phase E (interactive + MCP)
```
