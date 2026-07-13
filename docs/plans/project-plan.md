# Wyvern — MVP Project Plan

A sprint is a single testable deliverable that fits within one AI context window (~200k tokens) and represents 1–5 days of focused work. Each sprint has explicit acceptance criteria that must pass before the next sprint begins.

**sc-lint-boundary** is a planning activity applied from Phase B onwards — architectural boundary rules are reviewed at sprint planning, not implemented as a sprint.

**Review and hardening principle:** If something feels complicated, assume the design is unclear or overspecified before assuming more API is needed. Reviews should attack complication directly by collapsing semantic drift, clarifying contracts, and defending the smallest coherent command surface.

**Integration branch map:**

| Integration branch | Project plan phase | Sprint docs |
|---|---|---|
| `integrate/phase-A` | Phase A — Foundation | `docs/plans/phase-A/` |
| `integrate/phase-B` | Phase B — Core Dialogs | `docs/plans/phase-B/` |
| `integrate/phase-C` | Phase C — Release v0.1.0 | `docs/plans/phase-C/` |
| `integrate/phase-D` | Phase D — Wizard | `docs/plans/phase-D/` |
| `integrate/phase-E` | Phase E — Persistent & MCP | `docs/plans/phase-E/` |

Phase A sprint PRs target `integrate/phase-A`. Sprint authority: `docs/plans/phase-A/` (sprints **a.1–a.7**).

---

## Phase A — Foundation

**Phase goal:** Cross-platform foundation binary with HTML chrome frame and validated JSON I/O on a **single direct path**. Only `type: "chrome"` is executable. Win/Linux decoration polish deferred to Phase C.

**Execution model:** `load (LoadError) → validate (ValidationError) → Command → run (RunError) → CommandResult → stdout`. One `type` → one handler. No CLI flags, no stub handlers.

**Phase acceptance criteria:**

1. `wyvern '{"type":"message",...}'` → validation stderr, exit ≠ 0, no window
2. `wyvern '{"type":"chrome","title":"Foundation"}'` → chrome opens; OS close → `{"button":"dismissed"}`
3. `wyvern '{"type":"unknown"}'` → validation stderr on `type`, exit ≠ 0, no window

**Platform:** Cross-platform code patterns + CI `cargo test --workspace` on ubuntu, macos, and windows. Win/Linux validation is CI-automated only (no manual E2E). Optional macOS manual chrome smoke during dev. Win/Linux decoration polish → Phase C.

**Sprints:** seven active (**a.1–a.7**). See [docs/plans/phase-A/README.md](phase-A/README.md).

| Sprint | Title | Doc |
|--------|-------|-----|
| a.1 | Workspace scaffold | [a1-scaffold.md](phase-A/a1-scaffold.md) |
| a.2 | Native window (tests) | [a2-window.md](phase-A/a2-window.md) |
| a.3 | JSON loading | [a3-json-io.md](phase-A/a3-json-io.md) |
| a.4 | Validation (`chrome`) | [a4-validation.md](phase-A/a4-validation.md) |
| a.5 | Chrome E2E | [a5-chrome-frame.md](phase-A/a5-chrome-frame.md) |
| a.6 | sc-observability | [a6-sc-observability.md](phase-A/a6-sc-observability.md) |
| a.7 | sc-lint | [a7-sc-lint.md](phase-A/a7-sc-lint.md) |

---

## Phase B — Core Dialogs (MVP)

**Phase goal:** All four dialog types (`message`, `input`, `markdown`, `question`) work end-to-end from the CLI. This is the first genuinely useful version of Wyvern.

