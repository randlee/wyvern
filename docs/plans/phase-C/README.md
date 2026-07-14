# Phase C — Polish & Release v0.1.0 (`integrate/phase-C`)

Phase C release implementation PRs (**c.1–c.5**) target **`integrate/phase-C`**. Post-release error-handling fix PRs (**c.6–c.8**) target **`integrate/phase-C-fixes`**. This directory is the **sole authority** for sprint-level deliverables, acceptance criteria, and validation. `docs/plans/project-plan.md` carries phase-level goals and acceptance criteria only.

Release sprints are **sequentially numbered** `c.1` → `c.5`. Dependency graph:

```
Phase B ──┬──► c.1 ──► c.2 ──┐
          │                   ├──► c.4 ──► c.5
          └──► c.3 ───────────┘
```

- **c.1 → c.2:** icon asset bundle, then named-icon validation and resolution
- **c.3:** independent after Phase B (Win/Linux chrome does not block on c.1–c.2)
- **c.4:** depends on c.1, c.2, and c.3
- **c.5:** depends on c.4

## What Phase C closes (delivery rewrite c.9–c.16)

**Authoritative after c.8.** Prior c.1–c.5 work shipped on `wyvern-window`; that stack is **deleted in c.9**. v0.1.0 tag is valid only after **c.16**.

- HTTP dialog host (`wyvern-host`) + packaged `share/wyvern/ui/` for all dialog types
- Headless CI via Playwright/Puppeteer (`--viewer none`); embedded viewer default (c.15)
- `wyvern-window` entirely removed; no wry IPC, no inline HTML in Rust
- v0.1.0 release tarball with binary + UI bundle

### Historical (c.1–c.5 — merged on old stack, superseded)

- Icon bundle + named-icon validation (REQ-0030/0031) — **deprecated** on HTTP path; icons live in `ui/`
- Win/Linux wry chrome IPC (c.3) — **deleted** with `wyvern-window`; chrome in `ui/chrome/` (c.14)
- c.5 release tag tooling — reused in c.16 with updated artifacts

## What Phase C inherits from Phase B (semantics — transport changes at c.10)

- All four dialog types remain executable end-to-end (`message`, `input`, `markdown`, `question`) — **HTTP transport** replaces wry IPC (c.10+)
- Dialog sizing hints: **min 320×200**, **max 800×600** (REQ-0041) — enforced in template CSS / viewer hints, not wry attrs
- `chrome` fixed **480×360** open, **800×600** max — product constants in `ui/chrome/` (c.14)
- Modal behavior unchanged — minimize/maximize policy in template JS + viewer config (REQ-0083 semantics)
- Phase B [ipc-dialog-contract.md](../phase-B/ipc-dialog-contract.md) — **historical**; HTTP contracts authoritative after c.10

## What Phase C does not close

- `wizard` type — **Phase D**
- `--interactive` / lifecycle actions — **Phase E**
- MCP server — **Phase E**
- AI-generated icons — post-MVP (PRD)
- Homebrew formula tap (optional stretch; README documents install from GitHub release as minimum bar)

## Post-release fixes (`integrate/phase-C-fixes`)

After v0.1.0 release merge, error-handling hardening continues on **`integrate/phase-C-fixes`** (not `integrate/phase-C`):

```
c.5 (release) ──► c.6 ──► c.7
                     └──► c.8
```

| Sprint | Title | Doc | Status |
|--------|-------|-----|--------|
| c.6 | Result propagation — eliminate production unwrap/expect | [c6-result-propagation.md](c6-result-propagation.md) | complete |
| c.7 | CLI integration test hardening (`serial_test` for GUI-spawning CLI tests) | [c7-cli-test-hardening.md](c7-cli-test-hardening.md) | complete |
| c.8 | Clippy deny unauthorized panics in lib/`main` src | [c8-clippy-deny-unwrap.md](c8-clippy-deny-unwrap.md) | complete |

- **c.6:** production `Result` propagation + structured stderr emit boundary (REQ-0078)
- **c.7:** depends on c.6; local macOS CLI test serialization (CI already uses `--test-threads=1`)
- **c.8:** depends on c.6; clippy deny regression gate (parallel with c.7)

## Delivery rewrite (c.9–c.16) — HTTP host

Implementation PRs for **c.9–c.16** target **`integrate/phase-c-web-server`** (not `integrate/phase-C`).

