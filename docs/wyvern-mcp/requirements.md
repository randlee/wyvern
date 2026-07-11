# `wyvern-mcp` — Requirements

*Part of the [principal requirements](../requirements.md).*

---

## MCP Mode (REQ-0074 – REQ-0076)

**REQ-0074** — When running as an MCP server (`wyvern --mcp`), operate as a persistent background process. Window survives across tool calls.

**REQ-0075** — Each dialog type (`message`, `input`, `markdown`, `question`, `wizard`) registered as an MCP tool with parameter schemas identical to CLI JSON schemas — no field renaming.

**REQ-0076** — `question` MCP tool call blocks until the user answers; result returned as tool response. All other types are fire-and-forget display commands.
