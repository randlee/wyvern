---
name: wyvern-wizard-js
version: 0.1.0
description: Author vanilla-chrome Wyvern wizard pages (wizard.json, app.js, HTML). Use when scaffolding or linting wizard UI that must use wyvern-api.js, wizard-nav.js, collectCurrentPageData, and workflow.pre/post for disk. Do not use for Rust, crates, or rust-developer work.
model: sonnet
color: cyan
---

# Wyvern Wizard JS (vanilla chrome)

## Purpose

Author or lint **vanilla-chrome** wizard pages so Back / Next / Finish, opaque
page data, and optional `next_wizard` match shipped host chrome. Disk I/O is
never in page JS.

Do **not** spawn `rust-developer`. Do **not** edit `crates/**`.

## Inputs

- `action` (required): `scaffold` | `author` | `lint`
- `target_dir` (required for scaffold/author): destination directory
- `title` (optional): wizard title (default `Wizard`)
- `pages` (optional): `[{ id, title, html, terminal? }]` — default two-step
  `one` → `two` (terminal)
- `workflow` (optional): `{ pre?, post? }` script paths for `wizard.json`
- `next_wizard` (optional): finish hop `{ path, input?, ui_root? }`
- `validate` (optional, bool): lint only; write nothing

If omitted, treat as `{ "action": "scaffold", "target_dir": "./wizard" }`.

## Execution Steps

1. Validate inputs. Reject empty `target_dir` for `scaffold` / `author`.
2. Read goldens if the task is non-trivial:
   - `share/wyvern/examples/template-picker/` (`app.js`, `wizard.json`, pages)
   - `share/wyvern/examples/askuserquestion-hook/` (pre fills config; page JS
     never touches hook files)
   - `ui/shared/wizard-nav.js` (chrome contract)
3. For `scaffold`: copy
   `.claude/skills/creating-wyvern-wizard/templates/vanilla-chrome/`
   into `target_dir` (`wizard.json`, `app.js`, `pages/one.html`,
   `pages/two.html`). Adapt titles only if `title` / `pages` were supplied.
4. For `author`: write or update those files using the contracts below. Prefer
   one shared `app.js` (golden style) over inline page scripts.
5. For `lint` / `validate: true`: do not write. Fail on any Forbidden page-JS
   pattern, missing hook, missing chrome script, or missing required testid.
6. **Mandatory G4 before success (`scaffold` / `author`):** after writing files,
   run `wyvern wizard lint <target_dir>` (or `<target_dir>` if it contains
   `wizard.json`). If exit ≠ 0, read every WIZARD-LINT-* line, fix HTML/JS/
   `config.dataflow`, re-run until exit **0**. Do **not** return `success: true`
   or present the package to the user with open lint findings.
7. Return **only** the fenced JSON envelope (plus the required fence). No
   extra prose after the envelope.

## Shared chrome (required)

Every HTML page:

```html
<link rel="stylesheet" href="/shared/embedded-chrome.css" />
<link rel="stylesheet" href="/shared/wizard-chrome.css" />
<script src="/shared/wyvern-api.js"></script>
```

End of body, in this order:

```html
<script src="../app.js"></script>
<script src="/shared/wizard-nav.js" data-wizard-chrome></script>
```

`data-wizard-chrome` is mandatory — without it, `wizard-nav.js` does not wire
buttons.

Page root:

```html
<main id="dialog" class="dialog dialog--frame" data-testid="{page-id}">
```

Terminal page adds `data-wizard-terminal="true"` on that root.

Nav:

```html
<nav class="wizard-chrome" data-wizard-nav aria-label="Wizard navigation">
  <button type="button" class="secondary" data-wizard-back data-testid="wizard-back">Back</button>
  <button type="button" class="primary" data-wizard-next data-testid="wizard-next">Next</button>
</nav>
<p class="wizard-error" data-testid="wizard-error" hidden></p>
```

`wyvern-api.js` exposes `wyvernWizardState`, `wyvernWizardNext`,
`wyvernWizardBack`, `wyvernWizardFinish`, and `WyvernApi` (picker + layout).
`wizard-nav.js` calls:

- Next → `wyvernWizardNext(collectCurrentPageData(), wizardNextDescriptor)`
- Back → `wyvernWizardBack()` (uses `collectCurrentPageData` when defined)
- Finish → `wyvernWizardFinish({ button: "finish", data, stack, next_wizard? })`
  where `stack` is `window.wyvern.stack` plus `{ page, data }`

## Page-author hooks

Define these on `window` in `app.js`:

| Symbol | When | Value |
|--------|------|--------|
| `collectCurrentPageData` | always | `function () { return object }` — never `undefined` |
| `wizardNextDescriptor` | every non-terminal page | `{ id, title, html }` or `() => ({…})` |
| `wizardNextWizard` | only when chaining | `{ path, input?, ui_root? }` or function |

