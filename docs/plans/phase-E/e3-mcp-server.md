# Phase E / e.3 — MCP server wrapper and tool mapping

## Status
pending

## Acceptance Criteria

- Wyvern starts as MCP server with `wyvern --mcp`
- Each type (`message`, `input`, `markdown`, `question`, `wizard`) registered as an MCP tool
- Tool parameter schemas identical to CLI JSON schemas (no renaming)
- MCP tool calls invoke the correct dialog and return result as tool response

## Notes
