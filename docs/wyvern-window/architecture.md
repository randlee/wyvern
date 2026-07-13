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

**Status:** Accepted

**Decision:** All platforms use full-size content view with no OS title bar.
- **macOS** — transparent title bar + full-size content view; native traffic lights
- **Windows** — `decorations: false` + HTML close/minimize buttons via IPC
- **Linux** — `decorations: false` + HTML close/minimize buttons via IPC

**Consequences:** Consistent immersive look across platforms. HTML-rendered close/minimize on Windows/Linux wired to IPC.
