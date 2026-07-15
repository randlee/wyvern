---
id: d.6
title: Wizard polish, viewport sizing, and workspace layout
status: planning
branch: feature/phase-D-d6-wizard-polish
target: integrate/phase-D
---

# Sprint d.6 — Wizard polish, viewport sizing, and workspace layout

## Goal

UX edge cases, viewer dismiss, and viewport sizing. **Orthogonal to the stack** — see [viewport-sizing.md](viewport-sizing.md).

## Hard dependencies

- **d.5** merged (includes `workspace-hint` example + hint wire shape)

## Deliverables

### Viewport sizing (`ui/shared/wyvern-api.js` + `wyvern-viewer`)

| File | Change |
|------|--------|
| `ui/shared/wyvern-api.js` | `applySizingPolicy()`, slack (~1.25×), viewport clamp, `applyWorkspaceLayout()` |
| `ui/shared/embedded-chrome.css` | `dialog--workspace` styles |
| `crates/wyvern-viewer/src/run.rs` | Hidden until first resize; viewport bounds IPC to page; multi-resize refinement window |
| `crates/wyvern-viewer/src/platform.rs` | Document bootstrap policy (no 320×240 visible flash) |
| `docs/plans/phase-C/http-wizard-contract.md` | `page.layout`, opaque `config` passthrough (incl. example `estimated_size` shape) |
| `tests/l2/viewport-sizing.spec.ts` | **new** — golden dialog fit + workspace hint cases |

**Dialog mode (default):**

- Measure at natural width; apply ~25% slack; clamp to viewer-reported viewport × 0.92.
- Measure before `visibility: visible`; remeasure on `fonts.ready` + `ResizeObserver`.
- Overflow → `.content { overflow: auto }` inside clamped window.

**Workspace mode (`page.layout === "workspace"`):**

- Generic wizard page layout — Rust passes `layout` and opaque `config` only.
- `wyvern-api.js` applies viewport sizing; page HTML decides canvas content.
- No tool-specific parsing in Rust (ADR-0006).

### Shared wizard chrome (`ui/wizard/` — new packaged templates)

| File | Purpose |
|------|---------|
| `ui/wizard/chrome.html` | Optional wrapper template for wizard pages (nav bar slot) |
| `ui/wizard/wizard-nav.js` | Back/next/finish button wiring via `wyvern-api.js` |

**UX rules (implemented in `wizard-nav.js`, not host):**

| Condition | Behaviour |
|-----------|-----------|
| First page (cursor=0, no prior stack entries) | Back button `hidden` or `disabled` |
| Last page (page signals `isTerminal` via data attribute or JS convention) | Next label → `"Finish"`; click calls `wyvernWizardFinish` not `wyvernWizardNext` |
| Single-page wizard (N=1) | Back hidden; sole page shows Finish immediately |
| Empty `data` on submit | Treat as `{}`; no `undefined` access in helpers |

Pages may opt in: `<script src="/shared/wizard-nav.js" data-wizard-chrome></script>` — paths follow existing `ui/` static layout.

### Viewer dismiss (`wyvern-viewer` + host)

| File | Change |
|------|--------|
| `crates/wyvern-viewer/src/dismiss.rs` (or session handler) | Detect wizard session (`GET /api/wizard/state` reachable or URL path `/wizard/`); on OS-close POST **`/api/wizard/finish`** not `/api/result` |
| `crates/wyvern-viewer/tests/wizard_dismiss.rs` | **new** — viewer routes wizard dismiss correctly |

- Viewer OS-close on wizard session → viewer posts `POST /api/wizard/finish` with `{ "button": "dismissed", "data": {}, "stack": <current stack from state> }` per [http-viewer-contract.md](../phase-C/http-viewer-contract.md)
- Host accepts `dismissed` on finish route (d.2); d.6 adds viewer wiring + stack passthrough
- `crates/wyvern-host/tests/wizard_polish.rs` — **new**
- `crates/wyvern-wizard/tests/single_page.rs` — **new** — N=1 snapshot + navigate/finish paths

### L2 regression

- `tests/l2/wizard-edge-cases.spec.ts` — first-page back hidden, N=1, empty data, dismissed
- `tests/l2/viewport-sizing.spec.ts` — dialog slack fit + workspace-layout example

## Acceptance criteria

1. `cargo build --workspace` + `cargo clippy --workspace -- -D warnings` green
2. First page: back button hidden or disabled
3. Last page: next button label changes to "Finish"
4. Empty `data` on a page handled gracefully (no undefined errors in console)
5. Wizard with a single page (N=1) works correctly end-to-end
6. Viewer close on any wizard page returns `{"button":"dismissed","stack":[...]}` via `POST /api/wizard/finish` (not `navigate`)
7. Layout-picker example still passes full smoke from d.5
8. Dialog auto-size: representative message/input payloads fit on first open with slack (golden L2, no manual resize)
9. Workspace-hint example: `page.layout: "workspace"` + opaque `estimated_size` honored (viewport-clamped)
10. Viewer does not flash at 320×240 before first content-sized resize

## Required validation

```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
cargo test -p wyvern-host wizard_polish
cargo test -p wyvern-wizard single_page
cargo test -p wyvern-viewer viewport_bounds
# L2: wizard edge cases + viewport sizing
npx playwright test tests/l2/wizard-edge-cases.spec.ts tests/l2/viewport-sizing.spec.ts
```

## Non-closure

- `--interactive` wizard loops (Phase E)
- MCP wizard tools (Phase E e.3, after d.2)
- Tool-specific integrations (Flowise, etc.) — HTML author concern only

## Authority

- [viewport-sizing.md](viewport-sizing.md)
- [http-wizard-contract.md](../phase-C/http-wizard-contract.md)
- [http-viewer-contract.md](../phase-C/http-viewer-contract.md) — viewer dismiss protocol
- REQ-0066 (`dismissed` button), REQ-V008 (auto-size amendment)
- ADR-0020