# Viewport sizing policy (Phase D)

Authority for embedded viewer auto-size and wizard **workspace** layouts. Implements product rule: **fit on screen when possible; ~25% slack is fine; scroll only when content exceeds the display.**

Applies to blocking dialogs (all types) and wizard pages. Owned primarily by **d.6**; **d.5** proves workspace + external size hints.

Related: [http-viewer-contract.md](../phase-C/http-viewer-contract.md), REQ-V008, ADR-0020 (principal `docs/architecture.md`).

---

## Two layout modes

| Mode | When | Sizing behaviour |
|------|------|------------------|
| **`dialog`** (default) | message, input, markdown, question, wizard form steps | Intrinsic DOM measure + slack → viewport clamp → internal scroll if needed |
| **`workspace`** | DAG/graph editors (Flowise, Flowwise-style), full canvas pages | Use available viewport **or** author/tool size hints; no compact-dialog caps |

Pages declare mode via JSON (see below). Host passes hints through opaque `config`; host does not interpret graph semantics (ADR-0006).

---

## Dialog mode (default)

1. Measure content at natural width (no artificial wrap cap during measure).
2. Apply slack: `w = ceil(contentW × 1.25)`, `h = ceil(contentH × 1.25)` (20–30% oversize acceptable).
3. Clamp to available viewport: `min(sized, viewportAvail × 0.92)` per axis.
4. If content still exceeds clamp → window at clamp size; **`.content` scrolls** (no resize iteration).
5. Viewer **hidden until first valid resize** (or opens at viewport clamp, never 320×240 flash).
6. Remeasure after `document.fonts.ready` + `ResizeObserver` for async assets; viewer accepts refinement IPC for ~300ms.

**No fixed pixel tiers** (no “always 480px”). Works on laptop and 8K — caps are viewport-relative.

---

## Workspace mode

For graph/DAG UIs that may be full-screen or tool-estimated:

| Source | Priority |
|--------|----------|
| Explicit `width` / `height` on command (wizard or dialog JSON) | 1 — use if both set |
| `config.layout === "workspace"` + `config.estimated_size` | 2 — see wire shape below |
| `config.flowise` / `config.flowwise` size hints (optional) | 3 — same shape as `estimated_size` |
| Default | Fill available viewport (work area) |

### `estimated_size` wire shape (authoritative)

```json
{
  "config": {
    "layout": "workspace",
    "estimated_size": { "width": 1200, "height": 800 },
    "flowise": {
      "estimated_width": 1200,
      "estimated_height": 800
    }
  }
}
```

- Flowise (or similar) may emit `estimated_width` / `estimated_height` when it knows graph bounds.
- Wyvern normalizes to `estimated_size` in page bootstrap (`wyvern-api.js`); host does not validate graph math.
- Small workspace: use hints when provided; clamp to viewport like dialog mode.
- Large workspace: hints may exceed viewport → window at viewport max; page handles pan/zoom/scroll internally.

### CSS

- Template class: `dialog--workspace` — `width/height: 100%` of viewer, internal canvas scroll/pan.
- Full-screen Flowise embed: page fills viewer; no `measurePage()` compact path.

---

## Viewer ↔ page channel

**d.6 deliverable:** viewer sends available bounds before first paint (IPC `bounds:WxH` or query `?viewport=WxH`).

Page uses bounds in `WyvernApi.applySizingPolicy(payload, viewport)`:

```javascript
// Pseudocode — implemented in wyvern-api.js (d.6)
applySizingPolicy(payload, viewport) {
  if (payload.config?.layout === "workspace") return applyWorkspaceLayout(payload, viewport);
  return applyDialogFitWithSlack(measurePage(), viewport, SLACK = 1.25);
}
```

---

## Sprint ownership

| Work | Sprint |
|------|--------|
| `viewport-sizing.md` policy (this doc) | plan hardening |
| Workspace example page + Flowise-shaped `config` in layout-picker or sibling example | **d.5** |
| `wyvern-api.js` slack + viewport clamp; viewer hidden-until-sized; `dialog--workspace`; bounds IPC | **d.6** |
| Golden sizing tests (dialog fit + workspace hint) | **d.6** |
| Optional `layout` / `estimated_size` documented in `http-wizard-contract.md` | **d.6** |

---

## Non-goals (Phase D)

- Rust-side `CHAR_W` heuristics (deleted with `wyvern-window`)
- Third-party JS text-metrics libraries
- Flowise runtime integration (only **size hint** wire shape; tool emits JSON Wyvern consumes)
