# `wyvern-window` — Architecture

*Part of the [principal architecture](../architecture.md).*

---

## ADR-0001: Use `wry` as the webview engine

**Status:** Accepted

**Context:** Options: Electron (~150MB), OS-native dialogs (no HTML), egui (no HTML authoring), wry (OS webview wrapper, ~5MB binary).

**Decision:** Use `wry` (Tauri team). Delegates rendering to the OS webview already on the system.

**Consequences:** macOS WebKit (fast, ~30–50MB); Windows WebView2 (pre-installed Win11); Linux WebKitGTK (~100–150MB). Full HTML/CSS/JS supported. No control over exact engine version.

---

## ADR-0002: All UI rendered as HTML — no OS-native widgets

**Status:** Accepted

**Context:** Dialog chrome could be OS-native widgets or HTML-rendered.

**Decision:** Render all chrome (title bar, status bar, buttons) as HTML within the webview.

**Consequences:** Fully themeable. Consistent across types. No OS accessibility APIs for chrome (mitigated by HTML ARIA). Enables rich images and markdown natively.

---

## ADR-0010: Transparent title bar with full-size content view (macOS)

**Status:** Accepted

**Decision:** `with_titlebar_transparent(true)` + `with_fullsize_content_view(true)`. Traffic lights float over HTML. HTML title bar reserves ~72px left safe zone. Window draggable via `-webkit-app-region: drag`.

**Consequences:** Native traffic lights visible. HTML fills the full window. Modal types disable minimize.

---

## ADR-0010a: Full-size content view extended to all platforms

**Status:** Accepted (implementation deferred)

**Decision:** All platforms use full-size content view with no OS title bar.
- **macOS** — transparent title bar + full-size content view; native traffic lights
- **Windows** — `decorations: false` + HTML close/minimize buttons via IPC
- **Linux** — `decorations: false` + HTML close/minimize buttons via IPC

**Consequences:** Consistent immersive look across platforms. HTML-rendered close/minimize on Windows/Linux wired to IPC.

### Phase B platform policy (resolves ADR-0010a vs interim)

ADR-0010a describes the **target** cross-platform chrome. During **Phase B**, Windows and Linux keep **native OS window decorations** (same as Phase A). macOS uses ADR-0010 transparent title bar immediately. ADR-0010a Win/Linux implementation (`decorations: false` + HTML close/minimize, REQ-0085) ships in **Phase C** — not Phase B. Dialog **content** is always HTML; only the outer frame on Win/Linux stays native until Phase C.

---

## ADR-0014: Native file/folder picker via `rfd`

**Status:** Accepted

**Context:** `input` type `mode: file` and `mode: folder` require OS-native pickers. Options: custom GTK/Win32/Cocoa code in `wyvern-window`, or the `rfd` crate (cross-platform native dialogs).

**Decision:**
- Use the **`rfd`** crate **only** in `wyvern-window` for file and folder selection (b.4).
- `wyvern-schema` validates picker-related fields (`filter`, `multiple`, `start_path`) but never depends on `rfd`.
- Selected paths are returned as **plain strings** in `InputResult.input` (single path) or a **JSON array of strings** when `multiple: true` (REQ-0065).

**Headless CI strategy:**
- Test-only env `WYVERN_MOCK_PICKER_PATH` injects a path without showing picker UI when set.
- Linux CI (xvfb): prefer mock injection; tests that require real picker UI may be `#[ignore]` on headless runners.
- Boundary enforcement: `sc-lint` confirms `rfd` appears only in `wyvern-window` dependency graph.

**Consequences:** Small dependency footprint; native look-and-feel per OS. Picker logic is not unit-testable in `wyvern-schema`; integration tests live in `wyvern-window` with mocks.
