---
id: d.6
title: Wizard polish and edge cases
status: planning
branch: feature/phase-D-d6-wizard-polish
target: integrate/phase-D
---

# Sprint d.6 — Wizard polish and edge cases

## Goal

Close wizard UX edge cases and viewer-dismiss integration for wizard pages.

## Hard dependencies

- **d.5** merged

## Deliverables

- First-page back button hidden/disabled in wizard templates
- Last-page next label → "Finish"
- Single-page wizard (N=1) path
- Viewer close dismiss — `POST /api/wizard/finish` with `{ "button": "dismissed", ... }` per [http-viewer-contract.md](../phase-C/http-viewer-contract.md)
- L2 regression: layout-picker + edge-case specs

## Acceptance criteria

1. `cargo build --workspace` + `cargo clippy --workspace -- -D warnings` green
2. First page: back button hidden or disabled
3. Last page: next button label changes to "Finish"
4. Empty `data` on a page handled gracefully (no undefined errors)
5. Wizard with a single page (N=1) works correctly
6. Viewer close on any wizard page returns `{"button":"dismissed","stack":[...]}` via `POST /api/wizard/finish` (aligned with d.2 route — not `navigate`)

## Required validation

```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
cargo test -p wyvern-host wizard_polish
cargo test -p wyvern-wizard
# L2: wizard edge cases --viewer none
```

## Non-closure

- `--interactive` wizard loops (Phase E)
- MCP wizard tools (Phase E e.3, after d.2)

## Authority

- [http-wizard-contract.md](../phase-C/http-wizard-contract.md)
- [http-viewer-contract.md](../phase-C/http-viewer-contract.md) — viewer dismiss protocol
