# HTTP viewer selection (Phase C c.15+)

How the CLI opens the dialog URL after `wyvern-host` binds. Orthogonal to HTTP dialog contract — any client may load the URL.

**Rust types:** [HTTP-TYPES.md](HTTP-TYPES.md) (`ViewerMode`, `BrowserRegistryFile`).

## Viewer handoff after bind (locked — c.15)

Host **never** spawns `wyvern-viewer`. After bind, launch ownership is mode-specific:

```text
1. host bind  → DialogHandle { dialog_url, viewer_options }
                 (+ WYVERN_DIALOG_URL env when dialog_url_env / viewer none)
2. launch:
   - embedded → wyvern CLI spawns/navigates wyvern-viewer subprocess
   - system/named → wyvern-host browser_launch.rs opens URL
   - none → no process; harness attaches to URL
3. await    → DialogHandle::await_result() → CommandResult → stdout
```

**`wyvern-host::run`:** convenience for `none` / `system` / `named` only (host may call `browser_launch` inside). **Embedded one-shot must not call `host::run`** — CLI composes `DialogHandle` + `embedded_viewer_spawn` + `await_result`. See [HTTP-TYPES.md](HTTP-TYPES.md).

**Types:** [HTTP-TYPES.md](HTTP-TYPES.md) (`DialogHandle`, `ViewerLaunchOptions`, `HostRunOutcome`).

| Phase | Owner | Action |
|-------|-------|--------|
| Bind + HTTP serve | `wyvern-host` | Returns `dialog_url`; blocks on result channel internally |
| Subprocess spawn (`embedded`) | `wyvern` CLI | `embedded_viewer_spawn` — sibling binary discovery below |
| System / named open | `wyvern-host` | `browser_launch.rs` + registry — **not** CLI |
| Show / hide | `wyvern` CLI → `wyvern-viewer` | Lifecycle IPC/signals — **not** `HostSession` methods |
| OS window close | `wyvern-viewer` | POST dismissed before exit; CLI watches child as fallback |

Persistent `--interactive` / `--mcp`: CLI spawns **one** `wyvern-viewer` per session when `embedded`; each `run_dialog` returns a new `dialog_url` (CLI navigates). When `system`/named, `run_dialog` opens the URL via host `browser_launch` before returning the handle.

## `--viewer` values

| Value | Behavior | When |
|-------|----------|------|
| `embedded` | Spawn **`wyvern-viewer`** child from **`wyvern` CLI** (not `wyvern-host`) | **Product default** (c.15+) |
| `none` | No launch; set `WYVERN_DIALOG_URL` for harness | **CI / agents / headless e2e** |
| `system` | OS default browser (`webbrowser::open`) | User override |
| `chrome` | Google Chrome | User override |
| `safari` | Safari (macOS) | User override |
| `edge` | Microsoft Edge | User override |
| `firefox` | Mozilla Firefox | User override |

**Deprecated alias:** `browser` → `system` (accept for one phase; document deprecation).

**Env override:** `WYVERN_VIEWER` — when set, overrides CLI default (CI sets `WYVERN_VIEWER=none`).

**c.10 scope:** Parse full enum; **implement `none` only**; omitted flag defaults to **`none`** until c.15. Other values until c.15.

## Defaults

| Context | Default |
|---------|---------|
| **Interim (c.10–c.14)** — flag omitted | **`none`** (only mode implemented) |
| **Product CLI (c.15+)** — flag omitted | **`embedded`** |
| CI workflow env | `WYVERN_VIEWER=none` |
| Local e2e scripts | explicit `--viewer none` |
| `wyvern --interactive` / `--mcp` (Phase E) | `embedded` on desktop (c.15+); CI `none` |

## Wyvern browser registry (local cache)

There is **no** OS-standard cross-platform browser list. Wyvern maintains its **own registry** — a small local file built from a **hardcoded catalog** of browsers we know how to find. This keeps CLI processing simple: `--viewer chrome` is a registry lookup, not ad-hoc path probing on every invocation.

### Catalog (source of truth in code)

Static table of known `id` values and per-OS discovery recipes. Initial set (c.15):

