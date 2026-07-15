# HTTP POST schemas (page JavaScript → `wyvern-host`)

Authoritative wire shapes for **page → host** POST bodies. These align with [`wyvern-schema` `CommandResult`](../../crates/wyvern-schema/src/result.rs) stdout JSON — **the POST body for a completed dialog is the same object the CLI prints**.

Related: [http-dialog-contract.md](http-dialog-contract.md) (routes), Phase B [ipc-dialog-contract.md](../phase-B/ipc-dialog-contract.md) (historical `kind` field — **not used** on HTTP path).

---

## Conventions

| Rule | Value |
|------|--------|
| `Content-Type` | `application/json` |
| Charset | UTF-8 |
| Discriminator | Active dialog `type` from `GET /api/dialog` — host knows expected result shape |
| Extra fields | Unknown keys → **400** validation error (mirror REQ-0053) |
| Success response | `200` `{ "ok": true }` — host then completes `run()` and exits |

**No `kind` wrapper** on simple dialogs (c.10+). POST body **is** the stdout result object.

**Rust types:** [HTTP-TYPES.md](HTTP-TYPES.md).

---

## Phase C — blocking dialogs

### `POST /api/result` — `message`

**When:** User clicks a button or dismisses.

```json
{
  "button": "ok"
}
```

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `button` | string | yes | Preset label (`ok`, `cancel`, `yes`, `no`, …) or custom label from `custom_buttons`; `"dismissed"` on force-close |

**Example — cancel:**

```json
{ "button": "cancel" }
```

**Example — dismissed (OS close / `beforeunload`):**

```json
{ "button": "dismissed" }
```

**Stdout:** identical body.

---

### `POST /api/result` — `markdown`

Same shape as `message`:

```json
{ "button": "ok" }
```

---

### `POST /api/result` — `chrome`

Same shape as `message`:

```json
{ "button": "dismissed" }
```

---

### `POST /api/result` — `input`

**When:** User confirms or cancels text input, or after file/folder picker completes (see picker routes below).

**Text mode — OK:**

```json
{
  "button": "ok",
  "input": "user text"
}
```

**Text mode — cancel:**

```json
{ "button": "cancel" }
```

**File single path — OK** (after picker):

```json
{
  "button": "ok",
  "input": "/path/to/file.json"
}
```

**File multiple — OK:**

```json
{
  "button": "ok",
  "input": ["/path/a.json", "/path/b.json"]
}
```

**Folder — OK:**

```json
{
  "button": "ok",
  "input": "/path/to/dir"
}
```

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `button` | string | yes | |
| `input` | string \| string[] | no | Omit on `cancel` / `dismissed`. String = text or single path; array = multi-file only |

**Dismissed:**

```json
{ "button": "dismissed" }
```

---

### `POST /api/picker/file` — `input` helper (c.11)

**When:** Page needs native file picker (`mode: file`). Not a final result — returns paths for the page to include in `/api/result`.

**Request:**

```json
{
  "filter": ["json", "txt"],
  "multiple": false,
  "start_path": "/optional/dir"
}
```

All fields optional; host merges with dialog fields from `GET /api/dialog`.

**Response `200`:**

```json
{
  "ok": true,
  "paths": ["/selected/file.json"]
}
```

**Response `200` — user cancelled picker:**

```json
{
  "ok": false,
  "cancelled": true
}
```

Page stays open; user may retry or press Cancel → `POST /api/result` with `{ "button": "cancel" }`.

---

### `POST /api/picker/folder` — `input` helper (c.11)

**Request:**

```json
{
  "start_path": "/optional/dir"
}
```

**Response `200`:**

```json
{
  "ok": true,
  "paths": ["/selected/dir"]
}
```

---

### `POST /api/result` — `question`

**When:** User clicks Submit (normal completion).

```json
{
  "questions": [
    {
      "question": "Output format?",
      "header": "Format",
      "options": [
        { "label": "JSON", "description": "Structured" },
        { "label": "Plain", "description": "Text only" }
      ],
      "multiSelect": false
    }
  ],
  "answers": {
    "Output format?": "JSON"
  },
  "response": ""
}
```

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `questions` | array | yes | **Verbatim** echo of input `questions` from `/api/dialog` |
| `answers` | object | yes | Keys = each card's `question` string; values = selected `label` or comma-joined labels |
| `response` | string | yes | Always `""` on normal submit (REQ-0067) |
| `button` | string | **must omit** | Presence → host treats as fail-safe dismiss |

