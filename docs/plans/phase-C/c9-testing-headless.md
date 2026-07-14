# Headless browser testing strategy (c.10+)

Wyvern dialogs are HTTP pages. Tests should **not** spawn native windows, `wry`, or `xvfb` flocks.

**Product default:** `--viewer embedded` (wyvern-viewer; lands c.15).

**CI / agents / headless e2e:** `--viewer none` or `WYVERN_VIEWER=none` — never the product default.

See [http-viewer-contract.md](http-viewer-contract.md) for full `--viewer` enum and browser discovery.

---

## Test layers

| Layer | Tool | What it proves |
|-------|------|----------------|
| **L1 — API** | `reqwest` in `wyvern-host` `#[tokio::test]` | Routes, JSON shape, result unblocks `run()` |
| **L2 — Headless UI** | Playwright or Puppeteer | Page loads, button click, stdout JSON end-to-end |
| **L3 — Dev debug** | Cursor integrated browser (MCP) | Human/agent inspects live dialog URL without popups |

L1 runs on every `cargo test`. L2 runs in CI and pre-merge. L3 is optional local debugging.

---

## L1 — HTTP client only (Rust)

No browser. Fast, deterministic.

```text
spawn wyvern-host in-process OR wyvern subprocess with --viewer none
GET  http://127.0.0.1:{port}/api/dialog  → assert JSON
POST http://127.0.0.1:{port}/api/result   → assert CommandResult
```

Use for: schema/host contract, error paths, bind edge cases.

---

## L2 — Headless browser (CI E2E)

**Recommended: Playwright** (Chromium headless, good CI on ubuntu/macos/windows).

**Alternative: Puppeteer** — acceptable if repo already has Node harness.

### Flow

```text
1. wyvern '{"type":"message",...}' --viewer none &
2. Read dialog URL from WYVERN_DIALOG_URL (host sets when --viewer none) or parse stderr
3. Headless browser → page.goto(url)
4. page.click('#btn-ok')   // stable selectors in ui/message/
5. Wait for wyvern process exit 0
6. Assert stdout {"button":"ok"}
```

### Repo layout (**c.10** — first Playwright gate)

```text
tests/e2e/
  package.json          # playwright or puppeteer
  playwright.config.ts
  message.spec.ts
```

CI job (ubuntu): install Chromium via Playwright, no `xvfb`, no `WYVERN_AUTO_DISMISS`.

### UI requirements for automation

Package templates **must** expose stable hooks:

- `data-testid="btn-ok"` (or `id` per button label)
- No timing-only dismiss; result POST on click

Document in [http-dialog-contract.md](http-dialog-contract.md).

---

## L3 — Cursor integrated browser (debug)

For local development and agent debugging:

1. Run `wyvern '...' --viewer none` in terminal.
2. Open logged URL in **Cursor browser MCP** (`browser_navigate`).
3. Inspect DOM, click controls, verify POST — **no OS window spam**.

Not a CI gate — dev ergonomics only.

---

## What we delete (already in deletion inventory)

- `#[serial]` GUI tests in `crates/wyvern/tests/cli_validation.rs`
- `serial_test` dependency
- `wyvern-window/tests/*` (entire crate)
- macOS GUI `flock` in `support.rs`
- `WYVERN_AUTO_DISMISS` as primary test strategy (keep env for emergency smoke only)

---

## Host test hooks (implement in c.10)

| Hook | Purpose |
|------|---------|
| `--viewer none` | Never open browser/webview |
| `WYVERN_DIALOG_URL` | Host writes full URL before blocking (e2e reads this) |
| Optional `--print-url-only` | Exit 0 after printing URL (debug; no block) — *optional c.10 stretch* |

---

## CI matrix impact

| Leg | L1 | L2 headless |
|-----|----|-------------|
| ubuntu | `cargo test -p wyvern-host` | Playwright job |
| macos | same | Playwright job |
| windows | same | Playwright job |

Drop `--test-threads=1` requirement for dialog tests once GUI tests are gone.

---

## Sprint ownership

| Concern | Owner sprint |
|---------|--------------|
| L1 HTTP client tests | c.10+ per-type sprints |
| L2 Playwright/Puppeteer harness (`tests/e2e/`, specs) | **c.10** (first gate); extend per c.11–c.14 |
| L3 Cursor browser MCP | Dev only — not a sprint deliverable |

This doc is **strategy only** — no acceptance checklist here. Merge gates live in per-sprint docs (c.10+).
