# Wyvern — Requirements

Functional requirements are prefixed **REQ**, non-functional **NFR**. Crate-specific requirements live in `docs/<crate>/requirements.md` — follow the links below for progressive disclosure.

---

## Requirements by Crate

| Crate | Scope | Detail |
|-------|-------|--------|
| `wyvern` | CLI invocation, interactive mode | [docs/wyvern/requirements.md](wyvern/requirements.md) |
| `wyvern-schema` | Validation, error messages, return values | [docs/wyvern-schema/requirements.md](wyvern-schema/requirements.md) |
| `wyvern-window` | Dialog types, icons, chrome frame, platform window | [docs/wyvern-window/requirements.md](wyvern-window/requirements.md) |
| `wyvern-wizard` | Wizard navigation, history model, IPC contract | [docs/wyvern-wizard/requirements.md](wyvern-wizard/requirements.md) |
| `wyvern-mcp` | MCP server, tool mapping, persistent window | [docs/wyvern-mcp/requirements.md](wyvern-mcp/requirements.md) |

---

## Return Values Summary

| Type | Return |
|------|--------|
| `message` | `{ "button": "..." }` |
| `input` | `{ "button": "...", "input": "..." }` |
| `markdown` | `{ "button": "..." }` |
| `wizard` | `{ "button": "...", "data": {}, "stack": [] }` |
| `question` | `{ "questions": [], "answers": {}, "response": "" }` |
| Any (force close) | `{ "button": "dismissed" }` |

---

## Non-Functional Requirements

**NFR-0001** — On macOS, window opens in under 500ms from process launch.

**NFR-0002** — On macOS, resident memory does not exceed 80MB under normal operation.

**NFR-0003** — Compiled binary does not exceed 10MB on macOS.

**NFR-0004** — Wyvern does not require a browser installed on the host system.

**NFR-0005** — Runs on macOS (WebKit), Windows (WebView2), and Linux (WebKitGTK).

**NFR-0006** — JSON schema for all dialog types maps 1:1 to MCP tool parameters — no field renaming or restructuring.

**NFR-0007** — Validation error messages are human-readable and actionable without consulting documentation.

**NFR-0008** — Host never inspects or interprets wizard page data. All domain logic resides in caller-supplied HTML/JS.

**NFR-0009** — `question` type remains backward-compatible with Claude `AskUserQuestion` API at all times. Extensions are additive only.

**NFR-0010** — Interactive mode supports concurrent use from a background shell process with stdin/stdout handles held open, no additional setup required.