```js
window.collectCurrentPageData = function () {
  var label = document.querySelector("[data-testid='field-label']");
  var pathEl = document.querySelector("[data-testid='field-file-path']");
  return {
    label: label ? String(label.value || "") : "",
    file_path: pathEl ? String(pathEl.value || "") : ""
  };
};

window.wizardNextDescriptor = {
  id: "two",
  title: "Review",
  html: "pages/two.html"
};
```

Restore fields from `window.wyvern.page_data` and prior `stack` entries (REQ-0024).
Host treats `data` as opaque (ADR-0006).

## Disk I/O — page JS never touches disk

| Phase | Who | What |
|-------|-----|------|
| `workflow.pre` | CLI | Read disk; stdout `{ "config_patch": { … } }` (REQ-0124) |
| Page JS | `app.js` | Collect form values into opaque `data` |
| `workflow.post` | CLI | Read finish JSON on stdin; write disk (REQ-0125) |

**Forbidden in page JS** (lint-fail / `VALIDATION.DISK_IO`):

- `fetch("file://…")` or XHR to local files
- `require("fs")`, Deno FS, `child_process`
- File System Access API writes (`showSaveFilePicker`, `createWritable`)
- `FileReader` used to persist or parse a picked file as a disk side effect
- Scanning directories to build catalogs (g.6: `config.templates` only)

**File paths — collect strings only:**

1. `<input type="text" data-testid="field-file-path">` (preferred; g.6
   `field-output-path`)
2. `<input type="file" data-testid="field-file">` — on change, set the text
   field to `file.path || file.name`. Do **not** read bytes.
3. `WyvernApi.postPickerFile` / `postPickerFolder` — store the returned path
   string in `data`

Paths appear in finish `data` only. `workflow.post` performs the write.

Display existing paths from `window.wyvern.config` after pre (g.5
`config.hook_state.*.settings_path`), never by reading those files in the page.

## `data-testid` conventions

| Role | `data-testid` |
|------|----------------|
| Back | `wizard-back` (plus `data-wizard-back`) |
| Next / Finish | `wizard-next` (plus `data-wizard-next`) |
| Error | `wizard-error` |
| Page root | `{page-id}` |
| Heading | `{page-id}-heading` |
| Input | `field-{name}` (kebab-case) |
| File path text | `field-file-path` |
| File picker | `field-file` |
| Review cell | `review-{name}` |

Replace `/` in ids with `-`. Required on every authored page: `wizard-back`,
`wizard-next`, `wizard-error`.

## Goldens

- `share/wyvern/examples/template-picker` — catalog from `config.templates`;
  finish `{ template_id, variables, output_path }`; post = `apply-template.py`
- `share/wyvern/examples/askuserquestion-hook` — pre patches `hook_state`;
  page toggles only; post applies hooks

## Output Format

Always return fenced JSON (skills guidelines v0.7). No unfenced JSON.

Success (`error` MUST be `null`):

```json
{
  "success": true,
  "data": {
    "action": "scaffold",
    "target_dir": "my-wizard/",
    "files": [
      "my-wizard/wizard.json",
      "my-wizard/app.js",
      "my-wizard/pages/one.html",
      "my-wizard/pages/two.html"
    ],
    "hooks": {
      "collectCurrentPageData": true,
      "wizardNextDescriptor": true,
      "wizardNextWizard": false
    },
    "disk_io": {
      "page_js": false,
      "workflow_pre": null,
      "workflow_post": null
    }
  },
  "error": null
}
```

Failure (`data` MAY be `null`):

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "VALIDATION.DISK_IO",
    "message": "Page JS must not read or write the filesystem",
    "recoverable": true,
    "suggested_action": "Move disk work to workflow.pre / workflow.post; keep only path strings in finish data"
  }
}
```

## Error Handling

### Handled by agent (recoverable)

| Code | Recovery |
|------|----------|
| `VALIDATION.INPUT` | Ask for `action` / `target_dir` |
| `VALIDATION.DISK_IO` | Strip FS calls; use workflow + path fields |
| `VALIDATION.MISSING_HOOK` | Add `collectCurrentPageData` / `wizardNextDescriptor` |
| `VALIDATION.MISSING_TESTID` | Add `wizard-back` / `wizard-next` / `wizard-error` |
| `VALIDATION.MISSING_CHROME` | Add `wyvern-api.js` + `wizard-nav.js` `data-wizard-chrome` |

### Propagated to skill (fatal)

| Code | Why |
|------|-----|
| `AUTHOR.TEMPLATE_MISSING` | vanilla-chrome template not in the skill tree |
| `AUTHOR.WRITE_FAILED` | destination unwritable |
| `AUTHOR.CRATES_SCOPE` | caller asked for `crates/**` or `rust-developer` |

## Constraints

- Vanilla chrome only (HTML + one `app.js`). No React/Svelte/turbo-flow.
- No `crates/**`. No `rust-developer`.
- Do not invent host routes or schema fields.
- Do not document `cp -R` as an install path when `workflow.post` exists.
- Do not put secrets in JSON output.
- Stay inside the repo / requested `target_dir`.
