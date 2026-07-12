# Claude Instructions for wyvern

## ⚠️ CRITICAL: Branch Management

**NEVER switch the main repository branch from `develop`.**

- Main repo MUST remain on `develop` at all times
- **ALWAYS use `sc-git-worktree` skill** for all development work
- **ALWAYS create worktrees FROM `develop`**
- All sprint work happens in worktrees at `../wyvern-worktrees/<branch-name>`
- All PRs target `integrate/phase-N` (not `develop` directly)

```bash
# ✅ CORRECT
sc-git-worktree --create feature/p1-s1a-scaffold develop

# ❌ WRONG
git checkout -b feature/p1-s1a-scaffold
```

---

## Project Overview

**Wyvern** (*What You View, Engine Renders Natively*) is a Rust CLI tool that opens OS-native webview windows for user interaction and returns structured JSON results.

- JSON in / JSON out — MCP-compatible from the ground up
- All UI rendered as HTML/CSS/JS — no OS-native widgets
- Five-crate workspace: `wyvern-schema`, `wyvern-window`, `wyvern-wizard`, `wyvern` (CLI), `wyvern-mcp`
- Built on `wry` (Tauri) — wraps OS webviews (WebKit/WebView2/WebKitGTK)
- Dialog types: `message`, `input`, `markdown`, `question` (AskUserQuestion-compatible), `wizard`
- Interactive mode (`--interactive`): persistent stdin loop for blocking dialog commands plus `show`/`hide`/`exit`
- MCP mode: persistent background process, tool-call driven

---

## Key Documentation

| Document | Purpose |
|----------|---------|
| [`docs/prd/wyvern-prd.md`](docs/prd/wyvern-prd.md) | Full product requirements |
| [`docs/requirements.md`](docs/requirements.md) | Numbered requirements (REQ/NFR) — links to crate docs |
| [`docs/architecture.md`](docs/architecture.md) | ADRs — links to crate docs |
| [`docs/plans/project-plan.md`](docs/plans/project-plan.md) | 5-phase, 31-sprint plan |

**Per-crate docs** (referenced by principals above):

| Crate | Requirements | Architecture |
|-------|-------------|--------------|
| `wyvern` | [`docs/wyvern/requirements.md`](docs/wyvern/requirements.md) | [`docs/wyvern/architecture.md`](docs/wyvern/architecture.md) |
| `wyvern-schema` | [`docs/wyvern-schema/requirements.md`](docs/wyvern-schema/requirements.md) | [`docs/wyvern-schema/architecture.md`](docs/wyvern-schema/architecture.md) |
| `wyvern-window` | [`docs/wyvern-window/requirements.md`](docs/wyvern-window/requirements.md) | [`docs/wyvern-window/architecture.md`](docs/wyvern-window/architecture.md) |
| `wyvern-wizard` | [`docs/wyvern-wizard/requirements.md`](docs/wyvern-wizard/requirements.md) | [`docs/wyvern-wizard/architecture.md`](docs/wyvern-wizard/architecture.md) |
| `wyvern-mcp` | [`docs/wyvern-mcp/requirements.md`](docs/wyvern-mcp/requirements.md) | [`docs/wyvern-mcp/architecture.md`](docs/wyvern-mcp/architecture.md) |

**Boundary rules**: [`boundaries/`](boundaries/) — one TOML per crate, enforced in CI from Phase 2.

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
        └── integrate/phase-N          ← created at phase start
              ├── feature/pN-sXa-...   ← sprint PR targets integrate/phase-N
              └── feature/pN-sXb-...
        After all sprints → integrate/phase-N → develop
```

### Sprint execution
1. Create worktree from `develop` via `sc-git-worktree`
2. Implement sprint to its acceptance criteria
3. Run `cargo test --workspace` + clippy clean
4. PR → `integrate/phase-N`
5. Do NOT clean up worktree until user reviews

### Sprint naming convention
`feature/p{phase}-s{sprint}-{short-description}`
Example: `feature/p1-s1a-scaffold`, `feature/p2-s2a-message-type`

---

## Agent Model Selection

- **Haiku** — exploration, file search, simple validation, smoke tests
- **Sonnet** — implementation, documentation, standard sprints
- **Opus** — architecture decisions, cross-crate design, escalation

---

## Environment

- **External dependencies**: `sc-observability` at `../../sc-observability`, `sc-lint` at `../../sc-lint`
- **Worktrees**: `../wyvern-worktrees/<branch>`
- **Platforms**: macOS (primary), Windows, Linux
