---
id: c.3
title: Windows and Linux platform chrome
status: pending
branch: feature/phase-C-c3-win-linux-chrome
target: integrate/phase-C
---

# Sprint c.3 — Windows and Linux platform chrome

## Goal

- Implement ADR-0010a on Windows and Linux: `decorations: false` + HTML window controls wired via IPC (REQ-0085, REQ-0086, REQ-0087).
- Close the Phase A/B interim policy (`with_decorations(true)` on non-macOS).

## Hard Dependencies

- Phase B complete (all dialog types render in HTML shell)
- Independent of c.1–c.2 — may merge in parallel after Phase B; icon work does not block window frame

## Exact Targets

- `crates/wyvern-window/src/window.rs` — `apply_platform_chrome`: Win/Linux `with_decorations(false)`; modal types may retain `.with_enabled_buttons(WindowButtons::CLOSE)` for API consistency but it is **inert** when `decorations: false` (no native title bar to expose)
- `crates/wyvern-window/src/chrome/platform.rs` — new: `PlatformChrome` struct (`macos_safe_zone`, `show_minimize`, `show_window_controls`)
- `crates/wyvern-window/src/chrome/ipc.rs` — new: `pub(crate) enum ChromeIpc`, `pub(crate) fn parse_chrome_ipc` (shared by `run.rs` dialog apps and `question/handler.rs` — avoids QuestionApp ↔ run module cycle)
- `crates/wyvern-window/src/chrome/render.rs` — accept `PlatformChrome`; inject `{{WINDOW_CONTROLS_BLOCK}}` and `{{TITLE_BAR_STYLE}}`
- `crates/wyvern-window/src/chrome/template.html` — Win/Linux window control buttons; `<script>` block wiring `#window-controls` clicks → `window.ipc.postMessage`
- `crates/wyvern-window/src/message/template.html`, `input/template.html`, `markdown/template.html`, `question/template.html` — same placeholders; remove hard-coded `padding-left: 72px` from inline `<style>`; add or extend `<script>` for `#window-controls` IPC (same pattern as chrome template)
- `crates/wyvern-window/src/markdown/styles.css` — remove hard-coded `#title-bar { padding-left: 72px; }`; title-bar padding driven only via `{{TITLE_BAR_STYLE}}` injected at render time
- `crates/wyvern-window/src/message/render.rs`, `input/render.rs`, `markdown/render.rs`, `question/render.rs`, `chrome/render.rs` — pass `PlatformChrome` from `platform_chrome_for(command_type)`
- `crates/wyvern-window/src/run.rs` — **ChromeApp upgrade** (see below); import `parse_chrome_ipc` from `chrome::ipc`; extend MessageApp/InputApp/MarkdownApp `handle_ipc` for `window_close` / `window_minimize`
- `crates/wyvern-window/src/question/handler.rs` — import `parse_chrome_ipc` from `chrome::ipc`; extend `QuestionApp::handle_ipc` (QuestionApp lives here, not in `run.rs`, to break the module cycle)
- `crates/wyvern-window/tests/` — IPC tests for close/minimize; modal minimize no-op; ChromeApp `WYVERN_INJECT_IPC` integration test; render/fixture tests assert `#window-controls` click wiring and title-bar style per platform cfg
- `docs/wyvern-window/architecture.md` — mark ADR-0010a implemented for Win/Linux

## Deliverables

- Windows: borderless window; HTML **close** always; HTML **minimize** on non-modal types only
- Linux: same as Windows
- `-webkit-app-region: drag` on title bar (REQ-0087) — already on macOS; extend to Win/Linux title bar
- **Platform-conditional title bar:** macOS keeps 72px left safe zone (ADR-0010); Win/Linux **no** 72px left padding — title left-aligned, `#window-controls` on the **right**
- Close via HTML control → same stdout semantics as OS close (`dismissed` / question REQ-0068 shape)
- Minimize via HTML control → window minimizes (non-modal only); modal `handle_ipc` **no-ops** `window_minimize` (must not fall through to dismiss)
- macOS unchanged: native traffic lights, no HTML window buttons
- All Phase B dialog types render correctly on Windows and Linux CI legs
- `chrome` command: OS-equivalent close via HTML button → `{"button":"dismissed"}`

## Required Work — platform chrome (authoritative)

### Window attributes (`window.rs`)

```rust
#[cfg(not(target_os = "macos"))]
let attrs = attrs.with_decorations(false);
// Modal types may retain .with_enabled_buttons(WindowButtons::CLOSE) for parity with macOS cfg —
// inert on Win/Linux when decorations: false (no native title bar to expose).
```

Modal types may retain `.with_enabled_buttons(WindowButtons::CLOSE)` for API parity with macOS. On Win/Linux with `decorations: false`, `enabled_buttons` is **inert** — there is no native title bar. Minimize is blocked at the HTML layer (`show_minimize = false`) and host layer (`window_minimize` no-op).

### PlatformChrome render API

All dialog/chrome render entry points accept `PlatformChrome`:

