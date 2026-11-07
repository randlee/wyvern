# Claude Instructions for wyvern

## ⚠️ CRITICAL: Branch Management

**NEVER switch the main repository branch from `develop`.**

- Main repo MUST remain on `develop` at all times
- **ALWAYS use `sc-git-worktree` skill** for all development work
- **ALWAYS create worktrees FROM `develop`**
- All sprint work happens in worktrees at `../wyvern-worktrees/<branch-name>`
- All PRs target `integrate/phase-N` (not `develop` directly)
- Phase A (Foundation) PRs target `integrate/phase-A` — see `docs/plans/phase-A/README.md`

```bash
# ✅ CORRECT
sc-git-worktree --create feature/phase-A-a1-scaffold develop

# ❌ WRONG
git checkout -b feature/phase-A-a1-scaffold
```

---

## Project Overview

**Wyvern** (*What You View, Engine Renders Natively*) is a Rust CLI tool that opens OS-native webview windows for user interaction and returns structured JSON results.

- JSON in / JSON out — MCP-compatible from the ground up
- All UI rendered as HTML/CSS/JS — no OS-native widgets
- Five-crate workspace: `wyvern-schema`, `wyvern-window`, `wyvern-wizard`, `wyvern` (CLI), `wyvern-mcp`
- Built on `wry` (Tauri) — wraps OS webviews (WebKit/WebView2/WebKitGTK)
- Dialog types: `chrome` (Phase A foundation), then `message`, `input`, `markdown`, `question` (AskUserQuestion-compatible), `wizard` (Phase B+)
- Interactive mode (`--interactive`): persistent stdin loop for blocking dialog commands plus `show`/`hide`/`exit`
- MCP mode: persistent background process, tool-call driven

---

## Key Documentation

| Document | Purpose |
|----------|---------|
| [`docs/prd/wyvern-prd.md`](docs/prd/wyvern-prd.md) | Full product requirements |
| [`docs/requirements.md`](docs/requirements.md) | Numbered requirements (REQ/NFR) — links to crate docs |
| [`docs/architecture.md`](docs/architecture.md) | ADRs — links to crate docs |
| [`docs/plans/project-plan.md`](docs/plans/project-plan.md) | 5-phase, 30-sprint plan |
| [`docs/linting.md`](docs/linting.md) | sc-lint 0.5.0, Just bootstrap, CI analyzers |

**Per-crate docs** (referenced by principals above):

| Crate | Requirements | Architecture |
|-------|-------------|--------------|
| `wyvern` | [`docs/wyvern/requirements.md`](docs/wyvern/requirements.md) | [`docs/wyvern/architecture.md`](docs/wyvern/architecture.md) |
| `wyvern-schema` | [`docs/wyvern-schema/requirements.md`](docs/wyvern-schema/requirements.md) | [`docs/wyvern-schema/architecture.md`](docs/wyvern-schema/architecture.md) |
| `wyvern-window` | [`docs/wyvern-window/requirements.md`](docs/wyvern-window/requirements.md) | [`docs/wyvern-window/architecture.md`](docs/wyvern-window/architecture.md) |
| `wyvern-wizard` | [`docs/wyvern-wizard/requirements.md`](docs/wyvern-wizard/requirements.md) | [`docs/wyvern-wizard/architecture.md`](docs/wyvern-wizard/architecture.md) |
| `wyvern-mcp` | [`docs/wyvern-mcp/requirements.md`](docs/wyvern-mcp/requirements.md) | [`docs/wyvern-mcp/architecture.md`](docs/wyvern-mcp/architecture.md) |

**Boundary rules**: [`boundaries/`](boundaries/) — one TOML per crate, enforced in CI from Phase B.

---

## Crate Dependency Rules (ADR-0011)

```
wyvern-schema   →  (no internal deps)
wyvern-wizard   →  wyvern-schema
wyvern-window   →  wyvern-schema, wyvern-wizard, wry, winit
wyvern          →  wyvern-window, wyvern-schema
wyvern-mcp      →  wyvern-window, wyvern-schema
```

**Hard rules:**
- `wyvern-schema` and `wyvern-wizard` are pure logic — no I/O, no window, no async
- `wry` and `winit` only in `wyvern-window`
- `wyvern-mcp` never touches the window directly — only via `wyvern-window` API

---

## Branch & Sprint Workflow

### Branch structure
```
main
  └── develop
        ├── integrate/phase-A          ← Phase A Foundation
        ├── integrate/phase-B          ← Phase B Core Dialogs
        ├── integrate/phase-B … integrate/phase-E
        └── feature/pN-sXa-...         ← sprint PR targets integrate/phase-* for that phase
        After all sprints in a phase → integrate/phase-* → develop
```

### Sprint execution
1. Create worktree from `develop` via `sc-git-worktree`
2. Implement sprint to its acceptance criteria
3. Run `cargo test --workspace -- --test-threads=1` and sc-lint gates from [`docs/linting.md`](docs/linting.md):
   `sc-lint check native --config .sc-lint.toml`, `sc-lint clippy native --config .sc-lint.toml`,
   and (after materializing `.just/` via setup-sc-lint) `scripts/sc_lint_python_gate.sh` when touching policy
4. PR → `integrate/phase-N`
5. Do NOT clean up worktree until user reviews

### Sprint naming convention
`feature/p{phase}-s{sprint}-{short-description}`
Example: `feature/phase-A-a1-scaffold`, `feature/phase-B-b1-message`

---

## Agent Model Selection

- **Haiku** — exploration, file search, simple validation, smoke tests
- **Sonnet** — implementation, documentation, standard sprints
- **Opus** — architecture decisions, cross-crate design, escalation

---

## Environment

- **External dependencies**: `sc-observability` and `sc-lint` from [crates.io](https://crates.io) (no local path deps)
- **Worktrees**: `../wyvern-worktrees/<branch>`
- **Platforms**: macOS (primary), Windows, Linux

## Honest Work, No Ceremony (binding on agents and humans)

Goal: working, deployable capability. Process serves it and never becomes the product.

- **Process artifacts** (certificates, ledgers, dashboards, matrices, meta-reports, speculative checks) need four things to exist: a named consumer, the feature it gates, the observed defect class that justifies it, and a deletion condition. Missing any of these → don't create it. Test: if running code branches on it, it's product. If only humans or reports read it, it's process and needs the four things. Writing code just to flip that answer is itself the pathology. Exception: a minimal integrity/recovery control (crash-recovery state, provenance snapshot) that prevents a named evidence-loss or corruption mode.
- **Real code + real tests in the same unit of work.** Forbidden: faked tests, mocks/fixtures presented as live proof, weakened assertions, regenerating goldens to force green, hard-coded success paths, placeholder macros in commits, editing the spec instead of implementing it, narrowing scope while claiming full success.
- **No self-certification.** An independent verifier closes the work, citing evidence at an exact revision. In solo sessions, re-verify by re-running, and state what wasn't independently verified.
- **Typed refusal beats fabrication but is worth less than real capability.** Work that ends only in a refusal stays open and says so.
- **Null results are successes** ("checked X, found no material increment"). Unsupported claims are worse than silence.
- **Metrics** predeclare their denominator and countermetric. Agreement between agents can raise confidence but is never independent evidence. Never silence stderr in commands that produce evidence.
- **Name pathologies when they occur:** gate self-weakening, proof-class inflation, golden regeneration, tolerance widening, suppression-pragma laundering, refusal farming, follow-up laundering. The full catalog and countermeasures are in the `just-say-no-to-process-porn-and-ceremony` skill. Ask the operator for it if you can't find it.
