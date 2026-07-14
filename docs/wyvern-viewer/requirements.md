# `wyvern-viewer` — Requirements

*Part of the [principal requirements](../requirements.md).*

**Status:** Active (c.15). Presentation-only client for HTTP dialogs.

---

## Embedded viewer (REQ-0097 surface)

**REQ-V001** — `wyvern-viewer` opens an OS-native webview that loads a single dialog URL supplied via argv or `WYVERN_DIALOG_URL`.

**REQ-V002** — Default policy rejects non-loopback hosts and non-`http`/`https` schemes. Opt-in: `WYVERN_VIEWER_ALLOW_NON_LOOPBACK=1` (mirrors host `--allow-non-loopback`).

**REQ-V003** — On OS window close without a prior successful button POST from the page, the viewer POSTs `{ "button": "dismissed" }` to `/api/result` with bounded connect/read/write timeouts (best-effort; failures are logged, not fatal).

**REQ-V004** — The viewer does not embed dialog HTML, run an HTTP server, or speak wry dialog IPC. Host + packaged `ui/` own presentation content.

**Authority:** [http-viewer-contract.md](../plans/phase-C/http-viewer-contract.md), [docs/wyvern-host/requirements.md](../wyvern-host/requirements.md) (REQ-0097).
