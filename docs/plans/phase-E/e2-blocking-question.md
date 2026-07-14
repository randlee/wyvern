---
id: e.2
title: Blocking dialogs and exit in interactive mode
status: planning
branch: feature/phase-E-e2-blocking-question
target: integrate/phase-E
---

# Phase E / e.2 — Blocking dialogs and `exit` in interactive mode

## Status
pending

## Hard dependency

**e.1** merged.

## Ownership (locked)

| Concern | Owner |
|---------|-------|
| Blocking `run_dialog` + `DialogHandle::await_result` | **`wyvern-host`** |
| System / named URL open | **`wyvern-host`** `browser_launch` (inside `run_dialog`) |
| Embedded navigate per dialog | **`wyvern` CLI** → **`wyvern-viewer`** |
| `exit` + viewer teardown | **`wyvern` CLI** → viewer subprocess + `HostSession::shutdown` |

## Deliverables

- `HostSession::run_dialog` → `DialogHandle` in interactive loop (not one-shot `run()`)
- After `run_dialog`: CLI navigates embedded viewer **or** host already opened system/named URL inside `run_dialog`; then `await_result`
- `{"action":"exit"}` shuts down viewer + host cleanly
- `--persistent` accepted as alias for `--interactive`

## Acceptance criteria

- Blocking dialog commands return normal JSON result on stdout; loop resumes afterward (`DialogHandle::await_result`)
- `{"action":"exit"}` shuts down `HostSession` + CLI-owned viewer and terminates process cleanly
- Viewer OS-close in interactive mode (locked — [http-viewer-contract.md](../phase-C/http-viewer-contract.md)): active dialog completes with dismissed semantics; CLI posts `{ "button": "dismissed" }` to `POST /api/result` (or wizard `POST /api/wizard/finish`) when the child exits without a result; **loop stays alive** (not process exit)
- After `run_dialog`, launch/navigate by mode: CLI navigates embedded viewer; **host** `browser_launch` for `system`/named; no launch for `none`
- `--persistent` alias works identically to `--interactive`

## Required validation

```bash
cargo test -p wyvern interactive_blocking
cargo test -p wyvern-host host_session_dialog
# Manual: blocking question in loop → exit
```

## Non-closure

- MCP server (e.3–e.4)

## Authority

[http-interactive-mcp-contract.md](../phase-C/http-interactive-mcp-contract.md), [HTTP-TYPES.md](../phase-C/HTTP-TYPES.md)
