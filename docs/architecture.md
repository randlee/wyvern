# Wyvern — Architecture

Architecture decisions are recorded as ADRs. Cross-cutting ADRs live here. Crate-specific ADRs live in `docs/<crate>/architecture.md` — follow the links below for progressive disclosure.

---

## Crate Architecture Map

| Crate | Responsibility | ADRs |
|-------|---------------|------|
| `wyvern` | CLI entry point, arg parsing, `--interactive` loop | [docs/wyvern/architecture.md](wyvern/architecture.md) |
| `wyvern-schema` | JSON types, validation, error messages | [docs/wyvern-schema/architecture.md](wyvern-schema/architecture.md) |
| `wyvern-window` | Webview, IPC bridge, HTML chrome, platform chrome | [docs/wyvern-window/architecture.md](wyvern-window/architecture.md) |
| `wyvern-wizard` | Wizard navigation state machine | [docs/wyvern-wizard/architecture.md](wyvern-wizard/architecture.md) |
| `wyvern-mcp` | MCP server, tool mapping, persistent window | [docs/wyvern-mcp/architecture.md](wyvern-mcp/architecture.md) |

---

## Cross-Cutting ADRs

### ADR-0003: Rust as the implementation language

**Status:** Accepted

`wry` is a Rust crate. Rust gives a single statically-linked binary, small footprint, fast startup, and strong type-safety on the schema layer. `serde_json` for JSON I/O; `strsim` for Levenshtein validation suggestions.

---

### ADR-0004: JSON as the sole protocol — stdin/stdout

**Status:** Accepted

JSON in (stdin, file, or inline arg), JSON out (stdout). Errors on stderr as structured JSON. One command per line in interactive mode.

**Consequences:** Works from any shell, language, or agent. MCP tool parameters map 1:1 — no restructuring. Binary data passed by file path or base64.

---

### ADR-0011: Cargo workspace crate structure and boundaries

**Status:** Accepted

Five-crate workspace with enforced dependency boundaries:

```
wyvern-schema   →  (no internal deps — pure types + logic)
wyvern-wizard   →  wyvern-schema
wyvern-window   →  wyvern-schema, wyvern-wizard, wry, winit
wyvern          →  wyvern-window, wyvern-schema
wyvern-mcp      →  wyvern-window, wyvern-schema
```

**Boundary rules:**
- `wyvern-schema` and `wyvern-wizard` are pure logic — no I/O, no window, no async
- `wry` and `winit` only in `wyvern-window`
- `wyvern-mcp` accesses the window only through `wyvern-window`'s public API
- `wyvern` binary is a thin entry point — logic belongs in library crates

Boundary rules are encoded as sc-lint-boundary constraints in `boundaries/` and enforced in CI from Phase 2.

All five crate names confirmed available on crates.io.
