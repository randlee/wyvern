# `wyvern-host` — Architecture

*Part of the [principal architecture](../architecture.md).*

**Status:** Planned (c.10+). Greenfield crate — **do not refactor** deleted `wyvern-window` code.

---

## ADR-0016: HTTP dialog host with packaged, pluggable UI

**Status:** Accepted (planning)

**Decision summary:**

1. **Host, not renderer** — ephemeral HTTP server per one-shot invocation (persistent session under `--interactive` in Phase E).
2. **No inline HTML in Rust** — default UIs ship as files under `share/wyvern/ui/`.
3. **Pluggable templates** — `--ui-root` / `ui_root` JSON; swap HTML/icons without recompiling.
4. **Viewer orthogonal** — any HTTP client may load the dialog URL. **`wyvern` CLI** spawns `wyvern-viewer` for `--viewer embedded` (c.15); host never owns webview/wry. **Interim (c.10–c.14):** omitted `--viewer` → `none`. **Product default from c.15:** `embedded`. CI/agents use `none`. Named system browsers (`chrome`, `safari`, `edge`, `firefox`, `system`) via host `browser_launch.rs` + registry — [http-viewer-contract.md](../plans/phase-C/http-viewer-contract.md).
5. **Bind policy** — `127.0.0.1` default; explicit opt-in for `0.0.0.0`.

Full text: [principal ADR-0016](../architecture.md). **Rust types:** [HTTP-TYPES.md](../plans/phase-C/HTTP-TYPES.md).

---

## ADR-0017: HTTP transport replaces wry IPC for dialogs

**Status:** Accepted (planning)

**Context:** Phase B/C used `WebViewBuilder::with_html()` and wry `with_ipc_handler`. That fused CLI process, renderer, and page logic — unusable on macOS for real work and incompatible with remote browsers.

**Decision:**

| Concern | Mechanism |
|---------|-----------|
| Page load | `GET` static files from `{ui_root}/{type}/` |
| Dialog payload | `GET /api/dialog` → JSON from validated `Command` |
| User result | `POST /api/result` → `CommandResult` |
| Picker (input file/folder) | Host route + `rfd` (see contract doc) |

**Consequences:**

- [ipc-dialog-contract.md](../plans/phase-B/ipc-dialog-contract.md) is **historical** for the deprecated stack.
- Authoritative contract: [http-dialog-contract.md](../plans/phase-C/http-dialog-contract.md).
- POST body schemas: [http-post-schema.md](../plans/phase-C/http-post-schema.md).
- `wyvern-viewer` (c.15) only navigates to `http://{bind}:{port}/…` — spawned by **`wyvern` CLI**, not host. No dialog IPC.

---

## Module shape (implementation guide)

```text
wyvern-host/
  src/
    lib.rs          # pub fn run(command, HostOptions) -> Result<CommandResult, HostError>
    server.rs       # axum/hyper router, bind, graceful shutdown
    session.rs      # one-shot + persistent session state, result channel
    static_files.rs # UI root, security: no path traversal
    routes/
      dialog.rs     # GET /api/dialog (+ content_html / preview_html helpers)
      result.rs     # POST /api/result
      picker.rs     # file/folder picker endpoints (c.11)
    markdown.rs     # pulldown-cmark + ammonia → content_html (c.12)
    question/
      preview.rs    # pulldown-cmark + ammonia → preview_html (c.13)
    browser_catalog.rs  # hardcoded id → per-OS discovery recipes (c.15)
    browser_registry.rs # local browsers.json cache; refresh on first run / miss
    browser_launch.rs   # system + named browser dispatch (c.15) — NOT embedded
    routes/
      wizard.rs       # GET /api/wizard/state, POST navigate/finish, GET /wizard/** (d.1–d.2)
    error.rs        # HostError — mapped at CLI via emit_host_error
```

**Wizard session (Phase D):** `session.rs` holds `WizardSession`. Route handlers call `snapshot` / `navigate_*` / `finish` — no imports from private `history` internals (ADR-0007).

**Not in host:** `viewer.rs` for embedded spawn, `show`/`hide`, or `wry`/`winit`. Embedded viewer is **`wyvern` CLI → `wyvern-viewer` subprocess** — [http-viewer-contract.md](../plans/phase-C/http-viewer-contract.md).

**Rust types:** [HTTP-TYPES.md](../plans/phase-C/HTTP-TYPES.md).

---

## Dependencies (target)

```text
wyvern-host → wyvern-schema, axum, tokio, tower, tower-http, tracing
optional: rfd (c.11), webbrowser + dirs (c.15), pulldown-cmark + ammonia (c.12–c.13)
Phase D: wyvern-wizard (wizard route state only — d.1)
forbidden: wry, winit, embedded viewer spawn
```

**Locked (c.12):** `pulldown-cmark` + **`ammonia`** in host helper pre-renders sanitized `content_html` for `markdown` at `GET /api/dialog` — see [c12-host-markdown.md](../plans/phase-C/c12-host-markdown.md).

**Locked (c.13):** `pulldown-cmark` + **`ammonia`** sanitizes question option `preview` → `preview_html` at `GET /api/dialog` — see [c13-host-question.md](../plans/phase-C/c13-host-question.md).

---

## Error mapping (CLI boundary)

| HostError variant | Stderr slug / code | Exit |
|-------------------|-------------------|------|
| `Bind` | `host_bind` / `HOST_BIND_ERROR` | 7 |
| `UiNotFound` | `host_error` / `UI_NOT_FOUND` | 6 |
| `ViewerNotFound` | `host_viewer` / `HOST_VIEWER_ERROR` | 6 |
| `InvalidResult` | fail-safe dismiss or validation response | — |
| `UnsupportedType` | `host_error` / `UNSUPPORTED_TYPE` | 6 |

Exact emit helpers finalized in c.10; amend [docs/wyvern/architecture.md](../wyvern/architecture.md) table when coded.

---

## Relationship to Phase D / E

- **Wizard (D):** same host serves wizard pages; `wyvern-wizard` state stays pure; navigation via HTTP + wizard contract (d.2 updated for HTTP).
- **Interactive (E):** long-lived host process; stdin loop in `wyvern` dispatches commands to the same server; ADR-0008 stdin ingress unchanged, dialog transport is HTTP.
