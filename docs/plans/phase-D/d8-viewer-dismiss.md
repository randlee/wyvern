---
id: d.8
title: Wizard viewer dismiss
status: complete
branch: feature/phase-D-d8-viewer-dismiss
target: integrate/phase-D
---

# Sprint d.8 — Wizard viewer dismiss

## Goal

OS-close on wizard sessions returns `dismissed` with full visited stack via `POST /api/wizard/finish`.

## Hard dependencies

- **d.7** merged

## Deliverables

| File | Change |
|------|--------|
| `crates/wyvern-viewer/src/dismiss.rs` (or session handler) | Detect wizard session; on OS-close POST `/api/wizard/finish` |
| `boundaries/wyvern-viewer/viewer.toml` | Allow `serde` / `serde_json` for dismiss JSON (ADR-0021) |
| `crates/wyvern-viewer/tests/wizard_dismiss.rs` | Viewer routes wizard dismiss correctly |
| `crates/wyvern-host/tests/wizard_dismiss.rs` | Host accepts dismissed finish with full visited stack |
| `docs/plans/phase-C/http-viewer-contract.md` | Update dismissed wizard steps (owned here) |
| `docs/plans/phase-C/http-wizard-contract.md` | Cross-link dismissed stack algorithm |

**Dismissed algorithm (normative — matches d.2 finish):**

1. Viewer detects wizard session (`GET /api/wizard/state` succeeds or URL path `/wizard/`)
2. `GET /api/wizard/state` → read `page`, `page_data`, `stack` (prior entries per REQ-0024)
3. Build full visited stack = `stack` + `{ page, data: page_data }`
4. `POST /api/wizard/finish` with `{ "button": "dismissed", "data": <page_data>, "stack": <full visited stack> }` (request `data` must equal `page_data` so host `visited_stack_with_current(data)` validation succeeds; stdout `data` remains `{}`)
5. Host validates client `stack` against session-derived stack; mismatch → 400; stdout = validated result (`button: dismissed`, `data: {}`, full visited stack)

**Host/CLI fallback (normative — REQ-0097):** when viewer exits without POST or session times out, host derives dismissed result via `WizardSession::finish(Dismissed, page_data, derived_stack)` using the same in-memory algorithm as d.2 (full visited stack; stdout `data: {}`). Applies to `DialogHandle::viewer_exited_without_result()` and host session-timeout path.

## Acceptance criteria

1. Viewer close on any wizard page returns `{"button":"dismissed","stack":[...]}` including current page entry
2. Dismissed stdout stack equals session-derived `entries[0..=cursor]` (not prior-only)
3. Layout-picker example still passes full smoke from d.5

## Required validation

```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
cargo test -p wyvern-host wizard_dismiss
cargo test -p wyvern-viewer wizard_dismiss
sc-lint check native --config .sc-lint.toml
```

## Non-closure

- `--interactive` wizard loops (Phase E)
- MCP wizard tools (Phase E)

## Authority

- [http-wizard-contract.md](../phase-C/http-wizard-contract.md)
- [http-viewer-contract.md](../phase-C/http-viewer-contract.md)
- REQ-0066 (`dismissed` button), REQ-0097