```rust
// crates/wyvern-window/src/chrome/platform.rs
pub struct PlatformChrome {
    /// macOS only: reserve 72px left padding for traffic lights (ADR-0010).
    pub macos_safe_zone: bool,
    /// Win/Linux non-modal: show HTML minimize button.
    pub show_minimize: bool,
    /// Win/Linux: render HTML close/minimize block in title bar.
    pub show_window_controls: bool,
}

pub fn platform_chrome_for(command: CommandKind) -> PlatformChrome {
    #[cfg(target_os = "macos")]
    { PlatformChrome { macos_safe_zone: true, show_minimize: false, show_window_controls: false } }
    #[cfg(not(target_os = "macos"))]
    {
        let modal = matches!(command, CommandKind::Message | CommandKind::Input | CommandKind::Markdown | CommandKind::Question);
        PlatformChrome {
            macos_safe_zone: false,
            show_minimize: !modal,
            show_window_controls: true,
        }
    }
}
```

Template placeholders (all dialog + chrome templates):

| Placeholder | macOS | Win/Linux |
|-------------|-------|-----------|
| `{{TITLE_BAR_STYLE}}` | `padding-left: 72px;` | *(empty or `padding-left: 0;`)* |
| `{{WINDOW_CONTROLS_BLOCK}}` | *(empty)* | close + optional minimize buttons |

```html
<div id="title-bar" style="{{TITLE_BAR_STYLE}}">
  <span id="title-text">{{TITLE}}</span>
  {{WINDOW_CONTROLS_BLOCK}}
</div>
```

Win/Linux controls block:

```html
<div id="window-controls" class="no-drag">
  <!-- minimize omitted when show_minimize false -->
  <button id="btn-minimize" data-action="minimize" aria-label="Minimize">—</button>
  <button id="btn-close" data-action="close" aria-label="Close">×</button>
</div>
```

- `#window-controls` and buttons use `-webkit-app-region: no-drag`
- Render layer omits `#btn-minimize` when `show_minimize` is false

### Window control JS wiring (authoritative)

Each template that renders `#window-controls` must include a `<script>` block (alongside existing button-bar / input handlers) that posts chrome IPC via the same `window.ipc.postMessage` bridge as dialog buttons:

```html
<script>
  (function () {
    var controls = document.getElementById("window-controls");
    if (!controls) return;
    controls.addEventListener("click", function (ev) {
      var btn = ev.target.closest("button[data-action]");
      if (!btn) return;
      var action = btn.getAttribute("data-action");
      if (action === "close") {
        window.ipc.postMessage(JSON.stringify({ kind: "window_close" }));
      } else if (action === "minimize") {
        window.ipc.postMessage(JSON.stringify({ kind: "window_minimize" }));
      }
    });
  })();
</script>
```

Apply to: `chrome/template.html` (always on Win/Linux), and dialog templates when `{{WINDOW_CONTROLS_BLOCK}}` is non-empty. macOS renders empty `{{WINDOW_CONTROLS_BLOCK}}` — script is harmless no-op.

**Render/fixture tests:** assert rendered HTML for Win/Linux cfg includes `#window-controls`, `data-action="close"`, and (non-modal only) `data-action="minimize"`; assert macOS fixture has no `#window-controls`. Extend markdown render test to confirm `styles.css` title bar has **no** hard-coded `padding-left: 72px` and receives macOS padding only via `{{TITLE_BAR_STYLE}}`.

### IPC (see chrome-ipc-contract.md)

| User action | Page → host | Host behavior |
|-------------|-------------|---------------|
| HTML close | `{ "kind": "window_close" }` | Same as `dismissed` in dialog contract |
| HTML minimize | `{ "kind": "window_minimize" }` | Minimize window; no stdout — **no-op on modal types** |

Malformed chrome IPC → same fail-safe as dialog contract (log + dismissed).

### ChromeApp upgrade (blocking — Phase B gap)

Phase B `ChromeApp` has **no IPC handler** (no `with_ipc_handler`, no `DialogEvent` loop). c.3 must upgrade `run_chrome` / `ChromeApp` to match dialog apps:

```rust
// run.rs — ChromeApp must gain (same pattern as MessageApp):
struct ChromeApp {
    // ... existing fields ...
    proxy: EventLoopProxy<DialogEvent>,
    inject_ipc: Option<String>,
    pending_inject: bool,
}

fn run_chrome(...) -> Result<CommandResult, RunError> {
    let event_loop = EventLoop::<DialogEvent>::with_user_event()
        .build()
        .map_err(/* ... */)?;
    let proxy = event_loop.create_proxy();
    let inject_ipc = std::env::var(INJECT_IPC_ENV).ok();
    // ...
}

// resumed: WebViewBuilder::new(&window)
//     .with_html(self.html.clone())
//     .with_ipc_handler(move |req| { proxy.send_event(DialogEvent::Ipc(req.body().clone())); })
//     .build()

impl ChromeApp {
    fn handle_ipc(&mut self, event_loop: &ActiveEventLoop, raw: &str) {
        if let Some(msg) = parse_chrome_ipc(raw) {
            match msg {
                ChromeIpc::WindowClose => self.dismiss(event_loop),
                ChromeIpc::WindowMinimize => {
                    if let Some(window) = &self.window {
                        window.set_minimized(true);
                    }
                    // no CommandResult yet
                }
            }
            return;
        }
        // malformed → fail-safe dismissed
        eprintln!("wyvern-window: malformed chrome IPC; dismissing: {raw}");
        self.dismiss(event_loop);
    }
}
```