**Dismissed / fail-safe (REQ-0068):**

```json
{
  "button": "dismissed",
  "questions": [ ... ],
  "answers": {},
  "response": ""
}
```

Host rejects empty `answers` on submit without `button` → respond with fail-safe dismiss shape above.

---

## Shared JS helper (recommended)

Ship in `ui/shared/wyvern-api.js` (c.10+):

```javascript
/** POST final result; body === stdout JSON. */
export async function postResult(body) {
  const res = await fetch("/api/result", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  if (!res.ok) throw new Error(await res.text());
  return res.json(); // { ok: true }
}
```

**Dismiss on unload:**

```javascript
window.addEventListener("beforeunload", () => {
  navigator.sendBeacon(
    "/api/result",
    new Blob([JSON.stringify({ button: "dismissed" })], {
      type: "application/json",
    })
  );
});
```

Question templates use the REQ-0068 extended dismiss shape in `sendBeacon`.

---

## Phase D — `wizard` (separate contract)

Full spec: **[http-wizard-contract.md](http-wizard-contract.md)** (Phase D). Summary below.

Wizard uses **navigation + finish** routes — not a single `POST /api/result` per page click.

### Host → page (`GET /api/wizard/state`)

Initial load (`cursor=0`):

```json
{
  "type": "wizard",
  "page": { "id": "start", "title": "...", "html": "pages/start.html" },
  "page_data": {},
  "stack": []
}
```

After navigate to step-2 (`cursor=1`):

```json
{
  "type": "wizard",
  "page": { "id": "step-2", "title": "...", "html": "pages/step-2.html" },
  "page_data": { "choice": "layout-a" },
  "stack": [
    { "page": { "id": "start", ... }, "data": { "choice": "a" } }
  ]
}
```

`stack` = prior entries only (REQ-0024); current page via `page` + `page_data`.

### Page → host — navigation (non-terminal)

`POST /api/wizard/navigate`

```json
{
  "action": "next",
  "page_id": "step-2",
  "data": { "choice": "layout-a" },
  "next": { "id": "step-2", "title": "Step 2", "html": "pages/step-2.html" }
}
```

| `action` | Meaning |
|----------|---------|
| `next` | Advance history cursor; push new page when branching (requires `next` descriptor for DAG) |
| `back` | Move cursor back **without truncating** forward history (ADR-0005); host serves prior page |

Terminal outcomes (`finish`, `cancel`, `dismissed`) use **`POST /api/wizard/finish` only** — not `navigate`.

**Response `200`:** `{ "ok": true, "url": "http://127.0.0.1:port/wizard/pages/step-2.html" }` or host reload instruction.

### Page → host — finish (terminal)

`POST /api/wizard/finish`

```json
{
  "button": "finish",
  "data": { "final": "values" },
  "stack": [
    { "page": { "id": "start" }, "data": { "choice": "a" } }
  ]
}
```

| Field | Type | Required |
|-------|------|----------|
| `button` | `"finish"` \| `"cancel"` \| `"dismissed"` | yes |
| `data` | object | yes (may be `{}`) |
| `stack` | array | yes |

**Stdout:** same object. Full wizard HTTP contract is owned by Phase D [d2-wizard-ipc.md](../phase-D/d2-wizard-ipc.md) migration — update that sprint to HTTP when D starts.

---

## Validation ownership

| Layer | Responsibility |
|-------|----------------|
| Page JS | UX validation (all questions answered, non-empty text, etc.) |
| `wyvern-host` | JSON parse, required fields, type-specific rules (e.g. question omit `button` on submit), map to `CommandResult` |
| `wyvern-schema` | Already validated CLI input; host trusts `/api/dialog` source |

---

## Sprint ownership

| Schema | Sprint |
|--------|--------|
| `message` | c.10 |
| `input` + picker routes | c.11 |
| `markdown` | c.12 |
| `question` | c.13 |
| `chrome` | c.14 |
| `wizard` navigate/finish | Phase D — [http-wizard-contract.md](http-wizard-contract.md) |
| Interactive / MCP | Phase E — [http-interactive-mcp-contract.md](http-interactive-mcp-contract.md) |

**Rust types:** [HTTP-TYPES.md](HTTP-TYPES.md).

## Cross-links

- [http-dialog-contract.md](http-dialog-contract.md)
- [c9-testing-headless.md](c9-testing-headless.md) — e2e asserts POST → stdout match
- [../../wyvern-schema/requirements.md](../../wyvern-schema/requirements.md) — REQ-0064–0068
