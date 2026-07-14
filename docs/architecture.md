# Wyvern — Architecture

Architecture decisions are recorded as ADRs. Cross-cutting ADRs live here. Crate-specific ADRs live in `docs/<crate>/architecture.md` — follow the links below for progressive disclosure.

---

## Crate Architecture Map

| Crate | Responsibility | ADRs |
|-------|---------------|------|
| `wyvern` | CLI entry point, arg parsing, `--interactive` loop | [docs/wyvern/architecture.md](wyvern/architecture.md) |
| `wyvern-schema` | JSON types, validation, error messages | [docs/wyvern-schema/architecture.md](wyvern-schema/architecture.md) |
| `wyvern-host` | HTTP dialog server, static UI, session/result (c.10+) | [docs/wyvern-host/architecture.md](wyvern-host/architecture.md) |
| `wyvern-viewer` | Optional URL-only webview launcher (c.15) | *(planned)* |
| `wyvern-wizard` | Wizard navigation state machine | [docs/wyvern-wizard/architecture.md](wyvern-wizard/architecture.md) |
| `wyvern-mcp` | MCP server, tool mapping, persistent host | [docs/wyvern-mcp/architecture.md](wyvern-mcp/architecture.md) |
| ~~`wyvern-window`~~ | **Deleted in c.9** — crate removed from the workspace; archival docs only under `docs/wyvern-window/` | [docs/wyvern-window/architecture.md](wyvern-window/architecture.md) (archival) |

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

**Status:** Accepted — **amended c.9** (HTTP host delivery)

**Target workspace** (after c.16 delivery):

```
wyvern-schema   →  (no internal deps — pure types + logic)
wyvern-wizard   →  wyvern-schema
wyvern-host     →  wyvern-schema [, wyvern-wizard for Phase D wizard routes], HTTP stack (axum/tokio)
wyvern-viewer   →  wry, winit (optional crate — URL only)
wyvern          →  wyvern-host, wyvern-schema  (spawns wyvern-viewer via subprocess — not a required Cargo dep)
wyvern-mcp      →  wyvern-host, wyvern-schema
```

**Sprint timeline:** c.9 deletes `wyvern-window`; c.10 adds `wyvern-host`; c.15 adds `wyvern-viewer`; c.16 release.

**Boundary rules:**
- `wyvern-schema` and `wyvern-wizard` are pure logic — no I/O, no network, no async
- `wyvern-host` owns TCP/HTTP/static serve/dialog session — **no** `wry`/`winit`, **no** inline HTML templates, **no** embedded viewer spawn
- `wyvern-host` may depend on `wyvern-wizard` from Phase D (d.1) for wizard route state orchestration only — pure logic stays in `wyvern-wizard`
- `wry` and `winit` only in `wyvern-viewer` (optional) — not in `wyvern-host`
- **`wyvern` CLI** spawns `wyvern-viewer` as a **subprocess** for `--viewer embedded` — sibling binary discovery, not a required library dependency (dev builds may use `CARGO_BIN_EXE_wyvern-viewer`)
- `wyvern-mcp` accesses dialogs only through `wyvern-host`'s public API
- `wyvern` binary is a thin entry point — logic belongs in library crates
- `wyvern-window` is **removed** — do not extend. Optional URL webview = **`wyvern-viewer`** (c.15).

Boundary rules are encoded in `boundaries/` and enforced in CI.

---

### ADR-0016: HTTP dialog host with packaged, pluggable UI

**Status:** Accepted (planning — c.10+)

Ephemeral HTTP server serves packaged UI from disk; any HTTP client may be the viewer; JSON command surface unchanged. Icons, chrome, and templates live in UI files — not in Rust.

**Viewer policy (amendment):** **Interim (c.10–c.14):** omitted `--viewer` defaults to **`none`** (only `none` is implemented). **Product default from c.15:** **`embedded`** (`wyvern-viewer`). CI and headless tests use **`none`** via `WYVERN_VIEWER=none` or explicit flag. Users may select **`system`** or named browsers via **`--viewer <id>`** backed by a **local browser registry** (hardcoded catalog, cached on first run). See [http-viewer-contract.md](plans/phase-C/http-viewer-contract.md).

**Consequences:** Supersedes inline `with_html`, wry IPC, `render_*_html`, REQ-0030/0031 Rust icon catalog, REQ-0080–0087 platform chrome in `wyvern-window`. See [docs/wyvern-host/architecture.md](wyvern-host/architecture.md).

---

### ADR-0017: HTTP transport replaces wry IPC for dialogs

**Status:** Accepted (planning — c.10+)

