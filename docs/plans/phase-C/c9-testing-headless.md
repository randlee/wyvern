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

### Design rules

- **We do not design tests that block until test infrastructure shuts them down.** No spec should spawn a dialog and wait for Playwright timeout (60s) or session idle timeout (30s) to finish the run. Every L2 test **actively drives** the dialog to completion (~1s).
- A **blocking message** (or any blocking dialog) **must be closed by clicking a button** (or equivalent submit). Wyvern does not auto-complete blocking dialogs in headless mode.
- Playwright `timeout: 60_000` and headless `session_timeout: 30s` are **hang detectors / misconfiguration fuses** — if they fire, the run failed; they are never the designed completion path.
- When headless idle timeout fires, Wyvern exits **non-zero** (`SESSION_TIMEOUT_ERROR`, exit **6**) — CI must treat this as **test FAIL**.

### Anti-patterns (do not write)

| Anti-pattern | Why |
|--------------|-----|
| Spawn blocking JSON, no harness, assert after timeout | Not a test — infrastructure cleanup |
| Use Playwright timeout as the pass condition | Hung test masked as pass |
| Use `WYVERN_AUTO_DISMISS` as primary CI strategy | Bypasses real user flow |
| L2 spec with no `goto` + click / POST | Undriven dialog |

L1 host tests may use a **short** explicit `session_timeout` plus HTTP-only polling to verify **product** idle-dismiss semantics (embedded viewer). That is not an L2 e2e pattern — never copy it into Playwright specs.

### Flow

```text
1. wyvern '{"type":"message",...}' --viewer none &
2. Read dialog URL from WYVERN_DIALOG_URL (host sets when --viewer none) or parse stderr
3. Headless browser → page.goto(url)
4. page.getByTestId("btn-ok").click()   // ~1s end-to-end when wired correctly
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
