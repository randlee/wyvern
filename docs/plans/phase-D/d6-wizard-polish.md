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

d.6 has **three independent closure tracks**. Each track has its own deliverables, acceptance criteria, and validation — a track may not claim sprint success unless all of its AC pass.

## Hard dependencies

- **d.5** merged (includes `workspace-hint` example + hint wire shape)

---

## Track A — Viewport sizing (`ui/shared/wyvern-api.js` + `wyvern-viewer`)

### Deliverables

| File | Change |
|------|--------|
| `ui/shared/wyvern-api.js` | Canonical API: `WyvernApi.applyWizardLayout(state, viewport)`, `applyWorkspaceLayout(state, viewport)`, `applyDialogFitWithSlack(measure, viewport, slack)` |
| `ui/shared/embedded-chrome.css` | `dialog--workspace` styles |
| `crates/wyvern-viewer/src/run.rs` | Hidden until first resize; viewport bounds IPC to page; multi-resize refinement window |
| `crates/wyvern-viewer/src/platform.rs` | Document bootstrap policy (no 320×240 visible flash) |
| `crates/wyvern-viewer/tests/viewport_bounds.rs` | **new** — viewport bounds IPC tests |
| `docs/plans/phase-C/http-wizard-contract.md` | `page.layout`, opaque `config` passthrough (incl. example `estimated_size` shape) |
| `tests/l2/viewport-sizing.spec.ts` | Golden dialog fit + workspace hint cases |

**Dialog mode (default):** measure at natural width; ~25% slack; clamp to viewer viewport × 0.92; scroll overflow inside clamped window.

**Workspace mode (`page.layout === "workspace"`):** generic sizing via `applyWorkspaceLayout`; Rust passes `layout` and opaque `config` only (ADR-0006).

### Acceptance criteria (Track A)

1. Dialog auto-size: representative message/input payloads fit on first open with slack (golden L2, no manual resize)
2. Workspace-hint example: `page.layout: "workspace"` + opaque `estimated_size` honored (viewport-clamped rendering)
3. Viewer does not flash at 320×240 before first content-sized resize

### Required validation (Track A)

```bash
cargo test -p wyvern-viewer viewport_bounds
npx playwright test tests/l2/viewport-sizing.spec.ts
```

---

## Track B — Shared wizard chrome (`ui/shared/wizard-nav.js`)

### Deliverables

| File | Purpose |
|------|---------|
| `ui/shared/wizard-nav.js` | Back/next/finish button wiring via `wyvern-api.js` |
| `ui/wizard/chrome.html` | Optional wrapper template for wizard pages (nav bar slot) |

**Terminal page contract (normative):** page root MUST set `data-wizard-terminal="true"` when it is the last step. `wizard-nav.js` reads this attribute only — no other `isTerminal` convention.

```html
<!-- finish.html -->
<div data-wizard-terminal="true">
  …
</div>
```

**UX rules (implemented in `wizard-nav.js`, not host):**

| Condition | Behaviour |
|-----------|-----------|
| First page (cursor=0, empty `stack`) | Back button `hidden` or `disabled` |
| `data-wizard-terminal="true"` on page root | Next label → `"Finish"`; click calls `wyvernWizardFinish` |
| Single-page wizard (N=1) | Back hidden; sole page shows Finish immediately |
| Empty `data` on submit | Treat as `{}`; no `undefined` access in helpers |

Pages opt in: `<script src="/shared/wizard-nav.js" data-wizard-chrome></script>` — file lives at `ui/shared/wizard-nav.js`.

### Acceptance criteria (Track B)

1. First page: back button hidden or disabled
2. Last page (`data-wizard-terminal="true"`): next button label changes to "Finish"
3. Empty `data` on a page handled gracefully (no undefined errors in console)
4. Wizard with a single page (N=1) works correctly end-to-end

### Required validation (Track B)

```bash
cargo test -p wyvern-wizard single_page
npx playwright test tests/l2/wizard-edge-cases.spec.ts --grep "chrome|first-page|single-page|empty-data"
```

---

## Track C — Viewer dismiss (`wyvern-viewer` + host)

### Deliverables

| File | Change |
|------|--------|
| `crates/wyvern-viewer/src/dismiss.rs` (or session handler) | Detect wizard session; on OS-close POST `/api/wizard/finish` |
| `crates/wyvern-viewer/tests/wizard_dismiss.rs` | Viewer routes wizard dismiss correctly |
| `crates/wyvern-host/tests/wizard_polish.rs` | Host accepts dismissed finish with stack |

**Dismissed algorithm (normative):**

1. Viewer detects wizard session (`GET /api/wizard/state` succeeds or URL path `/wizard/`)
2. `GET /api/wizard/state` → read current `stack` (prior entries per REQ-0024)
3. `POST /api/wizard/finish` with `{ "button": "dismissed", "data": {}, "stack": <from state> }`
4. Host validates stack against session; stdout = same body

### Acceptance criteria (Track C)

1. Viewer close on any wizard page returns `{"button":"dismissed","stack":[...]}` via `POST /api/wizard/finish` (not `navigate`)
2. Layout-picker example still passes full smoke from d.5

### Required validation (Track C)

```bash
cargo test -p wyvern-host wizard_polish
cargo test -p wyvern-viewer wizard_dismiss
```

---

## Sprint-wide gates

### Acceptance criteria (all tracks)

1. `cargo build --workspace` + `cargo clippy --workspace -- -D warnings` green
2. Tracks A, B, and C each pass their AC above
3. `sc-lint check native --config .sc-lint.toml` passes workspace-wide

### Required validation (sprint closure)

```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
sc-lint check native --config .sc-lint.toml
# Track A
cargo test -p wyvern-viewer viewport_bounds
npx playwright test tests/l2/viewport-sizing.spec.ts
# Track B
cargo test -p wyvern-wizard single_page
npx playwright test tests/l2/wizard-edge-cases.spec.ts
# Track C
cargo test -p wyvern-host wizard_polish
cargo test -p wyvern-viewer wizard_dismiss
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
