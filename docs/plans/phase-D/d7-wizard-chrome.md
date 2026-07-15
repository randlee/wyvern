---
id: d.7
title: Shared wizard chrome
status: planning
branch: feature/phase-D-d7-wizard-chrome
target: integrate/phase-D
---

# Sprint d.7 — Shared wizard chrome

## Goal

Reusable back/next/finish chrome for wizard pages and N=1 edge case. Page JS only — no new stack logic.

## Hard dependencies

- **d.6** merged

## Deliverables

| File | Purpose |
|------|---------|
| `ui/shared/wizard-nav.js` | Back/next/finish button wiring via `wyvern-api.js` |
| `ui/wizard/chrome.html` | Optional wrapper template for wizard pages (nav bar slot) |
| `examples/wizards/single-page/wizard.json` | N=1 fixture |
| `examples/wizards/single-page/pages/only.html` | Single page with `data-wizard-terminal="true"` |
| `crates/wyvern-wizard/tests/single_page.rs` | N=1 snapshot + navigate/finish paths |
| `tests/l2/wizard-edge-cases.spec.ts` | L2 chrome edge cases — first-page back hidden, N=1, empty data |

**Terminal page contract (normative):** page root MUST set `data-wizard-terminal="true"` when it is the last step. `wizard-nav.js` reads this attribute only.

```html
<div data-wizard-terminal="true">…</div>
```

**UX rules (`wizard-nav.js`, not host):**

| Condition | Behaviour |
|-----------|-----------|
| First page (cursor=0, empty `stack`) | Back button `hidden` or `disabled` |
| `data-wizard-terminal="true"` | Next label → `"Finish"`; click calls `wyvernWizardFinish` |
| Single-page wizard (N=1) | Back hidden; sole page shows Finish immediately |
| Empty `data` on submit | Treat as `{}`; no `undefined` access in helpers |

Pages opt in: `<script src="/shared/wizard-nav.js" data-wizard-chrome></script>`

## Acceptance criteria

1. First page: back button hidden or disabled
2. Last page (`data-wizard-terminal="true"`): next button label changes to "Finish"
3. Empty `data` on a page handled gracefully (no undefined errors in console)
4. Wizard with a single page (N=1) works correctly end-to-end

## Required validation

```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
cargo test -p wyvern-wizard single_page
npx playwright test tests/l2/wizard-edge-cases.spec.ts
sc-lint check native --config .sc-lint.toml
```

## Non-closure

- Viewer dismiss (d.8)

## Authority

- [http-wizard-contract.md](../phase-C/http-wizard-contract.md)
- ADR-0006
