# `wyvern-mcp` — Requirements

*Part of the [principal requirements](../requirements.md).*

---

## MCP Mode (REQ-0074 – REQ-0077)

**REQ-0074** — When running as an MCP server (`wyvern --mcp`), operate as a persistent background process. Window survives across tool calls.

**REQ-0075** — Each dialog type (`message`, `input`, `markdown`, `question`, `wizard`) registered as an MCP tool with parameter schemas identical to CLI JSON schemas — no field renaming.

**REQ-0076** — Blocking dialog MCP tool calls (`message`, `input`, `markdown`, `question`, `wizard`) keep the same modal semantics they have in the CLI and return their normal JSON result as the tool response.

**REQ-0077** — `show`, `hide`, and `exit` are not public MCP tools in MVP.
