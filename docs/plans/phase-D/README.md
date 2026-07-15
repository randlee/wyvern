# Phase D — Wizard (`integrate/phase-D`)

Phase D implementation PRs target **`integrate/phase-D`**. Sprint docs (`d1`–`d6`) are the authority for deliverables and validation.

## Core model (read this first)

**The wizard is browser-style stack state management.** Nothing more in Rust.

```
entries: [{ page, data }, ...]   // visited pages + opaque data per step
cursor: usize                    // current index (like browser history)

next(data, next_page)  → push or truncate-forward-then-push; advance cursor
back()                 → cursor-- (forward entries kept)
snapshot()             → { page, page_data, stack, config }  // for GET /api/wizard/state
finish(button, …)      → terminal result; session ends
```

| Layer | Responsibility |
|-------|----------------|
| **`wyvern-wizard`** | `WizardSession` — stack + cursor (ADR-0005). Pure logic. ~1 module. |
| **`wyvern-host`** | HTTP: serve HTML, call `WizardSession`, return JSON |
| **Page HTML/JS** | Branching, forms, graphs — opaque `data`; picks `next` page descriptor |

Host does not interpret `data`. Pages do not touch the cursor. **DAG branching is page JS**, not Rust.

> **Examples only:** Sample HTML (layout-picker, canvas pages) illustrates usage — not extra Rust subsystems.

## Code baseline

Post-**c.16**: `wyvern-host`, packaged `ui/`, no `wyvern-window`. `integrate/phase-D` from that baseline.

## Phase goal

Multi-page wizards: navigate, back, restore data, return final `stack` JSON.

## Phase acceptance (smoke)

`examples/wizards/layout-picker/` completes with branching, back-navigation, data restoration, correct stdout `stack`.

## Sprint map (what each adds)

| Sprint | Adds to the stack model | Not a new subsystem |
|--------|-------------------------|---------------------|
| **d.1** | Schema + `GET /api/wizard/state` + static HTML + `WizardSession::new` / `snapshot` | |
| **d.2** | `POST navigate` / `finish` + `next` / `back` / `finish` on session + `wyvern-api.js` | |
| **d.3** | ADR-0005 edge-case **tests** (four history cases) — same `BrowserHistory` | ✓ tests only |
| **d.4** | `window.wyvern` bootstrap from `snapshot` + stack round-trip **tests** | ✓ tests + JS |
| **d.5** | HTML **examples** exercising the stack | ✓ examples |
| **d.6** | UX polish, viewer dismiss, viewport sizing ([viewport-sizing.md](viewport-sizing.md)) | orthogonal to stack |

**d.3 and d.4 do not add traits, routes, or new state machines** — they harden the stack from d.2.

## What Phase D does not close

- `--interactive` / MCP — **Phase E**
- Rust graph/DAG/Flowise integrations — page HTML only

## Boundaries

- `wyvern-wizard` — stack + cursor only ([wizard.toml](../../boundaries/wyvern-wizard/wizard.toml))
- `wyvern-host` — HTTP glue ([host.toml](../../boundaries/wyvern-host/host.toml))

## Sprint index

| Sprint | Doc |
|--------|-----|
| d.1 | [d1-wizard-host.md](d1-wizard-host.md) |
| d.2 | [d2-wizard-ipc.md](d2-wizard-ipc.md) |
| d.3 | [d3-history-nav.md](d3-history-nav.md) |
| d.4 | [d4-stack-inject.md](d4-stack-inject.md) |
| d.5 | [d5-dag-example.md](d5-dag-example.md) |
| d.6 | [d6-wizard-polish.md](d6-wizard-polish.md) |
