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

**Consequences:** Works from any shell, language, or agent. MCP tool parameters map 1:1 — no restructuring. Binary data passed by file path or base64. The protocol stays intentionally small: blocking dialog commands plus a few lifecycle actions in `--interactive`.

---

### ADR-0012: Prefer the smallest coherent API surface

**Status:** Accepted

Wyvern should solve the product with the fewest command shapes that preserve clear semantics. If an interaction starts to feel complicated, treat that as a documentation, scoping, or boundary problem first.

**Consequences:** `message` remains a blocking modal. Persistent transports (`--interactive`, MCP) do not silently change dialog semantics. Modeless behavior belongs in a separate future `notification` command rather than overloading existing commands.

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

---

### ADR-0013: Direct type dispatch — one handler per command

**Status:** Accepted

**Context:**
Wyvern accepts many JSON command shapes over time. A common failure mode is accumulating mode flags, stub handlers, and nested routing that makes it hard to trace JSON input to stdout output.

**Decision:**
After validation, each command becomes a typed `Command` enum variant. Execution is a single `match` (or equivalent) on `type` with one handler function per variant. Handlers return a `CommandResult` serialized to stdout. Unimplemented types are rejected at validation time for the current phase — never at runtime with a stub handler.

**Pipeline:**

```
load → parse_json → validate(phase_surface) → Command → dispatch(type) → CommandResult → stdout
```

**Consequences:**
- Phase 1 validates and executes only `chrome`
- Each Phase 2+ sprint adds one enum variant, one validator module, one handler — not a routing table refactor
- `--interactive` reuses the same `validate → dispatch` path inside the read loop; lifecycle `action` values are a separate small enum, not mixed into dialog `type` routing
- If implementation needs complicated branching to pick a path, treat that as a design smell and simplify before merging
- Each pipeline stage uses a **discriminated union** for errors; re-map to stderr JSON at scope boundaries only — do not merge unlike variants into one generic error type
