# `wyvern-wizard` — Requirements

*Part of the [principal requirements](../requirements.md).*

---

## Navigation (REQ-0020 – REQ-0025)

**REQ-0020** — Maintain a browser-history model: a cursor over an array of visited pages.

**REQ-0021** — Back navigation moves the cursor back without discarding forward history.

**REQ-0022** — Forward navigation to the same explicit next-page descriptor as the cached entry restores that page's previously collected data.

**REQ-0023** — Forward navigation to a different explicit next-page descriptor truncates all entries after the cursor and pushes the new page.

**REQ-0024** — On page load, expose `{ "page": {}, "page_data": {}, "stack": [] }` to the page. On the HTTP host path, `GET /api/wizard/state` returns this payload; `ui/shared/wyvern-api.js` sets `window.wyvern` from it. `page` is the current page descriptor; `stack` contains prior entries as `{ "page": {}, "data": {} }`; `page_data` is the restored opaque data for the current page.

**REQ-0025** — Pages signal navigation using explicit page descriptors. On HTTP: `POST /api/wizard/navigate` accepts `next` (with `data` + `next` descriptor) and `back` (with optional `data`); terminal `finish`, `cancel`, and `dismissed` use `POST /api/wizard/finish` only. Host treats page-specific `data` as opaque (NFR-0008).

---

## Page Descriptor (REQ-0026 – REQ-0027)

**REQ-0026** — Every wizard page has a minimal descriptor with `id`, `title`, and `html`. `id` is a stable page identity; `html` may be a relative or absolute path.

**REQ-0027** — HTTP navigation payloads (authoritative after Phase D):

- `POST /api/wizard/navigate` — `back` → `{ "action": "back", "data": {} }`; `next` → `{ "action": "next", "data": {}, "next": { "id", "title", "html" } }`
- `POST /api/wizard/finish` — terminal only: `{ "button": "finish|cancel|dismissed", "data": {}, "stack": [...] }`

See [http-wizard-contract.md](../plans/phase-C/http-wizard-contract.md).

*Historical (wyvern-window IPC — deleted c.9):* wry `action` messages included `finish` and `cancel` on the navigate channel. Do not implement on the HTTP path.

---

## History Model Example

```
A → B → C        history: [A, B, C]  cursor=2
back             history: [A, B, C]  cursor=1
back             history: [A, B, C]  cursor=0
→ B (same)       history: [A, B, C]  cursor=1  (B's data restored)
→ C (same)       history: [A, B, C]  cursor=2  (C's data restored)

back to A        history: [A, B, C]  cursor=0
→ D (different)  history: [A, D]     cursor=1  (B, C truncated)
```
