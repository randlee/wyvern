# `wyvern-mcp` — Architecture

*Part of the [principal architecture](../architecture.md).*

---

## ADR-0009: MCP mode runs Wyvern as a persistent background process

**Status:** Accepted

**Context:**
As an MCP server, Wyvern could launch and kill a window per tool call, or keep a persistent process with show/hide semantics.

**Decision:**
Wyvern MCP server is a persistent background process. Window persists across tool calls. Blocking dialog tools keep the same modal semantics they have in the CLI. In MVP, the public MCP tool surface is the dialog commands only; lifecycle controls remain part of `--interactive`.

**Consequences:**
- Window state survives between tool calls
- No per-call launch latency after first invocation
- Blocking dialog tool calls return their normal JSON result after the user completes the interaction
- Single Wyvern MCP instance serves the full agent session
