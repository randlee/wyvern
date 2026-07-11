# `wyvern-mcp` — Architecture

*Part of the [principal architecture](../architecture.md).*

---

## ADR-0009: MCP mode runs Wyvern as a persistent background process

**Status:** Accepted

**Context:**
As an MCP server, Wyvern could launch and kill a window per tool call, or keep a persistent process with show/hide semantics.

**Decision:**
Wyvern MCP server is a persistent background process. Window persists across tool calls. `show`/`hide` commands control visibility. Same JSON command vocabulary as `--interactive` mode used for MCP tool calls.

**Consequences:**
- Window state survives between tool calls
- No per-call launch latency after first invocation
- `question` tool calls block until answered, matching `canUseTool` callback pattern
- Single Wyvern MCP instance serves the full agent session
