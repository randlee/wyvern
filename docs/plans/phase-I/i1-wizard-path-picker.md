---
id: i.1
title: Wizard native path picker + bundled example
status: planned
branch: feature/phase-I-i1-wizard-path-picker
worktree: ../wyvern-worktrees/feature/phase-I-i1-wizard-path-picker
target: integrate/phase-I
---

# Sprint i.1 — Wizard path picker (#99)

## Goal

Close [#99](https://github.com/randlee/wyvern/issues/99): allow **`Command::Wizard`**
sessions to call `POST /api/picker/file` and `POST /api/picker/folder`, and ship a
**durable bundled example** under `share/wyvern/examples/path-picker/` for customers
and CI.

## Hard dependencies

- Phase D wizard host + picker routes (c.11)
- Phase G wizard JS conventions (`WyvernApi`, `collectCurrentPageData`, lint profiles)
- `develop` includes Phase H (no code dependency; baseline only)

## Deliverables

| Path | Purpose |
|------|---------|
| `docs/plans/phase-I/i1-wizard-path-picker.md` | This sprint doc |
| `docs/architecture.md` | **ADR-0026** (wizard picker routes) |
| `docs/wyvern-host/requirements.md` | **REQ-HOST-0150–0151** (wizard picker + 400 guard) |
| `docs/wyvern/requirements.md` | **REQ-0145** (bundled path-picker example) |
| `docs/requirements.md` | Phase I index (ADR-0026, REQ-0145) |
| `docs/plans/phase-C/http-post-schema.md` | Picker routes available for wizard sessions |
| `.claude/skills/creating-wyvern-wizard/references/wizard-types/path-picker.md` | Type recipe (new) |
| `.claude/skills/creating-wyvern-wizard/SKILL.md` | Link path-picker type in wizard type picker |
| `crates/wyvern-host/src/routes/picker.rs` | Accept `Command::Wizard`; request-body defaults; preserve picker-slot RAII + structured `ApiError` envelope |
| `share/wyvern/examples/path-picker/wizard.json` | 2-page vanilla-chrome wizard entry |
| `share/wyvern/examples/path-picker/pages/sources.html` | Browse file (multi) + folder; in-page list |
| `share/wyvern/examples/path-picker/pages/review.html` | Summary before Finish |
| `share/wyvern/examples/path-picker/app.js` | Picker calls, stack/`collectCurrentPageData` |
| `share/wyvern/examples/path-picker/README.md` | Run instructions (GUI + headless mock) |
| `crates/wyvern/share/wyvern/examples/path-picker/` | Packaged parity (share-sync) |
| `crates/wyvern-host/tests/wizard_path_picker.rs` | Wizard session + mock picker; input regression; 400 `cause`/`recovery`/`docs` on rejections |
| `crates/wyvern/tests/examples_path_picker.rs` | CLI smoke `--viewer none` + finish JSON shape |
| `.github/workflows/ci.yml` | `wyvern wizard lint share/wyvern/examples/path-picker` |

### REQ traceability (i.1 lands)

| REQ / ADR | Summary |
|-----------|---------|
| ADR-0026 | Wizard sessions may call picker routes; request-body defaults |
| REQ-HOST-0150 | Wizard session picker POST succeeds with body params |
| REQ-HOST-0151 | Non-wizard/non-input picker calls remain HTTP 400 |
| REQ-0145 | Bundled `path-picker` example + CI smoke |

Amends REQ-0113 scope (wizard allowed); input merge unchanged.

### Host behavior (normative)

**File route** — when `session.command()` is `Command::Wizard { .. }`:

- `filter` = `body.filter.unwrap_or_default()`
- `multiple` = `body.multiple.unwrap_or(false)`
- `start_path` = `body.start_path`

**Folder route** — when wizard:

- `start_path` = `body.start_path`

**Unchanged:** `Command::Input` arms merge body with dialog fields as today.
**Still 400:** `message`, `report`, `markdown`, `question`, `chrome`, wrong input mode.

**Error envelope (normative):** Wizard and input rejection paths must continue using
`picker_bad_request`, `picker_unavailable`, and `picker_timeout` helpers so HTTP 400/503
responses include structured `message`, `cause`, `recovery`, and `docs` (not status code
alone). Non-eligible commands must mention both input (matching mode) and wizard sessions
in recovery text (REQ-HOST-0151).

**Picker slot (normative):** Wizard arms call `acquire_picker_slot()`, hold
`OwnedSemaphorePermit` in the **async handler** through timeout/join (never inside
`spawn_blocking`), and drop before returning — same lifecycle as existing input arms
(ADR-0026 §3; RSH-002).

### Example finish JSON (normative)

```json
{
  "button": "finish",
  "data": {
    "file_paths": ["/abs/path/to/file.csproj"],
    "folder_paths": ["/abs/path/to/root"]
  },
  "stack": [ ... ]
}
```

Page JS collects path strings only — no filesystem reads/writes in the browser.

## Acceptance criteria

1. `cargo fmt --all --check` and `cargo clippy --all-targets --all-features -- -D warnings` pass.
2. `cargo test --workspace` passes (0 failures).
3. Wizard session: `POST /api/picker/file` with `WYVERN_MOCK_PICKER_PATH` returns `{ok:true, paths:[...]}`.
4. Wizard session: `POST /api/picker/folder` mock returns paths JSON.
5. Input file/folder picker tests in `http_input.rs` unchanged (regression).
6. `message` / `report` sessions still get HTTP 400 from picker routes with structured
   `cause`, `recovery`, and `docs` fields (not bare status).
7. `wyvern wizard lint share/wyvern/examples/path-picker` exits 0.
8. Headless: `WYVERN_VIEWER=none` + mock picker runs example wizard and stdout finish includes `file_paths` / `folder_paths`.
9. `scripts/check-share-sync.sh` passes (canonical + packaged example trees match).
10. Example README documents GUI and headless commands using `{wyvern_share}` paths.
11. `wizard_path_picker.rs` asserts `ApiError` JSON shape on wrong-mode and non-eligible
    command rejections (mirror `http_input.rs` success regression scope).

## Required validation

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace
cargo test -p wyvern-host wizard_path_picker
cargo test -p wyvern examples_path_picker
./scripts/check-share-sync.sh
wyvern wizard lint share/wyvern/examples/path-picker
WYVERN_MOCK_PICKER_PATH=/tmp/wyvern-path-picker-fixture.txt \
  WYVERN_VIEWER=none \
  wyvern share/wyvern/examples/path-picker/wizard.json \
  --ui-root share/wyvern/examples/path-picker
```

## Non-closure

- Wizard-level opt-in flag (`pickers: true` in command JSON) — deferred
- Welcome page bridge link — optional follow-up
- MCP / interactive mode picker exposure — Phase E

## Authority

- [#99](https://github.com/randlee/wyvern/issues/99)
- [http-post-schema.md](../phase-C/http-post-schema.md) — picker routes
- [platform-contract.md](../../../.claude/skills/creating-wyvern-wizard/references/core/platform-contract.md)
