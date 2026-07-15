# `wyvern-viewer` — Requirements

*Part of the [principal requirements](../requirements.md).*

**Status:** Active (c.15). Presentation-only client for HTTP dialogs.

---

## Embedded viewer (REQ-0097 surface)

**REQ-V001** — `wyvern-viewer` opens an OS-native webview that loads a single dialog URL supplied via argv or `WYVERN_DIALOG_URL`.

**REQ-V002** — Default policy rejects non-loopback hosts and non-`http`/`https` schemes. Opt-in: `WYVERN_VIEWER_ALLOW_NON_LOOPBACK=1` (mirrors host `--allow-non-loopback`).

**REQ-V003** — On OS window close without a prior successful button POST from the page, the viewer POSTs `{ "button": "dismissed" }` to `/api/result` with bounded connect/read/write timeouts (best-effort; failures are logged, not fatal).

**REQ-V004** — The viewer does not embed dialog HTML, run an HTTP server, or speak wry dialog IPC. Host + packaged `ui/` own presentation content.

**REQ-V005** — Viewer is Wyvern's alternate **browser shell**. macOS keeps **native traffic lights only** (red/yellow/green); no surrounding OS title frame — transparent titlebar + full-bleed content; HTML/CSS in `ui/` owns all other look and feel. Win/Linux use undecorated frames with HTML chrome. Non-modal, user-resizable, opens in front; user may alt-tab away.

**REQ-V006** — Process may run for an extended session (interactive/MCP, multi-step wizards). Supports in-process navigation via IPC `navigate:<url>` without exiting; one-shot dialogs may still close the window after submit.

**REQ-V007** — Embedded-only presentation helpers (`embedded-chrome.css`, `window.ipc`) must not be required for correctness — the same pages must work in `--viewer system` / named browsers (browser-first gate).

**REQ-V008** — When no explicit size is set (JSON or HTML meta), embedded viewer **auto-sizes** to rendered page content via IPC `resize:WxH`. Compact dialogs (`dialog--compact`) shrink to message-box proportions (~480px max width with word-wrap); full pages (wizard, chrome) measure document content up to viewer max (800×600).

**REQ-V009** — Optional window size on any command JSON (`width`, `height` in CSS pixels, 200–800 × 96–600). When **both** are set, embedded viewer uses that fixed size **instead of** auto-size. Pages may also declare `<meta name="wyvern:width">` / `<meta name="wyvern:height">` when JSON omits size. JSON from `GET /api/dialog` takes precedence over meta tags.

**Authority:** [http-viewer-contract.md](../plans/phase-C/http-viewer-contract.md), [http-interactive-mcp-contract.md](../plans/phase-C/http-interactive-mcp-contract.md), [http-wizard-contract.md](../plans/phase-C/http-wizard-contract.md), [docs/wyvern-host/requirements.md](../wyvern-host/requirements.md) (REQ-0097).
