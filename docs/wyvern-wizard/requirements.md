# `wyvern-wizard` — Requirements

*Part of the [principal requirements](../requirements.md).*

---

## Navigation (REQ-0020 – REQ-0025)

**REQ-0020** — Maintain a browser-history model: a cursor over an array of visited pages.

**REQ-0021** — Back navigation moves the cursor back without discarding forward history.

**REQ-0022** — Forward navigation to the same explicit next-page descriptor as the cached entry restores that page's previously collected data.

**REQ-0023** — Forward navigation to a different explicit next-page descriptor truncates all entries after the cursor and pushes the new page.

**REQ-0024** — On page load, inject `{ "page": {}, "page_data": {}, "stack": [] }` into the page via IPC, where `page` is the current page descriptor and `stack` contains all prior page entries as `{ "page": {}, "data": {} }`.

**REQ-0025** — Pages signal navigation via IPC using explicit page descriptors. `back` and `finish` include the current page descriptor plus opaque `data`; `next` also includes an explicit `next` page descriptor. Host treats page-specific `data` as opaque.

---

## Page Descriptor (REQ-0026 – REQ-0027)

**REQ-0026** — Every wizard page has a minimal descriptor with `id`, `title`, and `html`. `id` is a stable page identity; `html` may be a relative or absolute path.

**REQ-0027** — A minimal page-directed navigation payload is:
- `back` → `{ "action": "back", "page": { "id": "...", "title": "...", "html": "..." }, "data": {} }`
- `next` → `{ "action": "next", "page": { ... }, "data": {}, "next": { "id": "...", "title": "...", "html": "..." } }`
- `finish` → `{ "action": "finish", "page": { ... }, "data": {} }`
- `cancel` → `{ "action": "cancel" }`

**HTTP amendment (Phase D):** On the HTTP host path, `POST /api/wizard/navigate` accepts **`next`** and **`back` only**. Terminal **`cancel`**, **`finish`**, and **`dismissed`** use **`POST /api/wizard/finish`** — see [http-wizard-contract.md](../plans/phase-C/http-wizard-contract.md).

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