Dialog pages use `GET /api/dialog` and `POST /api/result`. Authoritative contract: [docs/plans/phase-C/http-dialog-contract.md](plans/phase-C/http-dialog-contract.md). Phase B IPC and [chrome-ipc-contract.md](plans/phase-C/chrome-ipc-contract.md) are **historical** for the deleted `wyvern-window` stack.

**Rust types:** [HTTP-TYPES.md](plans/phase-C/HTTP-TYPES.md).

---

### ADR-0018: Delete → verify → rebuild (no refactor-in-place)

**Status:** Accepted (planning — c.9)

**Context:** Porting `wyvern-window` incrementally leaves dual stacks, feature flags, and agent thrash. Forgetting to delete dead code is harder than deleting first.

**Decision:** c.9 removes the entire `wyvern-window` crate and related assets. Merge gate is **deletion inventory**, not compile. c.10+ rebuilds on `wyvern-host` greenfield.

**Consequences:** Temporary workspace breakage after c.9 is acceptable. No `wyvern-host` code lands before deletion completes. See [c9-deletion.md](plans/phase-C/c9-deletion.md).

---

### ADR-0019: Local browser registry for named `--viewer` targets

**Status:** Accepted (planning — c.15)

**Decision:** Named browsers (`chrome`, `firefox`, …) resolve via a Wyvern-owned cache file built from a hardcoded per-OS catalog on first run / refresh. `system` uses `webbrowser::open`; `embedded` uses **`wyvern` CLI subprocess spawn** of `wyvern-viewer` (not host).

**Consequences:** `wyvern browsers list` / `refresh` CLI; no cross-platform OS browser enumeration API required. See [http-viewer-contract.md](plans/phase-C/http-viewer-contract.md).

---

### Superseded ADRs (wyvern-window — archival only)

The following remain documented under [docs/wyvern-window/architecture.md](wyvern-window/architecture.md) for history. **Do not implement** on the HTTP host path:

| ADR | Topic | Superseded by |
|-----|-------|---------------|
| ADR-0001 | `wry` engine | ADR-0016 — `wry` only in optional `wyvern-viewer` (c.15) |
| ADR-0002 | HTML chrome in webview | ADR-0016 — chrome in packaged `ui/` |
| ADR-0010 | macOS transparent title bar | `wyvern-viewer` lifts platform attrs only (c.15) |
| ADR-0010a | Win/Linux HTML window controls | Packaged `ui/chrome/` + template JS (c.14) |
| ADR-0015 | Icon assets in Rust | REQ-0102/0103 — icons in UI files; no Rust catalog |

---

### ADR-0013: Direct type dispatch — one handler per command

**Status:** Accepted

**Context:**
Wyvern accepts many JSON command shapes over time. A common failure mode is accumulating mode flags, stub handlers, and nested routing that makes it hard to trace JSON input to stdout output.

**Decision:**
After validation, each command becomes a typed `Command` enum variant. Execution is a single `match` (or equivalent) on `type` with one handler function per variant. Handlers return a `CommandResult` serialized to stdout.

**Amendment (HTTP delivery, c.10+):** `wyvern-schema` validates all shipped dialog `type` values regardless of host implementation progress. Types not yet on the `wyvern-host` handler matrix return **`HostError::UnsupportedType` at run time** (after validation passes, before or without completing the dialog). This is not a validation-time phase gate and not a stub handler — the host rejects the command explicitly with stderr `host_error` / exit `6`. Pre-HTTP phases (A–B) may still gate types at validation until the host exists.

**Pipeline (c.15+):**

```
load → validate(value) → Command → host bind → DialogHandle
  → [CLI spawn wyvern-viewer when embedded]
  → [host browser_launch when system/named — inside run() / run_dialog]
  → await_result → CommandResult → stdout
```

Parse is owned by `load`; dispatch is internal to host bind + await. Viewer spawn for **`embedded`** is owned by **`wyvern` CLI** — not `HostSession`. System/named open is owned by **`wyvern-host`**. `wyvern-host::run` covers none/system/named only; embedded uses DialogHandle composition in the CLI.

**Consequences:**
- Phase A validates and executes only `chrome`
- Each later phase adds one enum variant, one validator module, one handler — not a routing table refactor
- `--interactive` reuses the same `validate → dispatch` path inside the read loop; lifecycle `action` values are a separate small enum, not mixed into dialog `type` routing
- If implementation needs complicated branching to pick a path, treat that as a design smell and simplify before merging
- Each pipeline stage uses a **discriminated union** for errors; re-map to stderr JSON at scope boundaries only — do not merge unlike variants into one generic error type
