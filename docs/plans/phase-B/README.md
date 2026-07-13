# Phase B — Core Dialogs (`integrate/phase-B`)

Phase B implementation PRs target **`integrate/phase-B`**. This directory is the **sole authority** for sprint-level deliverables, acceptance criteria, and validation. `docs/plans/project-plan.md` carries phase-level goals and acceptance criteria only.

Sprints are **sequentially numbered** `b.1` → `b.8` (strict dependency order — not parallel sub-sprints).

## What Phase B closes

- Four blocking dialog types end-to-end from CLI: `message`, `input`, `markdown`, `question`
- Dialog IPC contract ([ipc-dialog-contract.md](ipc-dialog-contract.md)) wired for button/input/dismiss events
- Per-type validation in `wyvern-schema` (incremental unlock per sprint — see below)
- `Command` / `CommandResult` enum extensions per ADR-0013
- Window **auto-size to content** (REQ-0041) replacing Phase A fixed 480×360 chrome
- HTML **button bar** populated (Phase A left `#button-bar` hidden)
- sc-lint-boundary rules reviewed and enforced in CI from Phase B onward

## What Phase B does not close

- Full shipped icon set (REQ-0030) — **Phase C** (`integrate/phase-C`); b.2 uses **placeholder SVGs** only
- Win/Linux `decorations: false` + HTML close/minimize (ADR-0010a, REQ-0085) — **Phase C**
- `wizard` type — **Phase D**
- `--interactive` / lifecycle actions — **Phase E**
- MCP server — **Phase E**

## Incremental executable surface (REQ-0049)

Each sprint unlocks its `type` for validation **and** execution when that sprint merges. Earlier types remain executable; later types stay validation errors until their sprint lands.

| After sprint | Executable `type` values | Notes |
|--------------|--------------------------|-------|
| Phase A | `chrome` | unchanged |
| b.1 | `chrome`, `message` | text + buttons only; no icons/markdown body extras |
| b.2 | `message` (full) | icons/images/markdown message body |
| b.3 | + `input` (`mode: text` only) | file/folder still rejected until b.4 |
| b.4 | `input` (all modes) | native picker via `rfd` in `wyvern-window` |
| b.5 | + `markdown` (file) | `.md` argv shorthand + `file` field |
| b.6 | `markdown` (full) | inline `content` + stylesheet |
| b.7 | + `question` (render) | preview deferred to b.8 |
| b.8 | `question` (full) | AskUserQuestion compliance + preview |

## Platform policy during Phase B

| Platform | Window chrome in Phase B | Deferred to Phase C |
|----------|--------------------------|---------------------|
| macOS | Transparent title bar (ADR-0010), HTML chrome shell | — |
| Windows/Linux | **Native OS decorations** (same as Phase A) | `decorations: false` + HTML close/minimize (REQ-0085) |

Dialog **content** (buttons, inputs, markdown) is always HTML inside the webview. Only the **outer window frame** on Win/Linux stays native until Phase C.

Modal types (`message`, `input`, `markdown`, `question`) disable minimize/maximize per REQ-0083 — enforced in `wyvern-window` window attributes from b.1.

## Phase acceptance criteria (smoke)

1. `wyvern '{"type":"message","title":"T","message":"Hi","buttons":"ok"}'` → dialog; OK → `{"button":"ok"}`; OS close → `{"button":"dismissed"}`
2. `wyvern '{"type":"input","title":"Name","message":"Enter name","default":"Ada"}'` → text field; OK → `{"button":"ok","input":"..."}`
3. `wyvern doc.md` and `wyvern '{"type":"markdown","file":"doc.md"}'` → rendered viewer; OK → `{"button":"ok"}`
4. `wyvern '{"type":"question","questions":[...]}'` → cards; submit → AskUserQuestion response shape (see [question-contract-examples.md](question-contract-examples.md))
5. `wyvern '{"type":"wizard",...}'` → validation stderr, exit ≠ 0, no window (still Phase D)

## Sprint index (sequential: b.1–b.8)

| Sprint | Doc | Branch (pattern) |
|--------|-----|------------------|
| b.1 | [b1-message-structure.md](b1-message-structure.md) | `feature/phase-B-b1-message-structure` |
| b.2 | [b2-message-icons.md](b2-message-icons.md) | `feature/phase-B-b2-message-icons` |
| b.3 | [b3-input-text.md](b3-input-text.md) | `feature/phase-B-b3-input-text` |
| b.4 | [b4-input-picker.md](b4-input-picker.md) | `feature/phase-B-b4-input-picker` |
| b.5 | [b5-markdown-file.md](b5-markdown-file.md) | `feature/phase-B-b5-markdown-file` |
| b.6 | [b6-markdown-inline.md](b6-markdown-inline.md) | `feature/phase-B-b6-markdown-inline` |
| b.7 | [b7-question-render.md](b7-question-render.md) | `feature/phase-B-b7-question-render` |
| b.8 | [b8-question-preview.md](b8-question-preview.md) | `feature/phase-B-b8-question-preview` |

## Cross-cutting contracts

| Doc | Purpose |
|-----|---------|
| [ipc-dialog-contract.md](ipc-dialog-contract.md) | JS ↔ Rust IPC for dialog interactions |
| [question-contract-examples.md](question-contract-examples.md) | Sample AskUserQuestion payloads + expected stdout |
| [../../wyvern-schema/architecture.md](../../wyvern-schema/architecture.md) | `Command` / `CommandResult` extension ADR |
| [../../wyvern-window/architecture.md](../../wyvern-window/architecture.md) | File picker ADR (`rfd`) |

## CI validation (authoritative)

Inherits Phase A matrix from [phase-A/README.md](../phase-A/README.md) with these additions:

| Leg | Phase B additions |
|-----|-------------------|
| All | `cargo test --workspace -- --test-threads=1` (GUI serialization) |
| ubuntu | xvfb + software GL flags (unchanged from Phase A) |
| b.4+ | file-picker tests use `rfd` with mocked paths or skip picker UI on headless Linux where noted in sprint doc |

After each sprint: `sc-lint check native --config .sc-lint.toml` (boundary enforcement active from Phase B).

## sc-lint-boundary

Review and extend `boundaries/*.toml` at **sprint planning** for each b.N sprint. Do not defer boundary rules to a separate sprint — they are a planning gate, not implementation work.
