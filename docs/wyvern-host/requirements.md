# `wyvern-host` — Requirements

*Part of the [principal requirements](../requirements.md).*

**Status:** Planned (c.10+). Supersedes `wyvern-window` delivery requirements (REQ-0030–0032, REQ-0080–0087, wry IPC).

---

## HTTP dialog host (REQ-0090 – REQ-0099)

**REQ-0090** — After successful validation, the host binds an HTTP server before `run()` returns a `CommandResult`. Validation failures never start the server.

**REQ-0091** — Default bind address is loopback (`127.0.0.1`). Bind address and port are configurable via CLI flags documented in [docs/wyvern/requirements.md](../wyvern/requirements.md).

**REQ-0092** — Port `0` selects an ephemeral port. The effective URL is observable for tests and for `--viewer none` (stderr info line or documented env var).

**REQ-0093** — The host serves static UI files from a **UI root directory**. The release default is `share/wyvern/ui/` adjacent to the installed binary (exact layout in [http-dialog-contract.md](../plans/phase-C/http-dialog-contract.md)).

**REQ-0094** — End users may override UI root via CLI flag (`--ui-root`). Per-command JSON `ui_root` field deferred to post-v0.1 (non-closure in c.10–c.16).

**REQ-0095** — Dialog fields validated by `wyvern-schema` are exposed to the page as JSON at `GET /api/dialog`. The host does not perform `{{placeholder}}` substitution in HTML.

**REQ-0096** — The page submits the final result at `POST /api/result`. Body schema per type is in [http-post-schema.md](../plans/phase-C/http-post-schema.md) and must match stdout `CommandResult`. On accept, the host shuts down and completes `run()`.

**REQ-0097** — If the viewer closes without a result POST, the host completes with existing dismissed semantics (`button: "dismissed"` or type-specific equivalent per contract).

**REQ-0098** — One CLI invocation hosts one dialog session unless `--interactive` (Phase E) extends the same server — see ADR-0016.

**REQ-0099** — The host does not embed HTML templates, CSS, JS, or icons in Rust. It serves static UI bytes from disk and serializes JSON command fields. **Locked exception:** for `GET /api/dialog` only, the host may add server-side `content_html` (`markdown`, c.12) and `preview_html` (`question`, c.13) via `pulldown-cmark` + **`ammonia`** sanitization — not general HTML generation or inline template rendering.

---

## Packaged UI (REQ-0100 – REQ-0104)

**REQ-0100** — Release artifacts include default UI trees: `message/`, `input/`, `markdown/`, `question/`, `chrome/` (complete by c.14).

**REQ-0101** — Each UI tree is self-contained (HTML, CSS, JS, assets). Icons and chrome are owned by the template author.

**REQ-0102** — JSON fields such as `level`, `icon`, and `image` are passed through in `/api/dialog` as opaque hints. Interpretation is template responsibility.

**REQ-0103** — No Rust built-in icon catalog. Schema may treat `icon` as an optional opaque string (c.9 amendment); strict named-icon catalog (REQ-0030/0031) is **deprecated**.

**REQ-0104** — File paths in JSON are passed through to `/api/dialog`; templates resolve URLs/paths. Optional host static mount for user asset dirs is a c.10+ follow-up if needed.

---

## Viewers (REQ-0105 – REQ-0110)

**REQ-0105** — `--viewer` selects how the dialog URL is opened. Values: `embedded`, `none`, `system`, `chrome`, `safari`, `edge`, `firefox`. Deprecated alias: `browser` → `system`. Env `WYVERN_VIEWER` overrides when set. **Interim (c.10–c.14):** omitted flag defaults to **`none`**. **Product default from c.15:** **`embedded`**. Full contract: [http-viewer-contract.md](../plans/phase-C/http-viewer-contract.md).

**REQ-0106** — `none`: host does not open a viewer; sets `WYVERN_DIALOG_URL` for headless e2e (Playwright/Puppeteer) and remote-browser workflows. **CI and agent runs use `none`** — not the product default.

**REQ-0107** — `system`: host opens the OS default browser via `webbrowser` crate (optional dependency).

**REQ-0108** — `embedded`: **`wyvern` CLI** spawns **`wyvern-viewer`** subprocess to load the dialog URL (c.15). Host does **not** create webviews. **Default for interactive desktop use.**

**REQ-0109** — Named browsers (`chrome`, `safari`, `edge`, `firefox`): resolved via **Wyvern browser registry** — local cache file built from hardcoded catalog on first run / refresh. `--viewer <id>` is a registry lookup. Clear error when `id` not installed. See [http-viewer-contract.md](../plans/phase-C/http-viewer-contract.md).

**REQ-0110** — c.10–c.14: implement **`none` only**; omitted `--viewer` defaults to **`none`**. c.15 adds `embedded` (becomes default when flag omitted) and named browsers.

**REQ-0111** — `wyvern browsers list` / `wyvern browsers refresh` (c.15).

---

## Dialog execution (inherited semantics)

Dialog **types** and stdout shapes remain as in Phase B/C (`message`, `input`, `markdown`, `question`, `chrome`). Validation stays in `wyvern-schema`. Host-specific sizing (formerly REQ-0041) moves to template/CSS; host may suggest default window dimensions to `embedded` viewer only.

**REQ-0112** — Default UI templates include stable automation hooks (`data-testid` on primary actions) for headless Playwright/Puppeteer tests.

---

## File / folder picker (REQ-0113 – REQ-0114)

**REQ-0113** — `input` `mode: file` and `mode: folder` use OS-native pickers via `rfd` in `wyvern-host` only (same product semantics as Phase B b.4).

**REQ-0114** — Picker is triggered by host API route or host-handled POST from page (see [http-dialog-contract.md](../plans/phase-C/http-dialog-contract.md)), not by wry IPC.

---

## Deprecated (do not implement in `wyvern-host`)

| IDs | Former owner | Reason |
|-----|--------------|--------|
| REQ-0030, REQ-0031 | `wyvern-window` | Icons in packaged HTML |
| REQ-0080 – REQ-0087 | `wyvern-window` | Chrome in packaged HTML |
| Phase B IPC contract | `wyvern-window` | Replaced by HTTP contract |
