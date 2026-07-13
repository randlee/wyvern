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

**Status:** Accepted (Win/Linux implementation in Phase C c.3; macOS implemented Phase A)

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

**Headless CI strategy (authoritative):**
- All CI platforms: picker tests set test-only env `WYVERN_MOCK_PICKER_PATH` to a fixture path; `picker.rs` skips `rfd` UI when set.
- Linux CI (xvfb): mock env required for non-ignored picker tests; real picker UI tests are `#[ignore]` when mock unset.
- macOS/Windows CI: same mock env pattern.
- Boundary enforcement: `sc-lint` confirms `rfd` appears only in `wyvern-window` dependency graph.

**Picker UX (Phase B):** file/folder modes use picker-on-OK — page sends `input_submitted` without `value`; host opens `rfd` synchronously. See [ipc-dialog-contract.md](../plans/phase-B/ipc-dialog-contract.md).

**Consequences:** Small dependency footprint; native look-and-feel per OS. Picker logic is not unit-testable in `wyvern-schema`; integration tests live in `wyvern-window` with mocks.

---

## ADR-0015: Built-in icon asset layout (Phase C)

**Status:** Accepted (implementation in Phase C c.1–c.2)

**Context:** Phase B ships four **placeholder** SVGs under `assets/icons/placeholder/` for `MessageLevel` values only (b.2). REQ-0030 requires a full curated bundle with multiple variants per semantic role. REQ-0031 requires named resolution with variant index.

**Decision:**
- Production icons live at `crates/wyvern-window/assets/icons/{role}/{index}.svg` (SVG primary; PNG/WebP allowed per REQ-0030).
- Six roles: `info`, `warning`, `error`, `question`, `success`, `loading` — minimum two variants each.
- Embed via `include_bytes!` in `wyvern-window/src/icons/` — no runtime filesystem access for built-in icons.
- `MessageLevel` maps to the homonymous role's variant 1 at render time.
- Named icon specs validated in `wyvern-schema` against the role catalog; unknown names → validation error (c.2).
- Phase B `assets/icons/placeholder/` retained for regression tests only after c.1 — not used in production render paths.

**Variant syntax:** `"warning"` → variant 1; `"warning:2"` → variant 2 (1-based index).

**Consequences:** Binary size increases — monitor NFR-0003 (< 10MB macOS release). Level icons and free-form `icon` field share one catalog. Post-MVP AI-generated icons remain out of scope (PRD).
