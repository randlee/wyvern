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
| `GET` | `/wizard/**` | Wizard page HTML + example assets under `--ui-root` (`page.html` paths) |
| `GET` | `/shared/**` | Packaged shared JS/CSS from `ui/shared/` (not `--ui-root`) |
| `GET` | `/api/wizard/state` | `page`, `page_data`, `stack`, `config` |
| `POST` | `/api/wizard/navigate` | Non-terminal: `next`, `back` |
| `POST` | `/api/wizard/finish` | Terminal: `finish`, `cancel`, `dismissed` |
| `GET` | `/api/dialog` | *Not used* for wizard — use `/api/wizard/state` |

**Static routing (normative — dual mount):** `GET /wizard/**` from `--ui-root`; `GET /shared/**` from `HostOptions.shared_ui_root` (packaged `ui/`, not overridden by `--ui-root`).

---

## `GET /api/wizard/state`

**Response (initial load — cursor=0):**

```json
{
  "type": "wizard",
  "config": { "theme": "dark" },
  "page": {
    "id": "start",
    "title": "Start",
    "html": "pages/start.html",
    "layout": "dialog"
  },
  "page_data": {},
  "stack": [],
  "width": 640,
  "height": 480
}
```

**Response (after navigating to step-2 — cursor=1):**

```json
{
  "type": "wizard",
  "config": { "theme": "dark" },
  "page": {
    "id": "step-2",
    "title": "Step 2",
    "html": "pages/step-2.html"
  },
  "page_data": { "choice": "layout-a" },
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

**Workspace page example** (`page.layout: "workspace"` + opaque `config`):

```json
{
  "type": "wizard",
  "config": {
    "estimated_size": { "width": 960, "height": 640 }
  },
  "page": {
    "id": "editor",
    "title": "Canvas",
    "html": "pages/editor.html",
    "layout": "workspace"
  },
  "page_data": {},
  "stack": []
}
```

- `page.layout` — optional `dialog` | `workspace` (omit = `dialog`). Host **passes through** only; sizing is page JS via `WyvernApi.applyWizardLayout` (d.6 / ADR-0020).
- `config` — **opaque** JSON object echoed to `window.wyvern.config`. Host does not interpret keys. Example shape used by workspace pages: `estimated_size: { width, height }` (illustrative; any extra keys remain opaque).
- `stack` — **prior entries only** per REQ-0024 / ADR-0005: `entries[0..cursor]`, exclusive of current page (current via `page` + `page_data`).
- `width` / `height` — optional from command; when both set they take priority for workspace/dialog fixed size (viewer / `applyWorkspaceLayout`).

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

**Opaque data write (normative):** whole-blob replace only — `entries[cursor].data = data`; no deep merge (ADR-0006). See d.2 for forward-same-page overwrite predicate.

**Finish stack (normative — mirrors d.2):**

1. Derive current entry data from request `data` (in-memory; session not mutated after finish).
2. Build session-derived stack = all visited entries `entries[0..=cursor]` (includes current).
3. **`finish`:** stdout `stack` = session-derived; stdout `data` = request `data`; client `stack` if present must match or **400** (`StackMismatch`).
4. **`cancel`:** `stack: []`, `data: {}` always (client `stack` ignored).
5. **`dismissed`:** stdout `stack` = session-derived full visited stack (same as `finish`); stdout `data` = `{}`.

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

**Dismissed (viewer OS-close — d.8):**

Viewer algorithm (normative — full visited stack):

1. `GET /api/wizard/state` → read `page`, `page_data`, and prior `stack` (entries before current).
2. Build full visited stack = prior `stack` + `{ page, data: page_data }` (matches d.2 finish algorithm).
3. `POST /api/wizard/finish` with `{ "button": "dismissed", "data": {}, "stack": <full visited stack> }` before process exit.

```json
{
  "button": "dismissed",
  "data": {},
  "stack": [
    { "page": { "id": "start" }, "data": { "choice": "a" } },
    { "page": { "id": "step-2" }, "data": { "name": "agent" } }
  ]
}
```

Host validates client `stack` against session-derived full visited stack; mismatch → **400**.

**Stdout:** identical body. Host shuts down session (one-shot) or returns to interactive loop (Phase E).

---

## Sprint mapping (Phase D)

| Sprint | HTTP work |
|--------|-------------|
| d.1 | Dual static mount (`/wizard/**`, `/shared/**`); `GET /api/wizard/state` initial load |
| d.2 | `navigate` + `finish` routes; `wyvern-api.js` helpers |
| d.3 | History regression tests (no new routes) |
| d.4 | Bootstrap round-trip tests (no new routes) |
| d.5 | Example wizards exercising HTTP stack |
| d.6 | Viewport sizing (orthogonal) |
| d.7 | Shared wizard chrome (`wizard-nav.js`) |
| d.8 | Viewer dismiss with full visited stack |

## Replaces

- Phase D sprint doc [d2-wizard-ipc.md](../phase-D/d2-wizard-ipc.md) wry `action` messages — **historical only**.