| `id` | Display name | Discovery hints (examples) |
|------|--------------|----------------------------|
| `chrome` | Google Chrome | macOS app bundle; Win `StartMenuInternet`; Linux `.desktop` / PATH |
| `edge` | Microsoft Edge | macOS bundle; Win `msedge.exe`; Linux `.desktop` / Flatpak |
| `firefox` | Mozilla Firefox | macOS bundle; Win `Firefox.exe`; Linux `firefox` / Snap |
| `safari` | Safari | macOS bundle only |
| `brave` | Brave | Same pattern as Chrome-derived |
| `chromium` | Chromium | Packaged / distro paths |
| `opera` | Opera | Optional catalog entry (not in c.15 `--viewer` enum) |
| `vivaldi` | Vivaldi | Optional; catalog entry |

`--viewer` enum for c.15: `chrome`, `safari`, `edge`, `firefox` (+ `embedded`, `none`, `system`). Additional catalog ids may be enabled later without changing the registry format.

`system` is **not** stored in the file — always resolved at launch via `webbrowser::open(url)`.

### Registry file (runtime cache)

| Property | Value |
|----------|-------|
| **Path** | Platform cache dir, e.g. `{cache}/wyvern/browsers.json` (use `dirs` crate) |
| **Written** | First run that needs named-browser resolution; on explicit refresh; on cache miss after failed launch |
| **Read** | Every `--viewer <id>` where `id` is a catalog browser |

Example shape (`BrowserRegistryFile` in [HTTP-TYPES.md](HTTP-TYPES.md)):

```json
{
  "version": 1,
  "updated_at": "2026-07-13T19:00:00Z",
  "platform": "macos-aarch64",
  "entries": [
    {
      "id": "chrome",
      "name": "Google Chrome",
      "executable": "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
    },
    {
      "id": "firefox",
      "name": "Firefox",
      "executable": "/Applications/Firefox.app/Contents/MacOS/firefox"
    }
  ]
}
```

Only browsers **found** on disk are written. Missing catalog entries are omitted (not errors until user requests that `id`).

### Refresh policy

| Trigger | Action |
|---------|--------|
| **First run** | Registry missing → run full catalog scan → write file |
| **`--viewer <id>` miss** | Re-scan catalog once → update file → retry; still missing → error |
| **`wyvern browsers refresh`** | Force full re-scan (c.15) |
| **User install later** | Next miss or `refresh` picks up new browser — no daemon |

No background watcher required for v0.1.

### CLI helpers (c.15)

| Command | Purpose |
|---------|---------|
| `wyvern browsers list` | Print registry entries (`id`, `name`, `executable`) |
| `wyvern browsers refresh` | Re-run catalog scan and rewrite cache |

Enables agents and users to see valid `--viewer` targets without reading docs.

### Overrides

| Override | Effect |
|----------|--------|
| `WYVERN_CHROME_PATH`, `WYVERN_EDGE_PATH`, … | Skip discovery for that `id`; still recorded in registry on refresh |
| `WYVERN_BROWSERS_FILE` | Alternate registry path (testing) |
| Manual edit of `browsers.json` | Supported; `refresh` overwrites unless entry marked — *defer custom merge rules to post-v0.1* |

### Launch flow

```text
--viewer system  → webbrowser::open(url)     # no registry
--viewer chrome  → load browsers.json
                 → miss? scan catalog → write → lookup chrome
                 → spawn executable with URL arg (platform-specific)
--viewer embedded → wyvern CLI spawns wyvern-viewer subprocess (not wyvern-host)
--viewer none    → WYVERN_DIALOG_URL only (wyvern-host sets env)
```

Use **`webbrowser`** for `system` only. Do **not** use the `open` crate for dialog URLs.

## Viewer close → host dismiss (REQ-0097)

When the user closes the OS window without clicking a dialog button, the host must still complete with dismissed semantics. Page `beforeunload`/`sendBeacon` covers browser tabs only — **not** `wyvern-viewer` OS close.

**Locked protocol (c.15 + d.8 wizard dismiss):**

