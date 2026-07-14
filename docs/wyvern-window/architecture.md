# `wyvern-window` — Architecture

> **DEPRECATED (c.9+)** — Superseded by [`wyvern-host`](../wyvern-host/architecture.md). Crate **deleted in c.9**. ADRs below are **archival** — see principal [Superseded ADRs](../architecture.md#superseded-adrs-wyvern-window--archival-only).

*Part of the [principal architecture](../architecture.md).*

---

## ADR-0001: Use `wry` as the webview engine

**Status:** Superseded (c.15) — `wry` only in optional `wyvern-viewer`; not in `wyvern-host`.

**Context:** Options: Electron (~150MB), OS-native dialogs (no HTML), egui (no HTML authoring), wry (OS webview wrapper, ~5MB binary).

**Decision:** Use `wry` (Tauri team). Delegates rendering to the OS webview already on the system.

**Consequences:** macOS WebKit (fast, ~30–50MB); Windows WebView2 (pre-installed Win11); Linux WebKitGTK (~100–150MB). Full HTML/CSS/JS supported. No control over exact engine version.

---

## ADR-0002: All UI rendered as HTML — no OS-native widgets

**Status:** Superseded (c.10+) — UI in packaged `share/wyvern/ui/`; host serves bytes only.

**Context:** Dialog chrome could be OS-native widgets or HTML-rendered.

**Decision:** Render all chrome (title bar, status bar, buttons) as HTML within the webview.

**Consequences:** Fully themeable. Consistent across types. No OS accessibility APIs for chrome (mitigated by HTML ARIA). Enables rich images and markdown natively.

---

## ADR-0010: Transparent title bar with full-size content view (macOS)

**Status:** Superseded (HTTP delivery) — platform attrs lifted to `wyvern-viewer` (c.15); templates own safe zones in `ui/`.

**Decision:** `with_titlebar_transparent(true)` + `with_fullsize_content_view(true)`. Traffic lights float over HTML. HTML title bar reserves ~72px left safe zone. Window draggable via `-webkit-app-region: drag`.

**Consequences:** Native traffic lights visible. HTML fills the full window. Modal types disable minimize.

---

## ADR-0010a: Full-size content view extended to all platforms

**Status:** Superseded (HTTP delivery) — Win/Linux/macOS chrome in packaged `ui/` (c.14); optional `wyvern-viewer` platform attrs (c.15). IPC chrome contract is historical.

**Decision:** All platforms use full-size content view with no OS title bar.
- **macOS** — transparent title bar + full-size content view; native traffic lights; HTML title bar reserves **72px left safe zone** (ADR-0010)
- **Windows** — `decorations: false` + HTML close/minimize buttons via IPC; **no** 72px left padding; controls on the right
- **Linux** — `decorations: false` + HTML close/minimize buttons via IPC; same title-bar layout as Windows

**Consequences:** Consistent immersive look across platforms. Chrome IPC parsed in `chrome/ipc.rs` (`parse_chrome_ipc`); dispatched in `run.rs` (MessageApp, InputApp, MarkdownApp, ChromeApp) and `question/handler.rs` (QuestionApp). `decorations: false` is orthogonal to modal `.with_enabled_buttons(WindowButtons::CLOSE)` — the former removes the OS frame; the latter restricts winit chrome buttons when decorations are enabled.

**Modal minimize policy:** HTML minimize button hidden on modal types; host `handle_ipc` must **no-op** `window_minimize` (not dismiss) as defense-in-depth.

**Render API:** `PlatformChrome` struct (`macos_safe_zone`, `show_minimize`, `show_window_controls`) drives template placeholders `{{TITLE_BAR_STYLE}}` and `{{WINDOW_CONTROLS_BLOCK}}` across all dialog and chrome templates.

**ChromeApp:** upgraded in c.3 to `EventLoop` user events + `with_ipc_handler`, matching dialog apps (`window_close` → dismissed; `window_minimize` → `set_minimized`).

### Phase B platform policy (historical)

ADR-0010a was the **target** cross-platform chrome. During **Phase B**, Windows and Linux kept **native OS window decorations**. macOS used ADR-0010 transparent title bar immediately. ADR-0010a Win/Linux (`decorations: false` + HTML close/minimize, REQ-0085) shipped in **Phase C c.3**.

---

## ADR-0014: Native file/folder picker via `rfd`

**Status:** Superseded (HTTP delivery) — `rfd` in **`wyvern-host`** only (c.11); HTTP picker routes (REQ-0113/0114). Historical detail below.

**Former decision (archival):** `rfd` only in `wyvern-window`; picker-on-OK via wry IPC. Replaced by `POST /api/picker/file` and `POST /api/picker/folder`. Mock env `WYVERN_MOCK_PICKER_PATH` may be retained for host picker tests in c.11.

---

## ADR-0015: Built-in icon asset layout (Phase C)

**Status:** Superseded (c.9+) — Rust icon catalog and `include_bytes!` embed removed; icons live in packaged `ui/` ([REQ-0103](../wyvern-host/requirements.md)). Historical for c.1–c.2 embedded stack only.

**Context:** Phase B ships four **placeholder** SVGs under `assets/icons/placeholder/` for `MessageLevel` values only (b.2). REQ-0030 requires a full curated bundle with multiple variants per semantic role. REQ-0031 requires named resolution with variant index.

**Decision:**
- Production icons live at `crates/wyvern-window/assets/icons/{role}/{index}.svg` (SVG primary; PNG/WebP allowed per REQ-0030).
- Six roles: `info`, `warning`, `error`, `question`, `success`, `loading` — minimum two variants each.
- **Role catalog** (`ROLES`, `variant_count`, `parse_icon_spec`) lives in `wyvern-schema/src/icons.rs` — pure logic, no window dependency (ADR-0011).
- **Embed helpers** (`variant_bytes`, `svg_markup`) live in `wyvern-window/src/icons/` via `include_bytes!` — no runtime filesystem access for built-in icons.
- `MessageLevel` maps to the homonymous role's variant 1 at render time.
- Named icon specs validated in `wyvern-schema` against the schema-local role catalog; unknown names → validation error (c.2).
- Phase B `assets/icons/placeholder/` retained for regression tests only after c.1 — not used in production render paths.

**Variant syntax:** `"warning"` → variant 1; `"warning:2"` → variant 2 (1-based index).

**Consequences:** Binary size increases — monitor NFR-0003 (< 10MB macOS release). Level icons and free-form `icon` field share one catalog. Post-MVP AI-generated icons remain out of scope (PRD).
