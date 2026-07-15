# `wyvern-mcp` — Architecture

*Part of the [principal architecture](../architecture.md).*

---

## ADR-0009: MCP mode runs Wyvern as a persistent background process

**Status:** Accepted

**Context:**
As an MCP server, Wyvern could launch and kill a window per tool call, or keep a persistent process with show/hide semantics.

**Decision:**
Wyvern MCP server is a persistent background process. **`HostSession`** persists across tool calls; optional **`wyvern-viewer`** subprocess is owned by **`wyvern` CLI** (not host). Blocking dialog tools keep the same modal semantics they have in the CLI. In MVP, the public MCP tool surface is the dialog commands only; lifecycle controls remain part of `--interactive`.

**Consequences:**
- Host HTTP state survives between tool calls (persistent `HostSession`)
- CLI may spawn one embedded viewer for desktop MCP sessions; CI uses `--viewer none`
- Each tool call: `run_dialog` → `DialogHandle` → `await_result`
- No per-call launch latency after first invocation
- Each blocking tool: HTTP dialog → `POST /api/result` → tool response JSON
- Single Wyvern MCP instance serves the full agent session

**Amendment (HTTP host):** MCP never touches wry IPC or inline HTML. All dialog tools delegate to `wyvern-host::HostSession::run_dialog`. See [http-interactive-mcp-contract.md](../plans/phase-C/http-interactive-mcp-contract.md).
