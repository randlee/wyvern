# HTTP Wizard contract (Phase D)

Wizard runs on the same **`wyvern-host`** HTTP server as blocking dialogs. No wry IPC. Page JS uses `fetch` + shared [wyvern-api.js](../../ui/shared/wyvern-api.js) helpers.

**Prerequisite:** Phase C c.16 complete (`wyvern-host`, packaged `ui/`, optional `wyvern-viewer`).

**Rust types:** [HTTP-TYPES.md](HTTP-TYPES.md) (`WizardStateResponse`, `WizardNavigateRequest`, `WizardFinishRequest`).

Related: [http-post-schema.md](http-post-schema.md), [http-dialog-contract.md](http-dialog-contract.md), [HTTP-TYPES.md](HTTP-TYPES.md), ADR-0005 (history model in `wyvern-wizard`).

---

## Session model

- One wizard invocation = one host session (same as one-shot dialogs).
- Host owns navigation state via **`wyvern-wizard`** (pure logic) — **`wyvern-host → wyvern-wizard`** dependency allowed from d.1 per ADR-0011 amendment and `boundaries/wyvern-host/host.toml`.
- Each page is **real HTML** under `ui_root` or command `page.html` path — served over HTTP, not embedded.

---

## Routes

| Method | Path | Purpose |
|--------|------|---------|
| `GET` | `/wizard/` or `/wizard/{page_id}/` | Current wizard page HTML |
| `GET` | `/api/wizard/state` | `page`, `page_data`, `stack`, `config` |
| `POST` | `/api/wizard/navigate` | Non-terminal: `next`, `back` |
| `POST` | `/api/wizard/finish` | Terminal: `finish`, `cancel`, `dismissed` |
| `GET` | `/api/dialog` | *Not used* for wizard — use `/api/wizard/state` |

Static assets: `GET /wizard/**` maps under wizard HTML directory from command `page.html` paths.

---

## `GET /api/wizard/state`

**Response:**

```json
{
  "type": "wizard",
  "config": { "theme": "dark" },
  "page": {
    "id": "start",
    "title": "Start",
    "html": "pages/start.html"
  },
  "page_data": {},
  "stack": [
    {
      "page": { "id": "start", "title": "Start", "html": "pages/start.html" },
      "data": { "choice": "layout-a" }
    }
  ],
  "width": 640,
  "height": 480
}
```

- `config` → available to JS as `window.wyvern.config` (set in page bootstrap from this payload).
- `stack` — full history per ADR-0005 / REQ-0024.
- `width` / `height` — optional from command; viewer uses when `--viewer embedded`.

---

## `POST /api/wizard/navigate`

**Request — next:**

```json
{
  "action": "next",
  "page_id": "step-2",
  "data": { "choice": "layout-a" },
  "next": { "id": "step-2", "title": "Step 2", "html": "pages/step-2.html" }
}
```

| Field | Required | Notes |
|-------|----------|-------|
| `action` | yes | `"next"` or `"back"` only — **`cancel` is invalid here** (use `/finish`) |
| `data` | on `next` | Opaque page payload host stores in history |
| `next` | on `next` | Page descriptor when branching (DAG) |
| `page_id` | optional | Validation hint |

**Request — back:**

```json
{
  "action": "back",
  "data": {}
}
```

**Response `200`:**

```json
{
  "ok": true,
  "url": "http://127.0.0.1:PORT/wizard/pages/step-2.html"
}
```

Host updates history cursor; viewer navigates to `url` (or full page reload).

**Invalid:** `POST /api/wizard/navigate` with `"action": "cancel"` → **400**. Terminal `cancel` / `finish` / `dismissed` use **`POST /api/wizard/finish` only** — see [http-post-schema.md](http-post-schema.md).

---

## `POST /api/wizard/finish` (terminal)

**Finish:**

```json
{
  "button": "finish",
  "data": { "final": "values" },
  "stack": [
    { "page": { "id": "start" }, "data": { "choice": "a" } }
  ]
}
```

**Cancel:**

```json
{
  "button": "cancel",
  "data": {},
  "stack": []
}
```

**Dismissed:**

```json
{
  "button": "dismissed",
  "data": {},
  "stack": []
}
```

**Stdout:** identical body. Host shuts down session (one-shot) or returns to interactive loop (Phase E).

---

## Sprint mapping (Phase D)

| Sprint | HTTP work |
|--------|-------------|
| d.1 | Serve wizard HTML paths; `GET /api/wizard/state` initial load |
| d.2 | `navigate` + `finish` routes; replace d2 IPC acceptance with this doc |
| d.3–d.6 | History, stack inject, DAG example, polish — all via HTTP |

## Replaces

- Phase D sprint doc [d2-wizard-ipc.md](d2-wizard-ipc.md) wry `action` messages — **historical only**.
