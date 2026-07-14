---
id: c.11
title: Host + UI — input (incl. picker)
status: planning
branch: feature/phase-C-c11-host-input
target: integrate/phase-c-web-server
---

# Sprint c.11 — `input` on HTTP host

## Goal

Close **`input`** end-to-end: template, picker routes, CLI wire, headless e2e.

## Hard dependencies

- **c.10** merged

## Deliverables

- `ui/input/` — text, password, file, folder, multi-file
- `POST /api/picker/file`, `POST /api/picker/folder` — `rfd` in `wyvern-host` only
- `wyvern-host::run` handles `Command::Input`
- `tests/e2e/input.spec.ts` — text OK + file mode with `WYVERN_MOCK_PICKER_PATH`

## Acceptance criteria

1. `cargo build --workspace` + `cargo clippy --workspace -- -D warnings` green
2. `wyvern '{"type":"input",...}' --viewer none` — headless e2e passes
3. Picker routes return paths JSON per [HTTP-TYPES.md](HTTP-TYPES.md) `PickerResponse`
4. Prior types (`message`) regression passes

## Required validation

```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
cargo test -p wyvern-host
cargo test -p wyvern
# L2: npx playwright test tests/e2e/input.spec.ts
# Picker unit/integration: WYVERN_MOCK_PICKER_PATH=/tmp/fixture.txt cargo test -p wyvern-host picker
```

## Non-closure

- `markdown`, `question`, `chrome` (c.12–c.14)

## Authority

- [HTTP-TYPES.md](HTTP-TYPES.md) — picker types
- [http-dialog-contract.md](http-dialog-contract.md), [http-post-schema.md](http-post-schema.md)
