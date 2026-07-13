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

- `crates/wyvern-window/src/window.rs` — `apply_platform_chrome`: Win/Linux `with_decorations(false)`
- `crates/wyvern-window/src/chrome/template.html` — Win/Linux window control buttons in title bar
- `crates/wyvern-window/src/chrome/render.rs` — platform conditional control markup
- `crates/wyvern-window/src/ipc/chrome.rs` — new module: `ChromePageIpc` enum, `parse_chrome_page_ipc` (Phase B has no `ipc/` tree; chrome IPC types live here)
- `crates/wyvern-window/src/run.rs` — extend per-dialog `handle_ipc` to dispatch `window_close` / `window_minimize` via `ipc::chrome`
- `crates/wyvern-window/src/lib.rs` — export `ipc` module
- `crates/wyvern-window/tests/` — IPC tests for close/minimize
- `docs/wyvern-window/architecture.md` — mark ADR-0010a implemented for Win/Linux

## Deliverables

- Windows: borderless window; HTML **close** always; HTML **minimize** on non-modal types only
- Linux: same as Windows
- `-webkit-app-region: drag` on title bar (REQ-0087) — already on macOS; extend to Win/Linux title bar
- Close via HTML control → same stdout semantics as OS close (`dismissed` / question REQ-0068 shape)
- Minimize via HTML control → window minimizes (only when window attributes allow — not modal types per REQ-0083)
- macOS unchanged: native traffic lights, no HTML window buttons
- All Phase B dialog types render correctly on Windows and Linux CI legs
- `chrome` command: OS-equivalent close via HTML button → `{"button":"dismissed"}`

## Required Work — platform chrome (authoritative)

### Window attributes (`window.rs`)

```rust
#[cfg(not(target_os = "macos"))]
let attrs = attrs.with_decorations(false);
```

Modal types retain `.with_enabled_buttons(WindowButtons::CLOSE)` — no minimize at winit layer.

### HTML title bar (Win/Linux only)

```html
<div id="title-bar">
  <span id="title-text">{{TITLE}}</span>
  <div id="window-controls" class="no-drag">
    <!-- minimize: hidden for modal dialog types -->
    <button id="btn-minimize" data-action="minimize" aria-label="Minimize">—</button>
    <button id="btn-close" data-action="close" aria-label="Close">×</button>
  </div>
</div>
```

- `#window-controls` and buttons use `-webkit-app-region: no-drag`
- Render layer sets `btn-minimize` `hidden` when modal (message/input/markdown/question)

### IPC (see chrome-ipc-contract.md)

| User action | Page → host | Host behavior |
|-------------|-------------|---------------|
| HTML close | `{ "kind": "window_close" }` | Same as `dismissed` in dialog contract |
| HTML minimize | `{ "kind": "window_minimize" }` | Minimize window; no stdout until dialog completes |

Malformed chrome IPC → same fail-safe as dialog contract (log + dismissed).

## Explicit Code Samples

```rust
// crates/wyvern-window/src/ipc/chrome.rs
#[derive(Debug, PartialEq, Eq)]
pub enum ChromePageIpc {
    WindowClose,
    WindowMinimize,
}

pub fn parse_chrome_page_ipc(raw: &str) -> Option<ChromePageIpc> { /* ... */ }
```

```rust
// run.rs — extend handle_ipc (conceptual)
use crate::ipc::chrome::{parse_chrome_page_ipc, ChromePageIpc};

match parse_page_ipc(raw) {
    // ... existing button_pressed, dismissed, etc.
    None => {
        if let Some(chrome) = parse_chrome_page_ipc(raw) {
            match chrome {
                ChromePageIpc::WindowClose => complete_with_dismissed(),
                ChromePageIpc::WindowMinimize => {
                    window.set_minimized(true);
                    // no CommandResult yet
                }
            }
        } else {
            fail_safe_dismissed();
        }
    }
}
```

```rust
// chrome/render.rs — inject minimize visibility
pub fn render_chrome_html(title: &str, status: Option<&str>, show_minimize: bool) -> String {
    let minimize_btn = if show_minimize {
        r#"<button id="btn-minimize" ...>"#
    } else {
        ""
    };
    // ...
}
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
- Modal types: minimize button absent or inert; REQ-0083 preserved
- Title bar draggable on Win/Linux
- No regression to macOS ADR-0010 behavior

## Required Validation

- `cargo test --workspace -- --test-threads=1` on all three CI OS legs
- Unit tests: IPC `window_close` / `window_minimize` mapping
- `sc-lint check native --config .sc-lint.toml`
- Grep gate: `with_decorations(true)` absent from non-test Win/Linux production paths in `window.rs`
