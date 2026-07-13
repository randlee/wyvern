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

| Command | Return |
|------|--------|
| `chrome` (Phase A) | `{ "button": "dismissed" }` on OS close |
| `message` | `{ "button": "..." }` |
| `input` | `{ "button": "...", "input": "..." }` |
| `markdown` | `{ "button": "..." }` |
| `wizard` | `{ "button": "...", "data": {}, "stack": [] }` |
| `question` (normal completion) | `{ "questions": [...], "answers": {}, "response": "" }` |
| `question` (force close) | `{ "button": "dismissed", "questions": [...], "answers": {}, "response": "" }` |
| `show` / `hide` / `exit` in `--interactive` | `{ "action": "...", "ok": true }` |

---

## Command Surface Summary

**Phase A executable** — `chrome` (Phase A foundation probe)

**Blocking dialog commands** — `message`, `input`, `markdown`, `question`, `wizard` (Phase B+; validated incrementally as each type ships)

**`--interactive` lifecycle actions** — `show`, `hide`, `exit` (Phase E)

**Deferred** — `notification` is reserved as the future fire-and-forget path for ephemeral updates. MVP does not overload `message` with modeless semantics.

---

## Non-Functional Requirements

**NFR-0001** — On macOS, window opens in under 500ms from process launch.

**NFR-0002** — On macOS, resident memory does not exceed 80MB under normal operation.

**NFR-0003** — Compiled binary does not exceed 10MB on macOS.

**NFR-0004** — Wyvern does not require a browser installed on the host system.

**NFR-0005** — Runs on macOS (WebKit), Windows (WebView2), and Linux (WebKitGTK).

**NFR-0006** — JSON schema for all dialog types maps 1:1 to MCP tool parameters — no field renaming or restructuring.

**NFR-0007** — Validation error messages are human-readable and actionable without consulting documentation.

**NFR-0008** — Host never inspects or interprets wizard page-specific `data`. It only uses page descriptors and explicit navigation pointers to move through the wizard.

**NFR-0009** — `question` type adopts the public Claude `AskUserQuestion` fields and behavior as closely as possible within Wyvern's standard `type`-based command envelope. Any divergence, such as Wyvern's explicit `dismissed` sentinel on force close, must be explicit and justified.

**NFR-0010** — Interactive mode supports concurrent use from a background shell process with stdin/stdout handles held open, no additional setup required.

**NFR-0011** — If any flow feels complicated in implementation or review, treat that as a design/documentation smell and simplify toward the smallest consistent API before adding new behavior.
