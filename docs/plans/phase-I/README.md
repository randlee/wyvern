# Phase I — Wizard native path picker (`integrate/phase-I`)

Phase I closes **[GitHub #99](https://github.com/randlee/wyvern/issues/99)**: wizard
sessions may invoke the native file/folder pickers already exposed in
`WyvernApi.postPickerFile` / `postPickerFolder`. Implementation PRs target
**`integrate/phase-I`**. Sprint docs are **sole authority** for deliverables,
acceptance criteria, and required validation.

**Prerequisite:** Phase H complete on `develop` (XHTML reporting — PR #124).

**Baseline branch:** Worktrees branch from `integrate/phase-I` (or `develop` after
phase-I merges).

---

## Problem

Wizard pages load `wyvern-api.js` and can call picker helpers, but the host rejects
`POST /api/picker/file` and `POST /api/picker/folder` unless the active command is
`input` with matching mode. Multi-page wizards cannot browse for paths in-place;
authors must use typed text fields or split into a separate `input` dialog.

**Use case:** Agent-first provisioning — pre-fill seed paths in `config`, user
browses for additional repo clones or project files on one wizard page, finish
JSON carries path strings for a `workflow.post` script.

---

## Core model

No new host routes. Extend existing picker routes to accept **`Command::Wizard`**
in addition to matching **`Command::Input`** modes. For wizard sessions, picker
parameters come from the **request body only** (`filter`, `multiple`, `start_path`);
omitted fields use defaults (`[]`, `false`, `None`). Session-level picker slot
and `WYVERN_MOCK_PICKER_PATH` behavior unchanged.

Bundled reference example: **`share/wyvern/examples/path-picker/`** — ships with
`cargo install` / repo `share/` (same pipeline as `template-picker`, `xhtml-review`).

---

## Sprint map

| Sprint | Ships | Doc |
|--------|-------|-----|
| **i.1** | Wizard picker host + `path-picker` example + CI | [i1-wizard-path-picker.md](i1-wizard-path-picker.md) |

**Merge order → `integrate/phase-I`:** i.1 only (single-sprint phase).

---

## Phase acceptance criteria

1. i.1 sprint acceptance criteria pass on `integrate/phase-I`.
2. `share/wyvern/examples/path-picker/` passes `wyvern wizard lint` and headless smoke in CI.
3. Input-mode picker merge behavior unchanged (regression tests green).
4. Non-wizard, non-input commands still receive HTTP 400 from picker routes.
5. Closes [#99](https://github.com/randlee/wyvern/issues/99).

---

## Phase integration smoke (non-normative)

```bash
# GUI
wyvern share/wyvern/examples/path-picker/wizard.json \
  --ui-root share/wyvern/examples/path-picker

# Headless + mock picker
WYVERN_MOCK_PICKER_PATH=/tmp/wyvern-path-picker-fixture.txt \
  WYVERN_VIEWER=none \
  wyvern share/wyvern/examples/path-picker/wizard.json \
  --ui-root share/wyvern/examples/path-picker
```