The embedded `wyvern-window` stack is **removed in c.9** (clean break; compile optional). **Principle: delete → verify → rebuild** — no refactor-in-place. Replacement: `wyvern-host` + packaged `ui/` (c.10–c.14) + `wyvern-viewer` (c.15) + release (c.16).

```
c.8 ──► c.9 ──► c.10 ──► c.11 ──► c.12 ──► c.13 ──► c.14 ──► c.15 ──► c.16 ──► v0.1.0
        │         │         │         │         │         │         │
        delete    message   input     md        question  chrome    viewer   release
        only      +host
```

| Sprint | Title | Doc | Status |
|--------|-------|-----|--------|
| c.9 | **Delete** `wyvern-window` (compile optional) | [c9-deletion.md](c9-deletion.md), [c9-deletion-and-rework.md](c9-deletion-and-rework.md) | complete |
| c.10 | `wyvern-host` + `message` + workspace green | [c10-http-host-message.md](c10-http-host-message.md), [c9-testing-headless.md](c9-testing-headless.md) | complete |
| c.11 | `input` (+ picker) | [c11-host-input.md](c11-host-input.md) | complete |
| c.12 | `markdown` | [c12-host-markdown.md](c12-host-markdown.md) | complete |
| c.13 | `question` | [c13-host-question.md](c13-host-question.md) | complete |
| c.14 | `chrome` — full dialog matrix | [c14-host-chrome.md](c14-host-chrome.md) | complete |
| c.15 | `wyvern-viewer` + browser registry | [c15-wyvern-viewer.md](c15-wyvern-viewer.md), [http-viewer-contract.md](http-viewer-contract.md) | complete |
| c.16 | Release bundle + **v0.1.0** (final Phase C) | [c16-release.md](c16-release.md) | implemented |

**c.9 merge gate:** deletion inventory passes; `wyvern-window` absent. **`cargo build` not required.**

**c.10+ merge gate:** workspace compile + CI green; each type sprint adds one headless e2e spec.

**Viewer default:** product CLI uses `embedded` (c.15); CI/e2e use `none`. See [http-viewer-contract.md](http-viewer-contract.md).

- **Contracts:** [HTTP-TYPES.md](HTTP-TYPES.md), [http-dialog-contract.md](http-dialog-contract.md), [http-post-schema.md](http-post-schema.md), [http-viewer-contract.md](http-viewer-contract.md), [http-wizard-contract.md](http-wizard-contract.md), [http-interactive-mcp-contract.md](http-interactive-mcp-contract.md)
- **Crate:** `wyvern-host` — see [../../wyvern-host/architecture.md](../../wyvern-host/architecture.md)
- **c.1–c.3, c.7 GUI flock:** historical; not extended. Icons/chrome REQ-0030/0080+ deprecated for host path.

## Platform policy (HTTP delivery — c.14+)

| Platform | Viewer | Chrome |
|----------|--------|--------|
| macOS | `wyvern-viewer`: transparent title bar; native traffic lights | Template + viewer attrs |
| Windows | `wyvern-viewer` or system browser | `ui/` HTML close/minimize |
| Linux | Same as Windows | Same as Windows |

Modal minimize policy lives in **template JS** and viewer configuration — not wry IPC.

## Phase B → Phase C handoff (historical — pre-c.9)

## Phase acceptance criteria (smoke) — delivery rewrite (c.16)

All commands use `--viewer none` in CI unless noted.

1. `wyvern '{"type":"message",...}' --viewer none` → headless e2e; OK → `{"button":"ok"}`
2. `input`, `markdown`, `question`, `chrome` — each passes headless e2e (c.11–c.14)
3. `wyvern '{"type":"message",...}'` (default) — embedded viewer smoke on macOS (c.15)
4. Tag `v0.1.0` produces binary + `share/wyvern/ui/**` on macOS, Windows, Linux (c.16)
5. `wyvern-window` absent; `cargo build --workspace` green on `integrate/phase-c-web-server` head

### Historical (c.1–c.5 — pre-deletion stack)

1. Production warning icon via Rust catalog — **superseded** by template-owned icons in `ui/`
2. Named icon validation against Rust catalog — **superseded**; `icon` is opaque string (REQ-0102)
3. xvfb + wry GUI matrix — **removed**; HTTP headless replaces

## Sprint index (c.1–c.5)

