# `wyvern-viewer` — Architecture

*Part of the [principal architecture](../architecture.md).*

**Status:** Active (c.15). Optional URL-only wry launcher — no dialog IPC, no HTTP server.

---

## ADR-0019: Optional embedded viewer as a sibling binary

**Status:** Accepted

**Decision summary:**

1. **`wyvern-viewer` is a separate binary** — navigates to a dialog URL served by `wyvern-host`; never embeds HTML or owns dialog session state.
2. **CLI owns spawn** — `wyvern` discovers and launches the sibling binary for `--viewer embedded` (`embedded_viewer_spawn`); `wyvern-host` never spawns it.
3. **Loopback-only by default** — dialog URLs must use `http`/`https` on a loopback host unless an explicit opt-in env is set (mirrors host bind policy / ADR-0016).
4. **OS-close → dismissed** — on window close, best-effort timed POST: wizard sessions use `GET /api/wizard/state` + `POST /api/wizard/finish` with the full visited stack (d.8); blocking dialogs POST `{ "button": "dismissed" }` to `/api/result`. CLI also watches child exit as a fallback (REQ-0097).

**Authority:** [http-viewer-contract.md](../plans/phase-C/http-viewer-contract.md), [HTTP-TYPES.md](../plans/phase-C/HTTP-TYPES.md), principal [ADR-0011](../architecture.md) / [ADR-0019](../architecture.md).

---

## Module shape

```
crates/wyvern-viewer/
  src/
    main.rs      # argv / env → run
    run.rs       # winit event loop + wry URL load; OS-close → dismiss
    dismiss.rs   # wizard finish stack vs blocking /api/result (d.8)
    platform.rs  # macOS / Win / Linux window attrs
    viewport.rs  # viewport-bounds IPC helpers (d.6)
```

**Boundaries:** `boundaries/wyvern-viewer/viewer.toml` — no `wyvern-host` / `wyvern-schema` deps; forbids `http_server`. Allows lifecycle stdin (`exit`) and lightweight chrome IPC (`resize:` / `navigate:`) per REQ-V006.
