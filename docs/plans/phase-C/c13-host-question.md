---
id: c.13
title: Host + UI — question
status: implemented
branch: feature/phase-C-c13-host-question
target: integrate/phase-c-web-server
---

# Sprint c.13 — `question` on HTTP host

## Goal

Close **`question`**: single + multi-select, dismiss semantics per contract.

## Hard dependencies

- **c.12** merged

## Deliverables

- `ui/question/` — options, `data-testid` per option
- Server-side preview sanitization — `ammonia` in `wyvern-host` maps option `preview` → `preview_html` at `GET /api/dialog`
- `POST /api/result` shapes per [http-post-schema.md](http-post-schema.md)
- `wyvern-host::run` handles `Command::Question`
- `tests/e2e/question.spec.ts` — single + multi + dismiss

## Acceptance criteria

1. `cargo build --workspace` + `cargo clippy --workspace -- -D warnings` green
2. `wyvern '{"type":"question",...}' --viewer none` — e2e passes (single + multi)
3. `GET /api/dialog` exposes sanitized `preview_html` for options with `preview` field
4. Dismiss returns extended shape per REQ-0068 / contract
5. Prior types regression passes

## Required validation

```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
cargo test -p wyvern-host
cargo test -p wyvern
# L2: npx playwright test tests/e2e/question.spec.ts
```

## Non-closure

- `chrome` (c.14)

## Authority

- [HTTP-TYPES.md](HTTP-TYPES.md) — `QuestionResult`, `DialogPayloadQuestion`
- [http-post-schema.md](http-post-schema.md)
