# Platform contract (host / CLI / page JS)

Normative seams for authoring a Wyvern wizard package. Load from the Layer 0
router at G1 and G3. Stack chrome lives in `references/stacks/`. Dataflow
declarations live in [dataflow-contracts.md](dataflow-contracts.md).

**Do not invent a second schema.** The CLI validator is the only schema.

## 1. Package shape

A wizard package is a directory the CLI can load:

| Path | Role |
|------|------|
| `wizard.json` | Command JSON (`type: "wizard"`) — entry page only |
| `pages/*.html` | Entry + hop targets |
| `app.js` | Page logic (`collectCurrentPageData`, hop descriptors) |
| optional `workflow.pre` / `workflow.post` | Disk I/O scripts (CLI-owned) |
| optional `dist/` | Prebuilt SPA (workspace-canvas only) |

Further pages are **not** a top-level `pages` map on `wizard.json`. They appear
via `app.js` hops (`wizardNextDescriptor`, `wyvernWizardNext`).

Repo-relative goldens: `share/wyvern/examples/template-picker/`,
`share/wyvern/examples/askuserquestion-hook/`,
`share/wyvern/examples/agent-dag/`, `share/wyvern/welcome/`.

## 2. Who owns what

| Actor | Owns | Must not |
|-------|------|----------|
| CLI (`wyvern`) | Load + schema validate; `workflow.pre` / `workflow.post`; resolve `next_wizard` after finish | Run page JS; write finish `data` |
| Host (`wyvern-host`) | Serve `/shared/*`; bind `window.wyvern`; copy `next_wizard` onto the finish result | Spawn workflow scripts; resolve `next_wizard`; type-check `data` / `config` (ADR-0006) |
| Page JS | Collect opaque finish `data`; declare next page / next wizard | Read or write the filesystem |

Host ignores `workflow` (ADR-0023). Host copies `next_wizard` and does not
resolve it (ADR-0024). CLI honors `next_wizard` only when `button` is `finish`.

## 3. `wizard.json` (schema — G1)

Required:

```json
{
  "type": "wizard",
  "page": {
    "id": "one",
    "title": "Step 1",
    "html": "pages/one.html"
  }
}
```

| Field | Rule |
|-------|------|
| `type` | `"wizard"` |
| `page.id` / `page.title` / `page.html` | Required. `html` is relative to the package (or `ui_root`) |
| `page.layout` | Omit (dialog) or `"workspace"` (canvas stack) |
| `width` / `height` | Optional dialog size |
| `config` | Opaque object (ADR-0006). Catalogs and `config.dataflow` live here |
| `workflow.pre` / `workflow.post` | Optional allowlisted path strings (`{wyvern_share}`, cwd, wizard dir) |

Validate with the same path the CLI uses to load a command (exit **4** =
schema / `validation`):

```bash
wyvern path/to/wizard.json --viewer none
```

Inline JSON (`wyvern '{"type":"wizard",...}'`) uses the same validator.
Parse / usage errors are exit **2**. There is no separate
`wyvern wizard schema-validate` subcommand.

Entry HTML at `page.html` must exist before G3. Hop-target HTML must exist
before G3. Missing files are package errors, not schema fields.

## 4. Shared chrome (host-served)

Host serves `ui/shared/*` at `/shared/…` (Phase D). Vanilla-chrome pages opt in:

```html
<link rel="stylesheet" href="/shared/embedded-chrome.css">
<link rel="stylesheet" href="/shared/wizard-chrome.css">
<script src="/shared/wyvern-api.js"></script>
<script src="../app.js"></script>
<script src="/shared/wizard-nav.js" data-wizard-chrome></script>
```

`wizard-nav.js` without `data-wizard-chrome` does not wire Back / Next / Finish.
Workspace-canvas requires `wyvern-api.js`; full wizard-nav chrome is optional
(see [workspace-canvas.md](../stacks/workspace-canvas.md)).

`window.wyvern` exposes `config`, `page`, `page_data`, and `stack`. Read those.
Do not fetch `file://` or scan directories for catalogs.

## 5. Page-author hooks

| Hook | Required | Role |
|------|----------|------|
| `collectCurrentPageData()` | yes | Opaque object for next / back / finish. Missing / null → treat as `{}` |
| `wizardNextDescriptor` | non-terminal Next | `{ id, title, html }` or `() => ({ id, title, html })` |
| `wizardNextWizard` | welcome-bridge / chain only | `{ path, input?, ui_root? }` or function; copied onto finish (REQ-0126) |

Terminal page root sets `data-wizard-terminal="true"`. Chrome relabels Next →
Finish and calls `wyvernWizardFinish({ button, data, stack, next_wizard? })`.

Primary testids: `wizard-back`, `wizard-next`, `wizard-error`. Full table:
[vanilla-chrome.md](../stacks/vanilla-chrome.md).

## 6. Finish, chain, workflow

Finish JSON (REQ-0024 / d.7):

```json
{
  "button": "finish",
  "data": {},
  "stack": [{ "page": { "id": "…", "title": "…", "html": "…" }, "data": {} }]
}
```

`cancel` / `dismissed` skip `workflow.post` and `next_wizard`.

| Phase | When | I/O |
|-------|------|-----|
| Pre (REQ-0124) | After validate, before host | stdout `{ "config_patch": { … } }`; deep-merge into `config` |
| Page | In the webview | Form fields only; opaque `data` |
| Post (REQ-0125) | After `button: "finish"`, before `next_wizard` | finish JSON on stdin; writes files |

`--workflow-dry-run` appends `--dry-run` to the script argv; the script must
write nothing. Timeout 30s. Allowlist: `{wyvern_share}`, cwd, wizard dir.
`.py` → `python3 <path>`. Failure → `WORKFLOW_ERROR` exit 9.

`next_wizard` (REQ-0126): `path` required; `input` defaults `{}`; `ui_root`
optional. Max 16 sessions. Final CLI stdout omits `next_wizard`.

## 7. Disk I/O (normative)

Page JS **must not** read or write the filesystem.

Forbidden in page JS: `fetch("file://…")`, Node/Deno `fs` / `child_process`,
File System Access API writes, using `FileReader` to persist bytes, directory
scans to invent catalog rows.

Allowed: path **strings** in finish `data` (`<input type="text">`,
`<input type="file">` → `file.path` or `file.name`,
`WyvernApi.postPickerFile` / `postPickerFolder`).

## 8. Stack vs type

| Axis | Registry | Meaning |
|------|----------|---------|
| Stack | `references/stacks/registry.yaml` | UI chrome + lint profile + authoring agent |
| Type | `references/wizard-types/` (g.12–g.13) | Domain shape (template, hook, welcome-bridge, dag) |

Default stack is `vanilla-chrome`. `workspace-canvas` is supported. Authors
pick one of each; the Layer 0 router names the pair.

## Authority

- ADR-0006, ADR-0023, ADR-0024
- REQ-0024, REQ-0124, REQ-0125, REQ-0126
- [wizard-workflow-architecture.md](../../../../../docs/plans/phase-G/wizard-workflow-architecture.md)
- [g4-welcome-guide-wizard.md](../../../../../docs/plans/phase-G/g4-welcome-guide-wizard.md)
- [author-workflow.md](author-workflow.md)
- [validation-and-lint.md](validation-and-lint.md)
