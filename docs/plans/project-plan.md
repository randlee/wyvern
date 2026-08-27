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
| `integrate/phase-F` | Phase F — CLI Extensions | `docs/plans/phase-F/` |
| `integrate/phase-G` | Phase G — Extension Agent Usability | `docs/plans/phase-G/` |
| `integrate/phase-E` | Phase E — Persistent & MCP | `docs/plans/phase-E/` |
| `integrate/phase-H` | Phase H — XHTML reporting & review | `docs/plans/phase-H/` |
| `integrate/phase-I` | Phase I — Wizard native path picker | `docs/plans/phase-I/` |
| `integrate/phase-J` | Phase J — sc-publish migration | `docs/plans/phase-J/` |

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

**Phase goal (revised c.9–c.16):** HTTP dialog host with packaged UI; optional embedded viewer; cross-platform headless CI. v0.1.0 after c.16.

**Historical goal (c.1–c.5, superseded):** Icon bundle (REQ-0030), Win/Linux wry chrome (ADR-0010a) — deleted with `wyvern-window` in c.9.

**Phase acceptance criteria:** See [delivery rewrite](phase-C/README.md#delivery-rewrite-c9c16--http-host) and [c.16 smoke](phase-C/README.md#phase-acceptance-criteria-smoke--delivery-rewrite-c16).

Phase C release sprint PRs (**c.1–c.5**) target `integrate/phase-C`. Post-release error-handling fix sprints (**c.6–c.8**) target `integrate/phase-C-fixes`. Sprint authority: `docs/plans/phase-C/`. Dependency graph:

```
Phase B ──┬──► c.1 ──► c.2 ──┐
          │                   ├──► c.4 ──► c.5 ──► c.6 ──► c.7
          └──► c.3 ───────────┘                      └──► c.8
```

- **c.1 → c.2:** icon asset bundle, then named-icon validation and resolution
- **c.3:** independent after Phase B (Win/Linux chrome does not block on c.1–c.2)
- **c.4:** depends on c.1, c.2, and c.3
- **c.5:** depends on c.4
- **c.6 → c.7 / c.8:** post-release Result propagation, then CLI test hardening and clippy deny gate (parallel after c.6)

**Inherited from Phase B:** Dialog auto-size **min 320×200** / **max 800×600**; Win/Linux native OS decorations until c.3; b.2 placeholder icons at `assets/icons/placeholder/` until c.1 production bundle.

> **Supersession (Phase D d.6 / ADR-0020 / REQ-V008):** Embedded-viewer auto-size is no longer capped at a hard **800×600** maximum. Dialog mode measures intrinsic content × ~25% slack and clamps to available viewport × 0.92. The **800×600** figures remain only as the browser / no-`viewport-bounds` fallback in `ui/shared/wyvern-api.js` (`VIEWER_MAX_W` / `VIEWER_MAX_H`) when the embedded viewer has not injected bounds.

**Sprints:** c.1–c.5 (historical, old stack) + c.6–c.8 (fixes) + **c.9–c.16 (delivery rewrite)**. See [docs/plans/phase-C/README.md](phase-C/README.md).

| Sprint | Title | Doc | Target branch |
|--------|-------|-----|---------------|
| c.1 | Production icon asset bundle | [c1-icon-set.md](phase-C/c1-icon-set.md) | `integrate/phase-C` |
| c.2 | Full icon field resolution | [c2-icon-resolution.md](phase-C/c2-icon-resolution.md) | `integrate/phase-C` |
| c.3 | Windows and Linux platform chrome | [c3-win-linux-chrome.md](phase-C/c3-win-linux-chrome.md) | `integrate/phase-C` |
| c.4 | Cross-platform validation and NFR pass | [c4-nfr-validation.md](phase-C/c4-nfr-validation.md) | `integrate/phase-C` |
| c.5 | Release tooling and v0.1.0 | [c5-release.md](phase-C/c5-release.md) | `integrate/phase-C` |
| c.6 | Result propagation (no production panics) | [c6-result-propagation.md](phase-C/c6-result-propagation.md) | `integrate/phase-C-fixes` |
| c.7 | CLI integration test hardening | [c7-cli-test-hardening.md](phase-C/c7-cli-test-hardening.md) | `integrate/phase-C-fixes` |
| c.8 | Clippy deny unauthorized panics | [c8-clippy-deny-unwrap.md](phase-C/c8-clippy-deny-unwrap.md) | `integrate/phase-C-fixes` |
| c.9 | Delete `wyvern-window` (compile optional) | [c9-deletion.md](phase-C/c9-deletion.md) | `integrate/phase-c-web-server` |
| c.10 | `wyvern-host` + `message` | [c10-http-host-message.md](phase-C/c10-http-host-message.md) | `integrate/phase-c-web-server` |
| c.11 | `input` on host | [c11-host-input.md](phase-C/c11-host-input.md) | `integrate/phase-c-web-server` |
| c.12 | `markdown` on host | [c12-host-markdown.md](phase-C/c12-host-markdown.md) | `integrate/phase-c-web-server` |
| c.13 | `question` on host | [c13-host-question.md](phase-C/c13-host-question.md) | `integrate/phase-c-web-server` |
| c.14 | `chrome` on host | [c14-host-chrome.md](phase-C/c14-host-chrome.md) | `integrate/phase-c-web-server` |
| c.15 | `wyvern-viewer` + browser registry | [c15-wyvern-viewer.md](phase-C/c15-wyvern-viewer.md) | `integrate/phase-c-web-server` |
| c.16 | Release + v0.1.0 | [c16-release.md](phase-C/c16-release.md) | `integrate/phase-c-web-server` |

---

## Phase C delivery rewrite (c.9–c.16)

**Phase goal (revised):** Usable cross-platform CLI via HTTP-packaged UI — not embedded wry IPC.

**Phase acceptance criteria (revised):** Full dialog matrix on HTTP host; release tarball includes `share/wyvern/ui/`; `wyvern-window` deleted; v0.1.0 after c.16.

See [docs/plans/phase-C/README.md](phase-C/README.md#delivery-rewrite-c9c16--http-host).

---

## Phase D — Wizard

**Phase goal:** Multi-page wizards with branching navigation and data persistence across pages.

**Transport:** [http-wizard-contract.md](phase-C/http-wizard-contract.md) on `wyvern-host`.

**Prerequisite:** Phase C **c.16** complete.

**Phase acceptance criteria:** The example DAG layout-picker wizard completes a full flow with branching, back-navigation, data restoration, and returns the correct stack JSON.

Phase D sprint PRs target `integrate/phase-D`. Sprint authority: `docs/plans/phase-D/` (sprints **d.1–d.8**, sequential).

**Sprints:** eight active (**d.1–d.8**). See [docs/plans/phase-D/README.md](phase-D/README.md).

| Sprint | Title | Doc |
|--------|-------|-----|
| d.1 | Wizard host — HTTP + initial stack snapshot | [d1-wizard-host.md](phase-D/d1-wizard-host.md) |
| d.2 | Wizard HTTP navigation + finish + browser stack | [d2-wizard-ipc.md](phase-D/d2-wizard-ipc.md) |
| d.3 | Browser-history regression tests | [d3-history-nav.md](phase-D/d3-history-nav.md) |
| d.4 | Page bootstrap + stack snapshot tests | [d4-stack-inject.md](phase-D/d4-stack-inject.md) |
| d.5 | Example DAG layout-picker wizard | [d5-dag-example.md](phase-D/d5-dag-example.md) |
| d.6 | Wizard viewport sizing | [d6-viewport-sizing.md](phase-D/d6-viewport-sizing.md) |
| d.7 | Shared wizard chrome | [d7-wizard-chrome.md](phase-D/d7-wizard-chrome.md) |
| d.8 | Wizard viewer dismiss | [d8-viewer-dismiss.md](phase-D/d8-viewer-dismiss.md) |

---

## Phase F — CLI Extensions

**Phase goal:** Declarative argv → validated `Command` JSON via an extension registry — file suffix defaults and subcommand aliases without new host dialog types.

**Prerequisite:** Phase D complete (wizard + `--ui-root`).

**Phase acceptance criteria:** `wyvern page.html`, `wyvern report.csv` (interactive JS table with sort/filter), `wyvern md report.csv`, and `wyvern compose render ...` (when `sc-compose` on PATH) all expand, validate, and run through the existing pipeline.

Phase F sprint PRs target `integrate/phase-F`. Sprint authority: `docs/plans/phase-F/` (sprints **f.1–f.4**, sequential — not parallel sub-sprints).

**Sprints:** four active (**f.1–f.4**). See [docs/plans/phase-F/README.md](phase-F/README.md).

| Sprint | Title | Doc |
|--------|-------|-----|
| f.1 | Extension runtime — registry, match, preexec, expand | [f1-extension-runtime.md](phase-F/f1-extension-runtime.md) |
| f.2 | Positional extensions — HTML and wizard.json | [f2-positional-extensions.md](phase-F/f2-positional-extensions.md) |
| f.3 | Compose render extension (sc-compose preexec) | [f3-compose-extension.md](phase-F/f3-compose-extension.md) |
| f.4 | CSV table viewer — JS DOM, sort/filter, md alias | [f4-csv-table-viewer.md](phase-F/f4-csv-table-viewer.md) |

---

## Phase G — Extension Agent Usability

**Phase goal (Wave 3):** Wizard authoring platform — progressive-disclosure skill, JS page-author agents, dataflow lint (WIZARD-LINT-005–008), type refs, and CI gate.

**Prerequisite:** Phase F complete on `develop`. Wave 2 after Wave 1 merged.

**Recommended before Phase E** so interactive/MCP agents inherit discoverable argv help and welcome examples.

**Phase acceptance (Wave 1):** `wyvern --help` lists every shipped skill (exit 0); `wyvern compose render --help` prints a skill card; `wyvern extensions list --json` is valid JSON; near-miss paths name the next command.

**Phase acceptance (Wave 3):** `wyvern wizard lint` passes on shipped welcome + agent-dag examples; dataflow lint integration tests green.

Phase G sprint PRs target `integrate/phase-G`. Sprint authority: `docs/plans/phase-G/` (sprints **g.1–g.14**).

**Input:** [phase-F-usability-review.md](phase-F/phase-F-usability-review.md)

**Walkthrough (Wave 2 review):** [examples-walkthrough.md](phase-G/examples-walkthrough.md)

**Wave 3 map:** [phase-G/wave-3-wizard-authoring/README.md](phase-G/wave-3-wizard-authoring/README.md)

| Sprint | Title | Doc |
|--------|-------|-----|
| g.1 | Help surface — global and extension-local | [g1-help-surface.md](phase-G/g1-help-surface.md) |
| g.2 | Error-teaches — near-miss diagnostics and preexec recovery | [g2-error-teaches.md](phase-G/g2-error-teaches.md) |
| g.3 | Skill catalog — list, JSON, show | [g3-skill-catalog.md](phase-G/g3-skill-catalog.md) |
| g.4 | Welcome guide wizard + workflow foundation | [g4-welcome-guide-wizard.md](phase-G/g4-welcome-guide-wizard.md) |
| g.5 | AskUserQuestion hook example | [g5-askuserquestion-claude-code.md](phase-G/g5-askuserquestion-claude-code.md) |
| g.6 | Template wizard | [g6-template-wizard.md](phase-G/g6-template-wizard.md) |
| g.7 | DAG agent demo + export | [g7-dag-agent-execution.md](phase-G/g7-dag-agent-execution.md) |
| g.8 | Wizard authoring foundation | [g8-wizard-authoring-foundation.md](phase-G/g8-wizard-authoring-foundation.md) |
| g.9 | Dataflow lint (WIZARD-LINT-005–008) | [g9-wizard-lint-dataflow.md](phase-G/g9-wizard-lint-dataflow.md) |
| g.10 | `creating-wyvern-wizard` skill router | [g10-creating-wyvern-wizard-skill.md](phase-G/g10-creating-wyvern-wizard-skill.md) |
| g.11 | `wyvern-wizard-js` page agent | [g11-wyvern-wizard-js-agent.md](phase-G/g11-wyvern-wizard-js-agent.md) |
| g.12 | `wyvern-dag-wizard-js` page agent | [g12-wyvern-dag-wizard-js-agent.md](phase-G/g12-wyvern-dag-wizard-js-agent.md) |
| g.13 | Wizard type refs + sc-compose snippets | [g13-wizard-type-refs-and-templates.md](phase-G/g13-wizard-type-refs-and-templates.md) |
| g.14 | Authoring CI + known lint HTML fixes | [g14-wizard-authoring-ci-and-fixes.md](phase-G/g14-wizard-authoring-ci-and-fixes.md) |

---

## Phase E — Interactive & MCP

**Phase goal:** Wyvern runs as a persistent process, driveable by agents over stdin or as an MCP server.

**Transport:** [http-interactive-mcp-contract.md](phase-C/http-interactive-mcp-contract.md) — persistent `HostSession`.

**Prerequisite:** Phase C **c.16** complete; **Phase F** complete for extension-aware MCP tools and argv expansion in interactive mode. **Phase G** recommended first for agent-facing help/catalog surfaces.

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

## Phase H — XHTML reporting & review

**Phase goal:** Non-wizard surfaces for sc-compose XHTML panels — single panel, panel
arrays, and optional **`--review`** (comments + Approve/Cancel) with structured finish
JSON for agent loops.

**Prerequisite:** Phase G complete on `integrate/phase-G` (extension runtime, help,
skill catalog, wizard lint — merge via [#117](https://github.com/randlee/wyvern/pull/117)).
Phase H does **not** require Phase E.

**Transport:** `type: "report"` on `wyvern-host` — static `/report/*`, optional
`/api/report/finish`. **Not** wizard (`wizard-nav.js`, stack, `/api/wizard/*`).

Phase H sprint PRs target `integrate/phase-H`. Sprint authority: `docs/plans/phase-H/`
(sprints **h.1–h.5**, sequential).

**Sprints:** five active (**h.1–h.5**). See [docs/plans/phase-H/README.md](phase-H/README.md).

| Sprint | Deliverable | Doc |
|--------|-------------|-----|
| h.1 | `report` host + `xhtml-suffix` + basic single-panel frame | [h1-xhtml-single-panel.md](phase-H/h1-xhtml-single-panel.md) |
| h.2 | `report-xhtml` extension + panel-array basic frame | [h2-xhtml-panel-array.md](phase-H/h2-xhtml-panel-array.md) |
| h.3 | `--review` frame + finish JSON contract | [h3-xhtml-review-mode.md](phase-H/h3-xhtml-review-mode.md) |
| h.4 | `wyvern-reporting` skill + reference docs | [h4-wyvern-reporting-skill.md](phase-H/h4-wyvern-reporting-skill.md) |
| h.5 | Synthetic example package + CI smoke | [h5-synthetic-xhtml-example.md](phase-H/h5-synthetic-xhtml-example.md) |

**Related:** [GitHub #115](https://github.com/randlee/wyvern/issues/115) — closed by h.1.

Contract: [xhtml-reporting-contract.md](phase-H/xhtml-reporting-contract.md).

---

## Phase I — Wizard native path picker

**Phase goal:** Wizard sessions may use native file/folder pickers in-page; bundled
`path-picker` example ships with install.

**Prerequisite:** Phase H complete on `develop`.

**Related:** [GitHub #99](https://github.com/randlee/wyvern/issues/99).

Phase I sprint PRs target `integrate/phase-I`. Sprint authority: `docs/plans/phase-I/`
(sprint **i.1** only — single-sprint phase).

| Sprint | Deliverable | Doc |
|--------|-------------|-----|
| i.1 | Wizard picker host + `share/wyvern/examples/path-picker/` | [i1-wizard-path-picker.md](phase-I/i1-wizard-path-picker.md) |

See [docs/plans/phase-I/README.md](phase-I/README.md).

---

## Phase J — sc-publish migration

**Phase goal:** Replace bespoke publish workflows/agents with the shared
**[`sc-publish`](https://github.com/randlee/sc-publish)** kit for consistency
across Rust repos — manifest-driven releases, per-channel retries, and fixed
winget/Homebrew semantics.

**Prerequisite:** Current release line on `main` (v0.5.0+). Tooling-only phase.

Phase J sprint PRs target `integrate/phase-J`. Sprint authority:
`docs/plans/phase-J/` (sprints **j.1–j.4**, sequential with j.3 rehearsal gate).

| Sprint | Deliverable | Doc |
|--------|-------------|-----|
| j.1 | Vendor kit + `install.json` + sync script | [j1-vendor-sc-publish-kit.md](phase-J/j1-vendor-sc-publish-kit.md) |
| j.2 | Upstream blockers, secrets, winget/docs | [j2-upstream-blockers-and-docs.md](phase-J/j2-upstream-blockers-and-docs.md) |
| j.3 | Full release rehearsal (kit state machine) | [j3-release-rehearsal.md](phase-J/j3-release-rehearsal.md) |
| j.4 | Production cutover; retire tag-push release | [j4-production-cutover.md](phase-J/j4-production-cutover.md) |

See [docs/plans/phase-J/README.md](phase-J/README.md).

---

## Phase Summary

| Phase | Sprints | Ships |
|-------|---------|-------|
| Phase A — Foundation | 7 | Working binary, `chrome` command |
| Phase B — Core Dialogs | 8 | **MVP — all dialog types usable** |
| Phase D — Wizard | 8 | Multi-page wizard with branching |
| Phase F — CLI Extensions | 4 | Suffix/subcommand argv expansion (CSV table, HTML, compose) |
| Phase G — Extension Agent Usability | 14 | Help/catalog (g.1–g.3) + welcome guide & examples (g.4–g.7) + authoring platform (g.8–g.14) |
| Phase E — Interactive & MCP | 4 | Agent-driveable status viewer + MCP |
| Phase H — XHTML reporting & review | 5 | Single/array XHTML panels + `--review` finish JSON |
| Phase I — Wizard native path picker | 1 | Wizard in-page native pickers + `path-picker` example (closes #99) |
| Phase J — sc-publish migration | 4 | Kit vendored publish; winget/Homebrew/crates retry legs; tag-push retired |

## Dependency Map

```
Phase A
  └─ Phase B
       └─ Phase C (c.9–c.16 HTTP delivery + wyvern-viewer + v0.1.0)
            └─ Phase D (wizard — HTTP on same host)
                 └─ Phase F (CLI extension registry)
                      └─ Phase G (extension agent usability + welcome examples)
                           ├─ Phase E (persistent host + MCP)
                           └─ Phase H (XHTML report surfaces — after G)
                                └─ Phase I (wizard native path picker — after H, closes #99)
```