1. **`wyvern-viewer`** registers a close handler (`dismiss.rs` / OS-close in `run.rs`):
   - **Wizard session** (`dialog_url` path under `/wizard/`):
     1. `GET /api/wizard/state` → read `page`, `page_data`, and prior `stack`
     2. Build full visited stack = prior `stack` + `{ page, data: page_data }` (d.2 finish algorithm)
     3. `POST /api/wizard/finish` with `{ "button": "dismissed", "data": <page_data>, "stack": <full visited stack> }` **before** process exit
     4. Host validates stack; stdout is `{ "button": "dismissed", "data": {}, "stack": <full visited> }`
   - **Blocking dialog** (`/message/`, `/input/`, …): `POST /api/result` with `{ "button": "dismissed" }` only
2. **`wyvern` CLI** watches the viewer child process. If the child exits without the host having received a result:
   - **One-shot:** call `DialogHandle::viewer_exited_without_result()` (in-process — see [HTTP-TYPES.md](HTTP-TYPES.md)).
   - **Wizard:** host derives dismissed via session `finish(Dismissed, page_data, derived_stack)` (full visited stack — not prior-only).
   - **Persistent (`--interactive` / `--mcp`):** CLI posts `{ "button": "dismissed" }` to `POST /api/result` via localhost HTTP client.
3. **`wyvern-host`** REQ-0097: if no result POST arrives before session timeout, map to dismissed (wizard = full visited stack, same as viewer-exit fallback).

Cross-links: [c15-wyvern-viewer.md](c15-wyvern-viewer.md), [d8-viewer-dismiss.md](../phase-D/d8-viewer-dismiss.md), [http-wizard-contract.md](http-wizard-contract.md) (finish / dismissed stack algorithm), [e2-blocking-question.md](../phase-E/e2-blocking-question.md).

### Implementation modules (c.15 + d.8)

```text
wyvern-host/src/
  browser_launch.rs   # system + named browser dispatch only
  browser_catalog.rs  # hardcoded id → discovery recipes
  browser_registry.rs # read/write cache, refresh, lookup
  session.rs          # wizard dismissed_on_exit_or_timeout (REQ-0097)

wyvern/src/
  viewer_spawn.rs     # embedded_viewer_spawn — subprocess + binary discovery

wyvern-viewer/src/
  main.rs             # binary entry — env/args → run, ExitCode
  run.rs              # URL navigate, show/hide; calls dismiss on OS-close
  dismiss.rs          # wizard finish stack vs blocking /api/result POST
  platform.rs         # wry/winit window + OS-close wiring
```

Optional: borrow path tables from **`browser-locations`** internally, but **Wyvern owns the cache file** — no dependency on an external registry service.

## Errors

If a named browser is not installed:

```text
viewer 'chrome': Google Chrome not found; install Chrome or use --viewer system
```

Exit via existing CLI error mapping (`HOST_VIEWER_ERROR` or equivalent — finalize in c.15).

## Implementation owner

| Mode | Crate |
|------|-------|
| `embedded` | **`wyvern`** spawns **`wyvern-viewer`** subprocess — binary discovery below |
| `none` | `wyvern-host` — set `WYVERN_DIALOG_URL`, no subprocess |
| `system`, `chrome`, … | `wyvern-host/src/browser_launch.rs` + registry modules |

`wry` / `winit` never enter `wyvern-host` or `wyvern`.

### Binary discovery (`wyvern` CLI, c.15)

| Layout | Resolution order |
|--------|------------------|
| Release install | Sibling `wyvern-viewer` next to `wyvern` binary in tarball |
| Dev workspace | `target/debug/wyvern-viewer` or `target/release/wyvern-viewer` via `CARGO_BIN_EXE_wyvern-viewer` at build time |
| Override | `WYVERN_VIEWER_BIN` env → explicit path |
| Fallback | `PATH` lookup for `wyvern-viewer` |

AC: missing binary → clear stderr + exit `HOST_VIEWER_ERROR` (not silent fallback to `none`).

## Authority cross-links

- [c9-testing-headless.md](c9-testing-headless.md) — CI uses `none`
- [c15-wyvern-viewer.md](c15-wyvern-viewer.md) — `embedded` implementation
- [http-dialog-contract.md](http-dialog-contract.md) — `WYVERN_DIALOG_URL` when `none`
- [../../wyvern-host/requirements.md](../../wyvern-host/requirements.md) — REQ-0105+
