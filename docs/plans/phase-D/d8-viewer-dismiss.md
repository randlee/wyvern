---
id: d.8
title: Wizard viewer dismiss
status: planning
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
| `crates/wyvern-viewer/tests/wizard_dismiss.rs` | Viewer routes wizard dismiss correctly |
| `crates/wyvern-host/tests/wizard_polish.rs` | Host accepts dismissed finish with full visited stack |
| `docs/plans/phase-C/http-viewer-contract.md` | Update dismissed wizard steps (owned here) |
| `docs/plans/phase-C/http-wizard-contract.md` | Cross-link dismissed stack algorithm |

**Dismissed algorithm (normative — matches d.2 finish):**

1. Viewer detects wizard session (`GET /api/wizard/state` succeeds or URL path `/wizard/`)
2. `GET /api/wizard/state` → read `page`, `page_data`, `stack` (prior entries per REQ-0024)
3. Build full visited stack = `stack` + `{ page, data: page_data }`
4. `POST /api/wizard/finish` with `{ "button": "dismissed", "data": {}, "stack": <full visited stack> }`
5. Host validates client `stack` against session-derived stack; mismatch → 400; stdout = validated result

## Acceptance criteria

1. Viewer close on any wizard page returns `{"button":"dismissed","stack":[...]}` including current page entry
2. Dismissed stdout stack equals session-derived `entries[0..=cursor]` (not prior-only)
3. Layout-picker example still passes full smoke from d.5

## Required validation

```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
cargo test -p wyvern-host wizard_polish
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
