# Wyvern — Architecture & Decision Records

Architecture decisions are recorded here as ADRs (Architecture Decision Records). Each captures context, the decision made, and consequences — including trade-offs accepted.

---

## ADR-0001: Use `wry` as the webview engine

**Status:** Accepted

**Context:**
Wyvern needs to render HTML/CSS/JS in a native window without bundling a full browser. Options considered:
- **Electron** — full Chromium bundle (~150MB), overkill for a CLI dialog tool
- **OS-native dialogs** — no HTML/CSS support, not customizable
- **egui** — native Rust UI, no HTML authoring, requires Rust for all UI layout
- **wry** — wraps OS-native webviews (WebKit/WebView2/WebKitGTK), ~5MB binary

**Decision:**
Use `wry` (Tauri team). It delegates rendering to the OS webview already present on the system, keeping the binary small and load times near-instant on macOS.

**Consequences:**
- macOS: WebKit — fast, low memory (~30–50MB), system-native
- Windows: WebView2 — pre-installed on Win11; optional install on Win10
- Linux: WebKitGTK — heavier (~100–150MB), slower start
- HTML/CSS/JS authoring is fully supported — any JS framework works
- No bundled browser means no control over exact rendering engine version

---

## ADR-0002: All UI rendered as HTML — no OS-native widgets

**Status:** Accepted

**Context:**
Dialog chrome (title bar, status bar, buttons) could be OS-native widgets or HTML-rendered. OS-native gives system-consistent look; HTML gives full control and theming.

**Decision:**
Render all chrome as HTML within the webview. Title bar, status bar, and button bar are HTML elements inside the window — not OS-native controls.

**Consequences:**
- Fully themeable — custom styles, animations, branding
- Consistent appearance across types and future types
- No OS accessibility APIs for the chrome (mitigated by HTML ARIA attributes)
- Enables image-rich dialogs and markdown natively without special-casing

---

## ADR-0003: Rust as the implementation language

**Status:** Accepted

**Context:**
`wry` is a Rust crate. The host binary needs to be small, fast-starting, and cross-platform.

**Decision:**
Implement Wyvern in Rust, using `wry` + `winit` for the event loop.

**Consequences:**
- Single statically-linked binary, no runtime dependency
- Small footprint; fast startup
- `serde_json` for JSON I/O; strong type-safety on the schema layer
- Levenshtein suggestions for validation errors implementable via `strsim` crate

---

## ADR-0004: JSON as the sole protocol — stdin/stdout

**Status:** Accepted

**Context:**
The tool needs a calling convention usable from any language, agent, or shell — including MCP tool calls.

**Decision:**
JSON in (stdin, file, or inline arg), JSON out (stdout). Errors on stderr as structured JSON. One command per line in interactive mode.

**Consequences:**
- Works from any shell, language, or agent with no SDK
- MCP tool parameters map 1:1 — no restructuring required
- Interactive mode is a simple readline loop — easy to drive from background processes
- Binary data (images) passed by file path or base64, not raw bytes

---

## ADR-0005: Wizard navigation uses browser-history model

**Status:** Accepted

**Context:**
A simple push/pop stack for wizard navigation loses forward history on back-navigation, forcing users to re-enter data if they go back and then forward on the same path.

**Decision:**
Implement a cursor-over-array browser-history model:
- Back moves the cursor back without discarding forward entries
- Forward on the same next-page restores cached data
- Forward on a *different* next-page truncates forward history and pushes the new page

**Consequences:**
- Users can explore back/forward freely without losing entered data
- Branching (choosing a different path) correctly clears the stale forward history
- Host maintains the history array and cursor; pages are stateless HTML
- Slightly more complex than a simple stack but well-understood (browser model)

---

## ADR-0006: Host is domain-agnostic — wizard data is opaque

**Status:** Accepted

**Context:**
Wyvern could interpret wizard page data (e.g., validate field values, understand DAG structure). This would couple the host to specific use-cases.

**Decision:**
The host stores and passes through `data` blobs without inspection. All domain logic lives in the HTML/JS. The host only manages navigation signals (`next`, `back`, `finish`, `cancel`) and the history stack.

**Consequences:**
- Any wizard can be built without changing Wyvern
- Pages can inspect the full stack via JS to make context-aware decisions
- Wyvern ships no wizard-specific business logic — it is a pure host
- Validation of wizard data is the caller's responsibility

---

## ADR-0007: Adopt Claude AskUserQuestion schema verbatim for `question` type

**Status:** Accepted