`parse_chrome_ipc` lives in `chrome/ipc.rs` (`pub(crate)`) — shared import for `run.rs` dialog apps and `question/handler.rs`. No top-level `src/ipc/` module tree; chrome IPC types stay under `chrome/`.

### Modal minimize no-op (all dialog apps)

Every modal `handle_ipc` (MessageApp, InputApp, MarkdownApp in `run.rs`; QuestionApp in `question/handler.rs`) must handle `window_minimize` as an explicit **no-op** before the malformed-IPC fail-safe:

```rust
fn handle_ipc(&mut self, event_loop: &ActiveEventLoop, raw: &str) {
    if parse_chrome_ipc(raw) == Some(ChromeIpc::WindowMinimize) {
        return; // modal: ignore — must NOT dismiss
    }
    if parse_chrome_ipc(raw) == Some(ChromeIpc::WindowClose) {
        self.dismiss(event_loop);
        return;
    }
    // ... existing dialog IPC ...
}
```

## Explicit Code Samples

```rust
// chrome/render.rs
pub fn render_chrome_html(title: &str, status: Option<&str>, chrome: PlatformChrome) -> String {
    let title_bar_style = if chrome.macos_safe_zone { "padding-left: 72px;" } else { "" };
    let controls = if chrome.show_window_controls {
        render_window_controls(chrome.show_minimize)
    } else {
        String::new()
    };
    CHROME_HTML
        .replace("{{TITLE}}", &escape_html_text(title))
        .replace("{{TITLE_BAR_STYLE}}", title_bar_style)
        .replace("{{WINDOW_CONTROLS_BLOCK}}", &controls)
        .replace("{{STATUS_BLOCK}}", &status_block)
}
```

```rust
// chrome/ipc.rs — shared parse helper (pub(crate), under chrome/)
pub(crate) enum ChromeIpc { WindowClose, WindowMinimize }

pub(crate) fn parse_chrome_ipc(raw: &str) -> Option<ChromeIpc> {
    let v: serde_json::Value = serde_json::from_str(raw).ok()?;
    match v.get("kind")?.as_str()? {
        "window_close" => Some(ChromeIpc::WindowClose),
        "window_minimize" => Some(ChromeIpc::WindowMinimize),
        _ => None,
    }
}

// run.rs / question/handler.rs — import and use in handle_ipc:
use crate::chrome::ipc::{parse_chrome_ipc, ChromeIpc};
```

## This Sprint Does Not Close

- macOS HTML close/minimize (uses native traffic lights — by design)
- Wizard minimize policy — Phase D (wizard enables minimize per REQ-0084)
- NFR benchmarking — c.4
- Release workflow — c.5

## Acceptance Criteria

- Windows CI: `cargo test --workspace` passes; window attribute tests assert `decorations(false)` on Win/Linux cfg
- Linux CI (xvfb): same
- HTML close on modal dialog → correct `CommandResult` (message button mapping unchanged)
- HTML close on `chrome` → `{"button":"dismissed"}`
- HTML minimize on `chrome` → window minimizes without stdout
- **ChromeApp:** `WYVERN_INJECT_IPC='{"kind":"window_close"}'` integration test completes with `{"button":"dismissed"}`
- Modal types: `window_minimize` IPC → no stdout, no dismiss; minimize button absent in HTML
- Win/Linux render tests: title bar has **no** `padding-left: 72px` (including markdown `styles.css` fixture); `#window-controls` present on right with `data-action` attributes
- macOS render tests: `padding-left: 72px` preserved via `{{TITLE_BAR_STYLE}}` only; no `#window-controls`
- Render/fixture test: `#window-controls` click wiring present in template HTML for Win/Linux cfg
- Title bar draggable on Win/Linux
- No regression to macOS ADR-0010 behavior

## Required Validation

- `cargo test --workspace -- --test-threads=1` on all three CI OS legs
- Unit tests: IPC `window_close` / `window_minimize` mapping; modal `window_minimize` no-op
- Integration test: ChromeApp + `WYVERN_INJECT_IPC` for `window_close`
- Render tests: `PlatformChrome` title-bar style and controls block per platform cfg; markdown `styles.css` has no hard-coded 72px title-bar padding
- Render/fixture tests: `#window-controls` IPC wiring in template `<script>` blocks
- `sc-lint check native --config .sc-lint.toml`
- Grep gate: `with_decorations(true)` absent from non-test Win/Linux production paths in `window.rs`
