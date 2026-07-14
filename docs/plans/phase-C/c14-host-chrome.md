---
id: c.14
title: Host + UI — chrome
status: planning
branch: feature/phase-C-c14-host-chrome
target: integrate/phase-c-web-server
---

# Sprint c.14 — `chrome` on HTTP host (full dialog matrix)

## Goal

Close **`chrome`**; complete pre-wizard dialog matrix on HTTP host.

## Hard dependencies

- **c.13** merged

## Deliverables

- `ui/chrome/` — status template; Win/Linux HTML close/minimize in template JS
- Sizing constants in template/CSS (480×360 open, 800×600 max)
- `wyvern-host::run` handles `Command::Chrome`
- `tests/e2e/chrome.spec.ts`
- Full L2 matrix: message, input, markdown, question, chrome

## Acceptance criteria

1. `cargo build --workspace` + `cargo clippy --workspace -- -D warnings` green
2. `wyvern '{"type":"chrome",...}' --viewer none` — e2e passes
3. All five types pass headless e2e with `--viewer none`
4. No `wyvern-window` paths remain in repo

## Required validation

```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
cargo test --workspace
sc-lint check native --config .sc-lint.toml
# L2: npx playwright test tests/e2e/
```

## Non-closure

- `wyvern-viewer`, browser registry (c.15)
- Release tag (c.16)

## Authority

- [HTTP-TYPES.md](HTTP-TYPES.md)
- [http-post-schema.md](http-post-schema.md)
