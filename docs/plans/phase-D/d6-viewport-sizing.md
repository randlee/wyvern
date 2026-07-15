---
id: d.6
title: Wizard viewport sizing
status: planning
branch: feature/phase-D-d6-viewport-sizing
target: integrate/phase-D
---

# Sprint d.6 — Wizard viewport sizing

## Goal

Zero-touch viewport sizing for wizard pages. **Orthogonal to the stack** — see [viewport-sizing.md](viewport-sizing.md).

## Hard dependencies

- **d.5** merged (includes `workspace-hint` example + hint wire shape)

## Deliverables

| File | Change |
|------|--------|
| `ui/shared/wyvern-api.js` | Canonical API: `WyvernApi.applyWizardLayout(state, viewport)`, `applyWorkspaceLayout(state, viewport)`, `applyDialogFitWithSlack(measure, viewport, slack)` |
| `ui/shared/embedded-chrome.css` | `dialog--workspace` styles |
| `crates/wyvern-viewer/src/run.rs` | Hidden until first resize; viewport bounds IPC to page; multi-resize refinement window |
| `crates/wyvern-viewer/src/platform.rs` | Document bootstrap policy (no 320×240 visible flash) |
| `crates/wyvern-viewer/tests/viewport_bounds.rs` | Viewport bounds IPC golden payload tests |
| `docs/plans/phase-C/http-wizard-contract.md` | `page.layout`, opaque `config` passthrough (incl. example `estimated_size` shape) |
| `tests/l2/viewport-sizing.spec.ts` | Golden dialog fit + workspace hint cases |

**Dialog mode (default):** measure at natural width; ~25% slack; clamp to viewer viewport × 0.92; scroll overflow inside clamped window.

**Workspace mode (`page.layout === "workspace"`):** generic sizing via `applyWorkspaceLayout`; Rust passes `layout` and opaque `config` only (ADR-0006).

### Viewport bounds IPC (normative wire)

Before first paint, `wyvern-viewer` injects bounds into the page:

```javascript
// wyvern-viewer → page (eval before show)
window.dispatchEvent(new CustomEvent("wyvern:viewport-bounds", {
  detail: { available_width: 1920, available_height: 1080 }
}));
```

`wyvern-api.js` listens and passes `viewport` to `WyvernApi.applyWizardLayout(state, viewport)`.

Golden assert in `viewport_bounds.rs`: payload matches `{ available_width, available_height }` (u32, non-zero).

## Acceptance criteria

1. Dialog auto-size: representative message/input payloads fit on first open with slack (golden L2, no manual resize)
2. Workspace-hint example: `page.layout: "workspace"` + opaque `estimated_size` honored (viewport-clamped rendering)
3. Viewer does not flash at 320×240 before first content-sized resize
4. `wyvern:viewport-bounds` event delivered before first visible paint

## Required validation

```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
cargo test -p wyvern-viewer viewport_bounds
npx playwright test tests/l2/viewport-sizing.spec.ts
sc-lint check native --config .sc-lint.toml
```

## Non-closure

- Shared wizard chrome (d.7), viewer dismiss (d.8)

## Authority

- [viewport-sizing.md](viewport-sizing.md)
- [http-wizard-contract.md](../phase-C/http-wizard-contract.md)
- REQ-V008 (auto-size amendment), ADR-0020
