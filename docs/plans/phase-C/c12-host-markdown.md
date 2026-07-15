---
id: c.12
title: Host + UI — markdown
status: complete
branch: feature/phase-C-c12-host-markdown
worktree: /Volumes/Extreme Pro/github/wyvern-worktrees/feature/phase-C-c12-host-markdown
target: integrate/phase-c-web-server
---

# Sprint c.12 — `markdown` on HTTP host

## Goal

Close **`markdown`** with server-side `content_html` in `/api/dialog`.

## Hard dependencies

- **c.11** merged

## Decision (locked)

**Server-side pre-render:** `pulldown-cmark` + **`ammonia`** in `wyvern-host` adds sanitized `content_html` to dialog JSON. Template renders HTML; no client-side markdown fetch.

## Deliverables

- `ui/markdown/` — styles, scroll, buttons
- `wyvern-host/src/markdown.rs` — `pulldown-cmark` → **`ammonia` sanitize** → `content_html` on `DialogPayloadMarkdown`
- `boundaries/wyvern-host/host.toml` — `pulldown-cmark`, `ammonia` allowed; `dialog_content_html` in `io_owns`
- `wyvern-host::run` handles `Command::Markdown`
- `wyvern my-doc.md --viewer none` unchanged file-path behavior
- `tests/e2e/markdown.spec.ts`

## Acceptance criteria

1. `cargo build --workspace` + `cargo clippy --workspace -- -D warnings` green
2. `wyvern '{"type":"markdown",...}' --viewer none` — e2e passes
3. `wyvern my-doc.md --viewer none` — e2e passes
4. `GET /api/dialog` for markdown includes sanitized `content_html` per `DialogPayloadMarkdown`
5. Markdown body with `<script>alert(1)</script>` → `content_html` has no script/event handlers (unit test)
6. Prior types regression passes

## Required validation

```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
cargo test -p wyvern-host
cargo test -p wyvern
# L2: npx playwright test tests/e2e/markdown.spec.ts
```

## Non-closure

- `question`, `chrome` (c.13–c.14)

## Authority

- [HTTP-TYPES.md](HTTP-TYPES.md) — `DialogPayloadMarkdown`
- [http-post-schema.md](http-post-schema.md)
