# HTTP Interactive & MCP contract (Phase E)

Phase E reuses **`wyvern-host`** as a **persistent** dialog server. Stdin (or MCP stdio) remains command ingress; **dialog transport is HTTP** (ADR-0016 / ADR-0008 amendment).

**Prerequisite:** Phase C complete (c.16), Phase D optional for wizard tools.

Related: [http-dialog-contract.md](http-dialog-contract.md), [http-post-schema.md](http-post-schema.md), [http-wizard-contract.md](http-wizard-contract.md), [http-viewer-contract.md](http-viewer-contract.md).

---

## Process model

```text
wyvern --interactive          wyvern --mcp
        │                            │
        └─► persistent HostSession ◄─┘
              ├─ HTTP server stays up
              └─ stdin/MCP dispatches validate → run_dialog → stdout/MCP result

wyvern CLI (parallel, session-scoped):
  ├─ optional wyvern-viewer subprocess (one window — CLI-owned, not host)
  ├─ embedded_viewer_spawn on session start (--viewer embedded)
  └─ show/hide/exit lifecycle → viewer subprocess + HostSession::shutdown
```

- **Not** one process per dialog — host survives between commands.
- Each blocking dialog: `run_dialog` → `DialogHandle` → launch/navigate per viewer mode (below) → `await_result` → emit JSON line.
- **`HostSession` has no `wyvern-viewer` child field** — spawn/show/hide owned by **`wyvern` CLI**.
- **`--viewer none` + headless** remains valid for agent-driven CI (not the product default).

---

## `--interactive` (e.1–e.2)

### Stdin (unchanged ingress)

One JSON object per line — same types as one-shot CLI.

### Lifecycle actions (separate enum — not dialog `type`)

| Command | Behavior |
|---------|----------|
| `{"action":"show"}` | **`wyvern` CLI** shows `wyvern-viewer` window if hidden; `{ "action": "show", "ok": true }` |
| `{"action":"hide"}` | **`wyvern` CLI** hides viewer; host HTTP stays up; `{ "action": "hide", "ok": true }` |
| `{"action":"exit"}` | CLI shuts down viewer + `HostSession::shutdown`; `{ "action": "exit", "ok": true }` then exit 0 |

### Blocking dialog in loop

1. Read line → `validate` → `HostSession::run_dialog(command)` → `DialogHandle`.
2. Launch / navigate by mode:
   - **`embedded`:** CLI navigates the existing `wyvern-viewer` subprocess to `dialog_url`.
   - **`system` / named:** **`wyvern-host`** opens the URL via `browser_launch.rs` (not the CLI).
   - **`none`:** no launch; harness reads `WYVERN_DIALOG_URL`.
3. `DialogHandle::await_result()` blocks until `POST /api/result` (or wizard finish).
4. Write **one** JSON result line to stdout.
5. Loop — host does **not** exit.

### Viewer policy

- **Default desktop:** one `wyvern-viewer` subprocess for the session (`--viewer embedded` default); CLI navigates to each new `dialog_url`.
- **Override `system` / named:** host launches each dialog URL via `browser_launch.rs` + registry (CLI does not open external browsers).
- **CI / agents:** `--viewer none`; Playwright attaches per dialog URL.

---

## MCP (e.3–e.4)

### ADR-0009 amendment

Persistent process = **persistent `wyvern-host` + CLI-owned optional viewer**, not persistent wry inline HTML.

### Tool mapping

| MCP tool | Maps to |
|----------|---------|
| `wyvern_message` | `Command::Message` → HTTP dialog |
| `wyvern_input` | `Command::Input` → HTTP dialog |
| … | Same JSON params as CLI (NFR-0006) |

Each tool call:

1. `validate(params)` → `run_dialog` on shared `HostSession` → `DialogHandle` (host opens URL if system/named).
2. CLI navigates embedded viewer when `embedded`; `await_result`.
3. Return result JSON as tool response.

### Lifecycle

- `show` / `hide` / `exit` — **not** MCP MVP tools (remain `--interactive` only per ADR-0012).
- MCP process starts host once; CLI may spawn viewer once for embedded desktop sessions.

### Testing (e.4)

- MCP stdio integration test with `--viewer none` + HTTP client completing dialog.
- No winit in CI default path.

---

## Host API (Phase E — `wyvern-host`)

```rust
pub struct HostSession { /* persistent HTTP — no viewer child */ }

impl HostSession {
    pub fn new(options: HostOptions) -> Result<Self, HostError>;
    /// Bind + return handle. System/named: host opens URL via browser_launch.
    /// Embedded/none: no host launch (CLI or harness).
    pub fn run_dialog(&mut self, command: Command) -> Result<DialogHandle, HostError>;
    pub fn shutdown(self) -> Result<(), HostError>;
}
```

**Not on `HostSession`:** `show`, `hide`, embedded spawn — see [HTTP-TYPES.md](HTTP-TYPES.md) and [http-viewer-contract.md](http-viewer-contract.md).

One-shot CLI (Phase C): `none`/`system`/`named` may use `wyvern-host::run`; **`embedded` uses DialogHandle + CLI `embedded_viewer_spawn`** (not `host::run`).

---

## Sprint mapping

| Sprint | Work |
|--------|------|
| e.1 | `HostSession`, stdin loop, lifecycle actions (CLI-owned viewer) |
| e.2 | Blocking dialogs in loop; viewer reuse via navigate |
| e.3 | `wyvern-mcp` → `HostSession::run_dialog` |
| e.4 | MCP + headless e2e |

## Crate boundaries

- `wyvern` — stdin loop, arg parsing, **`embedded_viewer_spawn`**, show/hide/exit
- `wyvern-host` — HTTP + `HostSession` (no webview, no embedded spawn)
- `wyvern-viewer` — URL webview; show/hide target; OS-close POST
- `wyvern-mcp` — MCP transport only; no direct HTTP