| Sprint | Doc | Branch (pattern) |
|--------|-----|------------------|
| c.1 | [c1-icon-set.md](c1-icon-set.md) | `feature/phase-C-c1-icon-set` |
| c.2 | [c2-icon-resolution.md](c2-icon-resolution.md) | `feature/phase-C-c2-icon-resolution` |
| c.3 | [c3-win-linux-chrome.md](c3-win-linux-chrome.md) | `feature/phase-C-c3-win-linux-chrome` |
| c.4 | [c4-nfr-validation.md](c4-nfr-validation.md) | `feature/phase-C-c4-nfr-validation` |
| c.5 | [c5-release.md](c5-release.md) | `feature/phase-C-c5-release` |

## Cross-cutting contracts

| Doc | Purpose |
|-----|---------|
| [chrome-ipc-contract.md](chrome-ipc-contract.md) | **Historical** — wry chrome IPC (deleted with `wyvern-window`) |
| [../phase-B/ipc-dialog-contract.md](../phase-B/ipc-dialog-contract.md) | **Historical** — Phase B dialog IPC |
| [HTTP-TYPES.md](HTTP-TYPES.md) | Rust types for host, viewer, registry, wizard |
| [../../wyvern-host/architecture.md](../../wyvern-host/architecture.md) | ADR-0016/0017 crate detail |

## CI validation (authoritative)

**After c.10 (HTTP host):**

| Leg | L1 | L2 headless |
|-----|----|-------------|
| `ubuntu-latest` | `cargo test -p wyvern-host` + workspace unit tests | Playwright (`--viewer none`) |
| `macos-latest` | same | Playwright |
| `windows-latest` | same | Playwright |

All legs: `cargo build --workspace`, `cargo clippy --workspace -- -D warnings`, `sc-lint check native --config .sc-lint.toml`.

No `xvfb`, no `--test-threads=1` gate for dialog tests once GUI tests are deleted (c.9).

**Historical (c.1–c.8 — pre-deletion):** xvfb + `cargo test --workspace -- --test-threads=1` on all legs.

After c.16: release workflow (see [c16-release.md](c16-release.md)) validates on tag push — not on every PR.

## Post–Phase C follow-up (error handling)

Phase C merged with production `expect`/`unreachable!` in hot paths. Follow-up sprints target **`integrate/phase-C-fixes`** off `develop`:

```
develop ──► integrate/phase-C-fixes ──► c.6 ──► c.7 ──► c.8
```

| Sprint | Doc | Focus |
|--------|-----|-------|
| c.6 | [c6-result-propagation.md](c6-result-propagation.md) | Eliminate production panics; `Result` through media + emit |
| c.7 | [c7-cli-test-hardening.md](c7-cli-test-hardening.md) | `serial_test` on nine GUI CLI tests; shared spawn helper |
| c.8 | [c8-clippy-deny-unwrap.md](c8-clippy-deny-unwrap.md) | Clippy deny on four roots (three lib + `main.rs`) |

**Sole authority** for deliverables, acceptance criteria, and validation: **c.6, c.7, c.8 sprint docs above.**

Reference only: [ERROR-HANDLING-PLAN.md](ERROR-HANDLING-PLAN.md) (policy + context), [UNWRAP-INVENTORY.md](UNWRAP-INVENTORY.md) (audit trail).

### NFR measurement (c.4 — macOS dev/CI optional job)

| NFR | Target | Measurement |
|-----|--------|-------------|
| NFR-0001 | Window open < 500ms | macOS manual measurement or `load_finished` hook (product target); **not** auto-dismiss timing; optional non-blocking CI job may use 2000ms smoke bound |
| NFR-0002 | Resident memory < 80MB | `ps -o rss= -p $(pgrep -x wyvern)` after **≥ 2s** settle post first paint (RSS KB ÷ 1024); Activity Monitor acceptable |
| NFR-0003 | Binary < 10MB | `ls -lh target/release/wyvern` on macOS release build |

NFR-0004–NFR-0007 remain satisfied by existing architecture; c.4 confirms no regression.

## sc-lint-boundary

- **c.10:** add `boundaries/wyvern-host/host.toml`; remove `wyvern-window` boundary
- **c.15:** add `wyvern-viewer` boundary when crate lands
- **Historical c.1/c.3:** asset paths and `PlatformChrome` IPC — deleted with `wyvern-window`
