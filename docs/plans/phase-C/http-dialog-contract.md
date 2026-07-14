# HTTP Dialog Contract (Phase C c.10+)

Authoritative contract between packaged UI (any HTTP client) and `wyvern-host`. Replaces [Phase B wry IPC](../phase-B/ipc-dialog-contract.md) for all new work.

**Rust types:** [HTTP-TYPES.md](HTTP-TYPES.md).

## Transport

- **Mechanism:** HTTP/1.1 on loopback (or configured bind)
- **Encoding:** JSON for API routes; static files served with correct `Content-Type`
- **Session:** one dialog per CLI invocation (one-shot). Single in-process session — **no** session id in URL; `/{type}/` selects the template tree only.

## Routes

| Method | Path | Purpose |
|--------|------|---------|
| `GET` | `/` | Redirect or 404; type entry is `/{type}/` |
| `GET` | `/{type}/` | `index.html` from `{ui_root}/{type}/` |
| `GET` | `/{type}/*` | Static assets relative to type tree |
| `GET` | `/api/dialog` | JSON payload for active command |
| `POST` | `/api/result` | Page submits final stdout-shaped result |
| `POST` | `/api/picker/file` | Host opens `rfd` file picker (c.11 — `input`) |
| `POST` | `/api/picker/folder` | Host opens `rfd` folder picker (c.11 — `input`) |

**Dialog URL (default):** `http://127.0.0.1:{port}/message/` for `type: message`.

## `GET /api/dialog`

Response body: JSON object — validated command fields needed by the template plus:

```json
{
  "type": "message",
  "title": "T",
  "message": "Hi",
  "buttons": "ok",
  "level": "warning"
}
```

- Host copies from `Command` enum; no HTML rendering.
- Unknown-to-template fields are ignored by the page.

## `POST /api/result`

**Authoritative per-type bodies:** [http-post-schema.md](http-post-schema.md).

Summary: POST body **equals** stdout JSON (no `kind` wrapper). Host deserializes to `CommandResult` for the active dialog `type`.

## Dismissed

**Browser / headless (`none`):** template uses `sendBeacon` or sync POST on `beforeunload` when possible.

**`wyvern-viewer` OS close:** viewer POSTs dismissed to host before exit; CLI watches child exit as fallback — [http-viewer-contract.md](http-viewer-contract.md#viewer-close--host-dismiss-req-0097).

If host receives nothing, map to `button: "dismissed"` (or question extended shape per REQ-0068).

## UI root layout

```text
share/wyvern/ui/
  message/
    index.html
    app.js
    style.css
    icons/
  input/
  markdown/
  question/
  chrome/
```

**Custom root:** same structure; user replaces files freely.

## Security

- Default bind `127.0.0.1` only.
- `0.0.0.0` requires explicit CLI flag + documented warning.
- Static handler must reject path traversal (`..`).

## CI / tests

- **CI / agents:** `--viewer none` or `WYVERN_VIEWER=none` — no native windows in automated runs.
- **Product default:** `embedded` (c.15) — not used in CI matrix.
- **L1:** `reqwest` / in-process HTTP — `cargo test -p wyvern-host`.
- **L2:** Headless **Playwright** or **Puppeteer** — load dialog URL, click controls, assert stdout. See [c9-testing-headless.md](c9-testing-headless.md).
- **Dev:** Cursor integrated browser MCP — open logged URL for debugging without popups.
- Package UI must expose stable `data-testid` on buttons for L2.
- Host sets `WYVERN_DIALOG_URL` when `--viewer none` so e2e can attach without parsing stderr.

**Viewer contract:** [http-viewer-contract.md](http-viewer-contract.md).