**Context:**
Wyvern needs a question/multiple-choice dialog type. Claude's `AskUserQuestion` tool already defines a well-specified schema for this. Options:
- Define a custom Wyvern schema
- Adopt the Claude schema verbatim

**Decision:**
Adopt the Claude `AskUserQuestion` JSON schema exactly — same input, same output. Wyvern becomes a drop-in native renderer for Claude's own tool calls.

**Consequences:**
- Zero translation layer when intercepting Claude's `AskUserQuestion` calls
- Can be used as a standalone question dialog with no Claude dependency
- Future extensions must remain backward-compatible with the Claude API
- Limited to 1–4 questions, 2–4 options each (current Claude API constraints)

---

## ADR-0008: Interactive mode uses stdin readline loop

**Status:** Accepted

**Context:**
A persistent Wyvern window needs a way to receive updates over time. Options considered:
- Named pipe / Unix socket
- Local HTTP server
- stdin readline loop

**Decision:**
`--interactive` flag puts Wyvern into a readline loop on stdin. Each newline-delimited JSON object is a command. Responses go to stdout. Process exits on `{"action": "exit"}` or window close.

**Consequences:**
- No socket setup or port conflicts
- Works identically in CLI and MCP modes
- Any agent or script can drive it by holding stdin/stdout handles open (background shell pattern)
- Sequential only — commands processed one at a time (sufficient for UI interaction cadence)

---

## ADR-0009: MCP mode runs Wyvern as a persistent background process

**Status:** Accepted

**Context:**
As an MCP server, Wyvern could launch and kill a window per tool call, or keep a persistent process with show/hide semantics.

**Decision:**
Wyvern MCP server is a persistent background process. The window persists across tool calls. `show`/`hide` commands control visibility. The same JSON command vocabulary used in `--interactive` is used for MCP tool calls.

**Consequences:**
- Window state (position, size, content) survives between tool calls
- No per-call launch latency after first invocation
- `question` tool calls block until answered, matching the `canUseTool` callback pattern
- A single Wyvern MCP instance serves the full agent session

---

## ADR-0010: Transparent title bar with full-size content view (macOS)

**Status:** Accepted

**Context:**
macOS windows have native traffic light buttons (close/minimize/maximize) in the upper-left. Since Wyvern renders all chrome as HTML (ADR-0002), three options exist:

- **Option A** — Keep native traffic lights; HTML content starts below the title bar strip
- **Option B** — Transparent title bar + `fullSizeContentView`; traffic lights float over HTML
- **Option C** — No decorations (`decorations: false`); fully borderless HTML window

**Decision:**
Use Option B — `with_titlebar_transparent(true)` + `with_fullsize_content_view(true)` on the `winit` WindowBuilder. Traffic lights remain visible and native. The HTML content view extends to fill the full window including the title bar area. Our HTML chrome renders a left-padding safe zone (~72px) so the title text does not clash with the traffic light buttons.

**Consequences:**
- Native macOS traffic lights visible and functional — familiar UX
- HTML content fills the full window for an immersive, modern look
- HTML title bar must reserve a ~72px left safe zone on macOS
- Window dragging handled via `-webkit-app-region: drag` on the HTML title bar element
- On Windows/Linux, standard window decorations remain (no transparent title bar support assumed for MVP)
- Minimize should be disabled for modal dialog types (`message`, `input`, `markdown`, `question`); enabled for `wizard` and `--interactive` status viewer

---

## ADR-0010a: Full-size content view extended to all platforms

**Status:** Accepted — supersedes NFR-0011 in requirements.md

**Context:**
ADR-0010 initially limited transparent/full-size content view to macOS. The goal — maximum screen real estate with close/minimize controls — applies equally on Windows and Linux.

**Decision:**
Apply full-size content view with no OS title bar on all platforms:
- **macOS** — `with_titlebar_transparent(true)` + `with_fullsize_content_view(true)`; native traffic lights float over HTML
- **Windows** — `with_decorations(false)` + custom HTML title bar with close/minimize buttons; use DWM hit-test to preserve native window snap/resize behavior
- **Linux** — `with_decorations(false)` + custom HTML title bar with close/minimize buttons

On Windows and Linux, Wyvern renders its own close and minimize buttons in the HTML chrome. These call `window.close()` / `window.minimize()` via IPC rather than relying on OS-drawn controls.

**Consequences:**
- Consistent immersive look across all platforms
- Maximum content area on every OS
- Windows/Linux require HTML-rendered close + minimize buttons wired to IPC
- Window dragging via `-webkit-app-region: drag` works on all three platforms
- NFR-0011 (macOS-only limitation) is voided
