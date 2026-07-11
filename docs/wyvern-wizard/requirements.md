# `wyvern-wizard` — Requirements

*Part of the [principal requirements](../requirements.md).*

---

## Navigation (REQ-0020 – REQ-0025)

**REQ-0020** — Maintain a browser-history model: a cursor over an array of visited pages.

**REQ-0021** — Back navigation moves the cursor back without discarding forward history.

**REQ-0022** — Forward navigation to the same next-page as the cached entry restores that page's previously collected data.

**REQ-0023** — Forward navigation to a different next-page truncates all entries after the cursor and pushes the new page.

**REQ-0024** — On page load, inject `{ "page_data": {}, "stack": [] }` into the page via IPC, where `stack` contains all prior pages' `{ id, data }` entries.

**REQ-0025** — Pages signal navigation via IPC: `{ "action": "next|back|finish|cancel", "button": "label", "data": {} }`. Host treats `data` as opaque.

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