**Phase acceptance criteria:** A developer can replace any `zenity`/`osascript` dialog call with a `wyvern` command and get a richer, JSON-returning equivalent. Numbered smoke checks: [docs/plans/phase-B/README.md](phase-B/README.md#phase-acceptance-criteria-smoke).

*sc-lint-boundary applied at sprint planning from this phase forward.*

Phase B sprint PRs target `integrate/phase-B`. Sprint authority: `docs/plans/phase-B/` (sprints **b.1–b.8**, sequential — not parallel sub-sprints).

**Sprints:** eight active (**b.1–b.8**). See [docs/plans/phase-B/README.md](phase-B/README.md).

| Sprint | Title | Doc |
|--------|-------|-----|
| b.1 | Message structure + buttons | [b1-message-structure.md](phase-B/b1-message-structure.md) |
| b.2 | Message icons + markdown body | [b2-message-icons.md](phase-B/b2-message-icons.md) |
| b.3 | Input text mode | [b3-input-text.md](phase-B/b3-input-text.md) |
| b.4 | Input file/folder picker | [b4-input-picker.md](phase-B/b4-input-picker.md) |
| b.5 | Markdown file + `.md` shorthand | [b5-markdown-file.md](phase-B/b5-markdown-file.md) |
| b.6 | Markdown inline + stylesheet | [b6-markdown-inline.md](phase-B/b6-markdown-inline.md) |
| b.7 | Question cards (radio/checkbox) | [b7-question-render.md](phase-B/b7-question-render.md) |
| b.8 | Question preview + compliance | [b8-question-preview.md](phase-B/b8-question-preview.md) |

---

## Phase C — Polish & Release v0.1.0

**Phase goal:** Wyvern ships as a usable, cross-platform CLI tool. Full icon set (REQ-0030). Win/Linux platform chrome (ADR-0010a). Binaries available for download.

**Phase acceptance criteria:** Install from GitHub release (or equivalent) works; a developer can run all Phase B dialog types on macOS, Windows, and Linux from a released binary. See [docs/plans/phase-C/README.md](phase-C/README.md#phase-acceptance-criteria-smoke).

Phase C sprint PRs target `integrate/phase-C`. Sprint authority: `docs/plans/phase-C/` (sprints **c.1–c.5**, sequential — not parallel sub-sprints).

**Inherited from Phase B:** Dialog auto-size **min 320×200** / **max 800×600**; Win/Linux native OS decorations until c.3; b.2 placeholder icons at `assets/icons/placeholder/` until c.1 production bundle.

**Sprints:** five active (**c.1–c.5**). See [docs/plans/phase-C/README.md](phase-C/README.md).

| Sprint | Title | Doc |
|--------|-------|-----|
| c.1 | Production icon asset bundle | [c1-icon-set.md](phase-C/c1-icon-set.md) |
| c.2 | Full icon field resolution | [c2-icon-variants.md](phase-C/c2-icon-variants.md) |
| c.3 | Windows and Linux platform chrome | [c3-win-linux-chrome.md](phase-C/c3-win-linux-chrome.md) |
| c.4 | Cross-platform validation and NFR pass | [c4-cross-platform-validation.md](phase-C/c4-cross-platform-validation.md) |
| c.5 | Release tooling and v0.1.0 | [c5-release.md](phase-C/c5-release.md) |

---

## Phase D — Wizard

**Phase goal:** Multi-page wizards with branching navigation and data persistence across pages.

**Phase acceptance criteria:** The example DAG layout-picker wizard completes a full flow with branching, back-navigation, data restoration, and returns the correct stack JSON.

Phase D sprint PRs target `integrate/phase-D`. Sprint authority: `docs/plans/phase-D/` (sprints **d.1–d.6**, sequential — not parallel sub-sprints).

**Sprints:** six active (**d.1–d.6**). See [docs/plans/phase-D/README.md](phase-D/README.md).

| Sprint | Title | Doc |
|--------|-------|-----|
| d.1 | Wizard host: HTML load and config injection | [d1-wizard-host.md](phase-D/d1-wizard-host.md) |
| d.2 | Wizard IPC contract | [d2-wizard-ipc.md](phase-D/d2-wizard-ipc.md) |
| d.3 | Browser-history navigation model | [d3-history-nav.md](phase-D/d3-history-nav.md) |
| d.4 | Stack injection and data restoration | [d4-stack-inject.md](phase-D/d4-stack-inject.md) |
| d.5 | Example DAG layout-picker wizard | [d5-dag-example.md](phase-D/d5-dag-example.md) |
| d.6 | Wizard polish and edge cases | [d6-wizard-polish.md](phase-D/d6-wizard-polish.md) |

---

## Phase E — Interactive & MCP

**Phase goal:** Wyvern runs as a persistent process, driveable by agents over stdin or as an MCP server.

**Phase acceptance criteria:** A Claude Code agent can open Wyvern in `--interactive` mode from a background shell, issue multiple blocking dialog commands against one persistent process, receive the JSON results, and exit — with no MCP required.

Phase E sprint PRs target `integrate/phase-E`. Sprint authority: `docs/plans/phase-E/` (sprints **e.1–e.4**, sequential — not parallel sub-sprints).

**Sprints:** four active (**e.1–e.4**). See [docs/plans/phase-E/README.md](phase-E/README.md).

| Sprint | Title | Doc |
|--------|-------|-----|
| e.1 | `--interactive` stdin loop and lifecycle actions | [e1-interactive-loop.md](phase-E/e1-interactive-loop.md) |
| e.2 | Blocking dialogs and `exit` in interactive mode | [e2-blocking-question.md](phase-E/e2-blocking-question.md) |
| e.3 | MCP server wrapper and tool mapping | [e3-mcp-server.md](phase-E/e3-mcp-server.md) |
| e.4 | MCP persistent window and integration testing | [e4-mcp-persistent.md](phase-E/e4-mcp-persistent.md) |

---

## Phase Summary

| Phase | Sprints | Ships |
|-------|---------|-------|
| Phase A — Foundation | 7 | Working binary, `chrome` command |
| Phase B — Core Dialogs | 8 | **MVP — all dialog types usable** |
| Phase C — Release v0.1.0 | 5 | **v0.1.0 on mac/win/linux** |
| Phase D — Wizard | 6 | Multi-page wizard with branching |
| Phase E — Interactive & MCP | 4 | Agent-driveable status viewer + MCP |

## Dependency Map

```
Phase A
  └─ Phase B ──────────────────── sc-lint-boundary applied from here
       └─ Phase C (v0.1.0 release)
            └─ Phase D (wizard)
                 └─ Phase E (interactive + MCP)
```
