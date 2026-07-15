# Viewport sizing policy (Phase D)

Authority for embedded viewer auto-size on **wizard pages** and blocking dialogs. Implements product rule: **fit on screen when possible; ~25% slack is fine; scroll only when content exceeds the display.**

**DAG / graph / Flowise UIs are wizard pages** — same `type: wizard` session, `/wizard/**` HTML, `/api/wizard/*` routes. Not a separate dialog type. A wizard may alternate form steps and full graph-editor steps in one flow.

Owned primarily by **d.6**; **d.5** proves DAG + workspace hints inside wizard examples.

Related: [http-wizard-contract.md](../phase-C/http-wizard-contract.md), [http-viewer-contract.md](../phase-C/http-viewer-contract.md), REQ-V008, ADR-0020.

---

## Two layout modes (per wizard page)

| Mode | Wizard use | Blocking dialog use |
|------|------------|-------------------|
| **`dialog`** (default) | Form steps, cards, summaries | message, input, markdown, question |
| **`workspace`** | DAG/graph/Flowise canvas pages | — (wizard only in Phase D) |

Layout is declared per page (preferred) or wizard-wide default in `config`:

```json
{
  "page": { "id": "flow-editor", "title": "Edit flow", "html": "pages/graph.html", "layout": "workspace" },
  "config": { "layout": "dialog", "flowise": { "estimated_width": 1200, "estimated_height": 800 } }
}
```

- `page.layout` wins for that step; else `config.layout`; else `dialog`.
- Host exposes `layout` and hints in `GET /api/wizard/state` (d.6); host does not interpret graph semantics (ADR-0006).
- Graph page JS may embed Flowise (or similar); navigation out uses `wyvernWizardNext` / `wyvernWizardFinish` like any wizard page.

---

## Dialog layout mode (default)

1. Measure content at natural width (no artificial wrap cap during measure).
2. Apply slack: `w = ceil(contentW × 1.25)`, `h = ceil(contentH × 1.25)` (20–30% oversize acceptable).
3. Clamp to available viewport: `min(sized, viewportAvail × 0.92)` per axis.
4. If content still exceeds clamp → window at clamp size; **`.content` scrolls** (no resize iteration).
5. Viewer **hidden until first valid resize** (or opens at viewport clamp, never 320×240 flash).
6. Remeasure after `document.fonts.ready` + `ResizeObserver` for async assets; viewer accepts refinement IPC for ~300ms.

**No fixed pixel tiers** (no “always 480px”). Works on laptop and 8K — caps are viewport-relative.

Blocking dialogs (non-wizard) use the same dialog-mode policy via existing `wyvern-api.js` paths.

---

## Workspace layout mode (wizard graph pages)

For DAG/graph/Flowise steps inside a wizard — full-screen or tool-estimated:

| Source | Priority |
|--------|----------|
| Explicit `width` / `height` on wizard command | 1 — session default if both set |
| `page.layout === "workspace"` + page or `config.estimated_size` | 2 |
| `config.flowise` / `config.flowwise` size hints | 3 — normalized in page bootstrap |
| Default for workspace page | Fill available viewport (work area) |

### `estimated_size` wire shape (authoritative)

```json
{
  "type": "wizard",
  "page": {
    "id": "flow-editor",
    "title": "Edit flow",
    "html": "pages/graph.html",
    "layout": "workspace"
  },
  "config": {
    "layout": "dialog",
    "estimated_size": { "width": 1200, "height": 800 },
    "flowise": {
      "estimated_width": 1200,
      "estimated_height": 800
    }
  }
}
```

- Flowise may emit `estimated_width` / `estimated_height` when it knows graph bounds; page or agent places them in wizard `config`.
- Wyvern normalizes to `estimated_size` in `window.wyvern` bootstrap (`wyvern-api.js`); host passes through opaque JSON.
- Small graph: use hints when provided; clamp to viewport.
- Large graph: hints may exceed viewport → window at viewport max; **page** handles pan/zoom/scroll (Flowise canvas).

### CSS

- Wizard graph pages use `dialog--workspace` on the page root — fills viewer; internal canvas controls sizing.
- Dismiss/finish/stack behaviour unchanged — still wizard HTTP contract.

---

## Viewer ↔ page channel

**d.6 deliverable:** viewer sends available bounds before first paint (IPC `bounds:WxH` or query `?viewport=WxH`).

Wizard pages call `WyvernApi.applyWizardLayout(state, viewport)` (wraps sizing policy):

```javascript
// Pseudocode — wyvern-api.js (d.6); called after GET /api/wizard/state
function layoutForWizardPage(state, viewport) {
  var layout = state.page.layout || state.config.layout || "dialog";
  if (layout === "workspace") return applyWorkspaceLayout(state, viewport);
  return applyDialogFitWithSlack(measurePage(), viewport, 1.25);
}
```

---

## Sprint ownership

| Work | Sprint |
|------|--------|
| `viewport-sizing.md` (this doc) | plan hardening |
| Wizard examples: layout-picker (form DAG) + workspace-hint (graph page) | **d.5** |
| `page.layout`, state passthrough, `wyvern-api.js`, viewer bounds IPC | **d.6** |
| `http-wizard-contract.md` — wizard page `layout`, Flowise hints | **d.6** |
| Golden L2: form wizard step + graph wizard step in one flow | **d.5–d.6** |

---

## Non-goals (Phase D)

- Separate `type: flowise` or non-wizard host session for graphs
- Rust-side `CHAR_W` heuristics
- Live Flowise server API (hint JSON + embedded page only)
