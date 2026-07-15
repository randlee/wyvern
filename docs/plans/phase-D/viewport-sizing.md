# Viewport sizing policy (Phase D)

Authority for embedded viewer auto-size on **wizard pages** and blocking dialogs. Implements product rule: **fit on screen when possible; ~25% slack is fine; scroll only when content exceeds the display.**

> **HTML-side examples only:** Docs may mention DAG, graph, or Flowise as **illustrative page content** — something you might build in wizard HTML/JS. Wyvern Rust code does **not** integrate Flowise or any graph engine. Host/wizard pass opaque JSON (`config`, `page.layout`); sizing policy is generic (ADR-0006, ADR-0020).

Owned primarily by **d.6**; **d.5** ships HTML examples that exercise workspace layout.

Related: [http-wizard-contract.md](../phase-C/http-wizard-contract.md), REQ-V008, ADR-0020.

---

## Two layout modes (per wizard page)

| Mode | Typical HTML (examples) | Rust knows |
|------|-------------------------|------------|
| **`dialog`** (default) | Forms, cards, summaries | `layout` enum value only |
| **`workspace`** | Large canvas pages (e.g. graph editor HTML) | `layout` enum value only |

Layout is declared per page (preferred) or wizard-wide default in `config`:

```json
{
  "page": { "id": "editor", "title": "Edit", "html": "pages/editor.html", "layout": "workspace" },
  "config": {
    "layout": "dialog",
    "estimated_size": { "width": 1200, "height": 800 }
  }
}
```

- `page.layout` wins for that step; else `config.layout`; else `dialog`.
- Host echoes `layout` and opaque `config` in `GET /api/wizard/state` — **no interpretation** of graph semantics.
- What the HTML page embeds (Flowise, custom canvas, etc.) is entirely page JS.

---

## Dialog layout mode (default)

1. Measure content at natural width (no artificial wrap cap during measure).
2. Apply slack: `w = ceil(contentW × 1.25)`, `h = ceil(contentH × 1.25)` (20–30% oversize acceptable).
3. Clamp to available viewport: `min(sized, viewportAvail × 0.92)` per axis.
4. If content still exceeds clamp → window at clamp size; **`.content` scrolls** (no resize iteration).
5. Viewer **hidden until first valid resize** (or opens at viewport clamp, never 320×240 flash).
6. Remeasure after `document.fonts.ready` + `ResizeObserver` for async assets; viewer accepts refinement IPC for ~300ms.

Blocking dialogs (non-wizard) use the same dialog-mode policy via existing `wyvern-api.js` paths.

---

## Workspace layout mode (wizard pages)

For HTML pages that need a large viewport (example: graph editor). Rust deliverables are **generic only**:

| Source | Priority | Rust behaviour |
|--------|----------|----------------|
| `width` / `height` on wizard command | 1 | Pass through to state JSON |
| `page.layout === "workspace"` | 2 | Pass through |
| `config.estimated_size` (opaque object) | 3 | Pass through unchanged |
| Default for workspace page | — | Fill available viewport |

**Example** `config` shape (illustrative — any extra keys are opaque):

```json
{
  "estimated_size": { "width": 1200, "height": 800 }
}
```

Page JS may read `window.wyvern.config.estimated_size` (or any keys the HTML author chose). Wyvern does not define or parse Flowise-specific fields in Rust.

### CSS (HTML/template)

- Workspace wizard pages use `dialog--workspace` on the page root — fills viewer.
- Pan/zoom/scroll inside the canvas is **page JS**, not host logic.

---

## Viewer ↔ page channel (d.6)

Viewer sends available bounds before first paint. Wizard pages call `WyvernApi.applyWizardLayout(state, viewport)` after `GET /api/wizard/state`:

```javascript
var layout = state.page.layout || state.config.layout || "dialog";
if (layout === "workspace") return applyWorkspaceLayout(state, viewport);
return applyDialogFitWithSlack(measurePage(), viewport, 1.25);
```

---

## Sprint ownership

| Work | Sprint | Rust? |
|------|--------|-------|
| `viewport-sizing.md` | plan hardening | — |
| HTML examples (layout-picker, workspace-hint) | **d.5** | tests only |
| `page.layout`, opaque `config` in state; `wyvern-api.js` sizing; viewer bounds IPC | **d.6** | yes (passthrough + viewer) |
| Golden L2 sizing | **d.6** | — |

---

## Non-goals (Phase D)

- Rust integration with Flowise, Flowwise, or any graph library
- Parsing tool-specific keys in `config` (opaque blob only)
- Separate non-wizard dialog type for canvases
