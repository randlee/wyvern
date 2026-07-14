---
id: c.10
title: wyvern-host scaffold + message + workspace green
status: complete
branch: feature/phase-C-c10-http-host-message
worktree: /Volumes/Extreme Pro/github/wyvern-worktrees/feature/phase-C-c10-http-host-message
target: integrate/phase-c-web-server
---

# Sprint c.10 — `wyvern-host` + `message` (workspace compiles)

## Goal

Greenfield `wyvern-host`, wire `wyvern` pipeline, ship `ui/message/`. **First compile + CI gate** after c.9.

## Hard dependencies

- **c.9** merged — `wyvern-window` absent; `./scripts/verify-c9-deletion.sh` exit 0

## Deliverables

- `crates/wyvern-host/` — `run()`, server, session, `GET /api/dialog`, `POST /api/result`
- `ui/message/` + `ui/shared/wyvern-api.js` with `data-testid` on primary buttons
- `wyvern` → `wyvern_host::run` for `Command::Message` only
- `wyvern-mcp` stub — drop `wyvern-window` dep
- `boundaries/wyvern-host/host.toml` enforced
- Parse `--viewer` enum; implement **`none` only**; **omitted `--viewer` defaults to `none`** until c.15; set `WYVERN_DIALOG_URL`
- L2 harness: `tests/e2e/package.json`, `playwright.config.ts`, `message.spec.ts`

## Acceptance criteria

1. `cargo build --workspace` + `cargo clippy --workspace -- -D warnings` green
2. `cargo test -p wyvern-host` — HTTP only, no wry/winit
3. `wyvern '{"type":"message","title":"T","message":"Hi","buttons":"ok"}' --viewer none` → `{"button":"ok"}`
4. `wyvern '{"type":"message",...}'` with **no** `--viewer` flag → same as `--viewer none` (interim default)
5. Other types → `HostError::UnsupportedType` at run time (validation passes) until c.11–c.14
6. `--ui-root ./custom-ui` serves alternate template without rebuild
7. `sc-lint check native --config .sc-lint.toml` passes
8. Playwright `message.spec.ts` passes with `--viewer none`

## Required validation

```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
cargo test -p wyvern-host
cargo test -p wyvern
sc-lint check native --config .sc-lint.toml
# L2 (CI): npx playwright test tests/e2e/message.spec.ts
```

## Non-closure

- `input`, `markdown`, `question`, `chrome` (c.11–c.14)
- `embedded` / named viewers (c.15)
- Release tag (c.16)

## Authority

- [HTTP-TYPES.md](HTTP-TYPES.md) — `HostOptions`, `HostError`, `run()`
- [http-dialog-contract.md](http-dialog-contract.md), [c9-testing-headless.md](c9-testing-headless.md) (strategy; harness owned here)
