# Phase C — Polish & Release v0.1.0 (`integrate/phase-C`)

Phase C implementation PRs target **`integrate/phase-C`**. This directory is the **sole authority** for sprint-level deliverables, acceptance criteria, and validation. `docs/plans/project-plan.md` carries phase-level goals and acceptance criteria only.

Sprints are **sequentially numbered** `c.1` → `c.5`. Dependency graph:

```
Phase B ──┬──► c.1 ──► c.2 ──┐
          │                   ├──► c.4 ──► c.5
          └──► c.3 ───────────┘
```

- **c.1 → c.2:** icon asset bundle, then named-icon validation and resolution
- **c.3:** independent after Phase B (Win/Linux chrome does not block on c.1–c.2)
- **c.4:** depends on c.1, c.2, and c.3
- **c.5:** depends on c.4

## What Phase C closes

- Full shipped icon bundle (REQ-0030, REQ-0031) replacing Phase B **placeholder** SVGs at `assets/icons/placeholder/`
- Named icon resolution with variant index (`"warning:2"`), path, and base64 — unknown named icons → **validation error** (replacing b.2 run-time info fallback)
- Windows/Linux platform chrome: `decorations: false` + HTML close/minimize via IPC (ADR-0010a, REQ-0085, REQ-0086, REQ-0087)
- macOS NFR verification (NFR-0001–NFR-0003) and cross-platform rendering regression pass
- v0.1.0 release: GitHub Actions binary matrix, README quickstart, `CHANGELOG.md`

## What Phase C inherits from Phase B (unchanged)

- All four dialog types executable end-to-end (`message`, `input`, `markdown`, `question`)
- Window auto-size bounds: **min 320×200**, **max 800×600** (REQ-0041) — dialog types only; `chrome` keeps Phase A fixed **480×360** open with **800×600** max
- Modal window attributes: minimize/maximize disabled (REQ-0083)
- Dialog IPC contract ([../phase-B/ipc-dialog-contract.md](../phase-B/ipc-dialog-contract.md)) for button/input/question events
- macOS transparent title bar + 72px safe zone (ADR-0010, REQ-0080–REQ-0082)

## What Phase C does not close

- `wizard` type — **Phase D**
- `--interactive` / lifecycle actions — **Phase E**
- MCP server — **Phase E**
- AI-generated icons — post-MVP (PRD)
- Homebrew formula tap (optional stretch; README documents install from GitHub release as minimum bar)

## Phase B → Phase C handoff (authoritative)

| Area | Phase B reality (merged) | Phase C completes |
|------|--------------------------|-------------------|
| Level icons | Four placeholder SVGs in `assets/icons/placeholder/` mapped by `MessageLevel` | Production icons per role; `level` renders variant 1 from bundle |
| Named `icon` field | Resolves only to placeholder set; `:variant` syntax accepted but **ignored**; unknown names fall back to info placeholder at run time | Full role catalog incl. `success`, `loading`; variant index honored; unknown → validation error |
| Win/Linux frame | `with_decorations(true)` in `window.rs` — native OS title bar | `decorations: false` + HTML window controls ([chrome-ipc-contract.md](chrome-ipc-contract.md)) |
| NFR targets | Not measured in Phase B | macOS benchmarks in c.4; binary size monitored after icon bundle lands |
| Release CI | `.github/workflows/ci.yml` only (build/test/clippy/sc-lint) | c.5 adds release workflow on tag push |

## Platform policy after Phase C

| Platform | Window chrome | Window controls |
|----------|---------------|-----------------|
| macOS | Transparent title bar (ADR-0010), HTML shell | Native traffic lights; no HTML close/minimize |
| Windows | `decorations: false`, full-size HTML chrome (ADR-0010a) | HTML close; HTML minimize on **non-modal** types only (`chrome`; wizard in Phase D) |
| Linux | Same as Windows | Same as Windows |

Modal types (`message`, `input`, `markdown`, `question`) keep REQ-0083: minimize disabled at window-attribute layer — HTML minimize hidden or inert on Win/Linux.

## Phase acceptance criteria (smoke)

1. `wyvern '{"type":"message","title":"T","message":"Hi","level":"warning","buttons":"ok"}'` → production warning icon (not placeholder marker); OK → `{"button":"ok"}`
2. `wyvern '{"type":"message","title":"T","message":"Hi","icon":"success:2","buttons":"ok"}'` → second success variant
3. `wyvern '{"type":"message","title":"T","message":"Hi","icon":"nonexistent","buttons":"ok"}'` → validation stderr listing valid icon names, exit ≠ 0, no window
4. All Phase B README smoke checks pass on **ubuntu, macos, and windows** CI legs (no manual Win/Linux E2E required)
5. Tag `v0.1.0` produces attached macOS/Windows/Linux release binaries

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
| [chrome-ipc-contract.md](chrome-ipc-contract.md) | Win/Linux HTML window control IPC (extends Phase B dialog IPC) |
| [../phase-B/ipc-dialog-contract.md](../phase-B/ipc-dialog-contract.md) | Dialog button/input/question IPC (unchanged) |
| [../../wyvern-window/architecture.md](../../wyvern-window/architecture.md) | ADR-0010a deferral closure; ADR-0015 icon asset layout |

## CI validation (authoritative)

Inherits Phase B matrix from [../phase-B/README.md](../phase-B/README.md):

| Leg | Commands |
|-----|----------|
| `ubuntu-latest` | xvfb + software GL flags → `cargo test --workspace -- --test-threads=1` |
| `macos-latest` | `cargo test --workspace -- --test-threads=1` |
| `windows-latest` | `cargo test --workspace -- --test-threads=1` |

All legs: `cargo build --workspace`, `cargo clippy --workspace -- -D warnings`, `sc-lint check native --config .sc-lint.toml`.

After c.5: release workflow (see [c5-release.md](c5-release.md)) validates on tag push — not on every PR.

### NFR measurement (c.4 — macOS dev/CI optional job)

| NFR | Target | Measurement |
|-----|--------|-------------|
| NFR-0001 | Window open < 500ms | macOS manual measurement (product target); optional non-blocking CI job may use 2000ms smoke bound |
| NFR-0002 | Resident memory < 80MB | macOS Activity Monitor or `ps` after dialog open |
| NFR-0003 | Binary < 10MB | `ls -lh target/release/wyvern` on macOS release build |

NFR-0004–NFR-0007 remain satisfied by existing architecture; c.4 confirms no regression.

## sc-lint-boundary

Review `boundaries/*.toml` at sprint planning for c.1 (new asset module paths) and c.3 (IPC handler surface). No new crate deps expected.
