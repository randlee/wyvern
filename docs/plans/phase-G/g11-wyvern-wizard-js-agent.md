---
id: g.11
title: Vanilla-chrome wizard JS authoring agent
status: complete (integrate)
branch: feature/phase-G-g11-wizard-js-agent
worktree: ../wyvern-worktrees/feature/phase-G-g11-wizard-js-agent
target: integrate/phase-G
---

# Sprint g.11 — Wizard JS authoring agent

## Goal

Give a cold agent a single, copy-safe way to author **vanilla-chrome** wizard
pages. Ship `wyvern-wizard-js` (execution layer) plus a two-step skeleton under
`creating-wyvern-wizard`. Page JS collects opaque finish data only; disk I/O
stays in `workflow.pre` / `workflow.post` (REQ-0124, REQ-0125, ADR-0023).
Chaining uses `wizardNextWizard` → finish `next_wizard` (REQ-0126, ADR-0024).

This sprint does **not** use `rust-developer`. No `crates/**` changes.

## Hard dependencies

- g.4 merged (`workflow.pre` / `workflow.post`, `next_wizard` copy + CLI resolve)
- g.5–g.6 golden examples on `integrate/phase-G`:
  `share/wyvern/examples/askuserquestion-hook`,
  `share/wyvern/examples/template-picker`
- Shared chrome already shipped: `ui/shared/wyvern-api.js`,
  `ui/shared/wizard-nav.js` (served as `/shared/…`)

## Exact targets

- `docs/plans/phase-G/g11-wyvern-wizard-js-agent.md`
- `.claude/agents/wyvern-wizard-js.md`
- `.claude/agents/registry.yaml`
- `.claude/skills/creating-wyvern-wizard/SKILL.md`
- `.claude/skills/creating-wyvern-wizard/templates/vanilla-chrome/wizard.json`
- `.claude/skills/creating-wyvern-wizard/templates/vanilla-chrome/app.js`
- `.claude/skills/creating-wyvern-wizard/templates/vanilla-chrome/pages/one.html`
- `.claude/skills/creating-wyvern-wizard/templates/vanilla-chrome/pages/two.html`

## Deliverables

Every listed deliverable lands at a production-ready level for this sprint.
No deliverable may be silently dropped or partially deferred.

| Path | Purpose |
|------|---------|
| `docs/plans/phase-G/g11-wyvern-wizard-js-agent.md` | Sole authority for g.11 AC and validation |
| `.claude/agents/wyvern-wizard-js.md` | Vanilla-chrome page authoring agent (v0.7 fenced JSON) |
| `.claude/agents/registry.yaml` | Register `wyvern-wizard-js` 0.1.0 (and skill `depends_on`) |
| `.claude/skills/creating-wyvern-wizard/SKILL.md` | Discovery layer; delegates to `wyvern-wizard-js` |
| `.claude/skills/creating-wyvern-wizard/templates/vanilla-chrome/` | Minimal two-step skeleton (`wizard.json`, `app.js`, `pages/`) |

**Do not** spawn or assign `rust-developer`. **Do not** edit `crates/**`.

### Contracts

#### Shared scripts (required on every page)

```html
<link rel="stylesheet" href="/shared/embedded-chrome.css" />
<link rel="stylesheet" href="/shared/wizard-chrome.css" />
<script src="/shared/wyvern-api.js"></script>
<!-- page body: app.js then chrome opt-in -->
<script src="../app.js"></script>
<script src="/shared/wizard-nav.js" data-wizard-chrome></script>
```

`wizard-nav.js` without `data-wizard-chrome` does not wire Back/Next/Finish.

#### Page-author hooks (window globals)

| Hook | Required | Role |
|------|----------|------|
| `collectCurrentPageData()` | yes | Opaque blob for next / back / finish. Missing / null → `{}` |
| `wizardNextDescriptor` | non-terminal Next | `{ id, title, html }` or `() => ({ id, title, html })` |
| `wizardNextWizard` | optional | `{ path, input?, ui_root? }` or function; copied onto finish (REQ-0126) |

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

// Optional CLI hop. Omit when this wizard does not chain.
window.wizardNextWizard = {
  path: "{wyvern_share}/examples/template-picker/wizard.json",
  input: { from: "author" },
  ui_root: "{wyvern_share}/examples/template-picker"
};
```

Terminal page root sets `data-wizard-terminal="true"`. Chrome relabels Next →
Finish and calls `wyvernWizardFinish({ button, data, stack, next_wizard? })`.

#### Disk I/O (normative)

Page JS **must not** read or write the filesystem. Disk belongs to workflow:

| Phase | Owner | I/O |
|-------|-------|-----|
| Pre (REQ-0124) | CLI `workflow.pre` | stdout `{ "config_patch": { … } }`; page reads `window.wyvern.config` |
| Page | `app.js` | Form fields only; opaque `data` |
| Post (REQ-0125) | CLI `workflow.post` | finish JSON on stdin; writes files |

Forbidden in page JS (agent must refuse / lint-fail):

- `fetch("file://…")`, `XMLHttpRequest` to local files
- Node/Deno/`require("fs")` / `child_process`
- File System Access API writes (`showSaveFilePicker`, `createWritable`)
- `FileReader` used to persist or parse a picked file into disk side effects
- Directory scans to invent catalog rows (g.6: read `config.templates` only)

Allowed path **collection** (strings in finish `data` only):

- `<input type="text" data-testid="field-file-path">` (g.6 `field-output-path`)
- `<input type="file" data-testid="field-file">` — copy `file.path || file.name`
  into the text field / finish blob; do **not** read file bytes
- `WyvernApi.postPickerFile` / `postPickerFolder` — host returns a path string;
  store that string in `data`

Finish `data` example (paths are strings; post script consumes them):

```json
{
  "label": "Notes",
  "file_path": "docs/notes.txt"
}
```

Golden examples (do not regress their contracts):

- `share/wyvern/examples/template-picker` — `config.templates` only; no catalog
  scan; `output_path` in finish `data`; `workflow.post` = `apply-template.py`
- `share/wyvern/examples/askuserquestion-hook` — `workflow.pre` fills
  `config.hook_state` (including display paths); page JS toggles only;
  `workflow.post` applies hook files

#### `data-testid` conventions

| Role | `data-testid` | Also |
|------|---------------|------|
| Back | `wizard-back` | `data-wizard-back` |
| Next / Finish | `wizard-next` | `data-wizard-next` |
| Error | `wizard-error` | hidden until set |
| Page root | `{page-id}` | `id="dialog"` + `dialog dialog--frame` |
| Heading | `{page-id}-heading` | |
| Form field | `field-{name}` | kebab-case |
| File path text | `field-file-path` | |
| File picker | `field-file` | `type="file"` |
| Review value | `review-{name}` | |

Slash in ids becomes hyphen (`wizard/minimal` → `template-card-wizard-minimal`
in g.6). Nav root: `data-wizard-nav`.

#### Agent fenced JSON (skills guidelines v0.7)

`wyvern-wizard-js` always ends with a fenced JSON envelope. `success: true`
requires `error: null`. Failure populates `error` (`NAMESPACE.CODE`).

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

## Required work

- Author `wyvern-wizard-js` so a fresh agent can scaffold, author, or lint
  vanilla-chrome pages from the contracts above without opening `crates/**`.
- Copy the two-step skeleton from the skill templates when `action` is
  `scaffold` (or instruct the caller to copy them).
- Register the agent (and skill `depends_on`) in `.claude/agents/registry.yaml`.
  Frontmatter `version` must match the registry version.
- Index g.11 on the Phase G README as Wave 3 (authoring). Do not rewrite Wave 1–2
  AC or validation lists.

## Explicit code samples

See Contracts. Additional chrome wiring from `wizard-nav.js`:

```js
// Next: wyvernWizardNext(collectCurrentPageData(), wizardNextDescriptor)
// Back: wyvernWizardBack() — uses collectCurrentPageData when defined
// Finish stack = window.wyvern.stack + { page, data }
```

`wizard.json` skeleton (no `workflow` until a real pre/post script exists):

```json
{
  "type": "wizard",
  "page": {
    "id": "one",
    "title": "Step 1",
    "html": "pages/one.html"
  },
  "width": 640,
  "height": 400
}
```

## This sprint does not close

- Rust / `crates/**` / `rust-developer` implementation
- New host routes, schema fields, or dialog types
- Replacing `share/wyvern/templates/wizard/two-step/` (catalog skeleton stays)
- React / Svelte / turbo-flow authoring (g.7 canvas remains a special case)
- Workflow script authoring (`scripts/ext/*.py`) beyond pointing at pre/post
- Claude Code hook / MCP / `--interactive` auto-chain (Phase E)
- `--emit-all`, `wyvern chain` subcommand

## Acceptance criteria

1. `.claude/agents/wyvern-wizard-js.md` exists with YAML `name: wyvern-wizard-js`
   and `version: 0.1.0`. It documents `wyvern-api.js`, `wizard-nav.js`,
   `collectCurrentPageData`, `wizardNextDescriptor`, and `wizardNextWizard`.
2. The agent forbids disk I/O in page JS and requires `workflow.pre` /
   `workflow.post` for disk. File paths are collected via form fields and/or
   `<input type="file">` and appear only in finish `data`.
3. The agent states the `data-testid` table in Contracts and requires
   `wizard-back`, `wizard-next`, and `wizard-error` on every skeleton page.
4. The agent’s required output is a v0.7 fenced JSON envelope
   (`success`, `data`, `error`) with the error object shape in Contracts.
5. `templates/vanilla-chrome/` is a two-step wizard: `wizard.json`, `app.js`,
   `pages/one.html`, `pages/two.html`. Pages load `/shared/wyvern-api.js` and
   `/shared/wizard-nav.js` with `data-wizard-chrome`. `app.js` defines
   `collectCurrentPageData` and `wizardNextDescriptor`. Page two is terminal
   (`data-wizard-terminal="true"`).
6. `.claude/agents/registry.yaml` lists `wyvern-wizard-js` at `0.1.0` with
   `path: .claude/agents/wyvern-wizard-js.md`. Version in frontmatter equals
   the registry version.
7. No file under `crates/` is added, modified, or deleted on this branch vs
   `integrate/phase-G`.

## Required validation

```bash
test -f docs/plans/phase-G/g11-wyvern-wizard-js-agent.md
test -f .claude/agents/wyvern-wizard-js.md
test -f .claude/skills/creating-wyvern-wizard/SKILL.md
test -f .claude/skills/creating-wyvern-wizard/templates/vanilla-chrome/wizard.json
test -f .claude/skills/creating-wyvern-wizard/templates/vanilla-chrome/app.js
test -f .claude/skills/creating-wyvern-wizard/templates/vanilla-chrome/pages/one.html
test -f .claude/skills/creating-wyvern-wizard/templates/vanilla-chrome/pages/two.html
```

```bash
rg -q "collectCurrentPageData" .claude/agents/wyvern-wizard-js.md
rg -q "wizardNextDescriptor" .claude/agents/wyvern-wizard-js.md
rg -q "wizardNextWizard" .claude/agents/wyvern-wizard-js.md
rg -q "workflow.pre" .claude/agents/wyvern-wizard-js.md
rg -q "workflow.post" .claude/agents/wyvern-wizard-js.md
rg -q "data-testid" .claude/agents/wyvern-wizard-js.md
rg -q '"success"' .claude/agents/wyvern-wizard-js.md
rg -q "VALIDATION.DISK_IO" .claude/agents/wyvern-wizard-js.md
```

```bash
rg -q "collectCurrentPageData" .claude/skills/creating-wyvern-wizard/templates/vanilla-chrome/app.js
rg -q "wizardNextDescriptor" .claude/skills/creating-wyvern-wizard/templates/vanilla-chrome/app.js
rg -q "/shared/wyvern-api.js" .claude/skills/creating-wyvern-wizard/templates/vanilla-chrome/pages/one.html
rg -q "data-wizard-chrome" .claude/skills/creating-wyvern-wizard/templates/vanilla-chrome/pages/one.html
rg -q "data-wizard-chrome" .claude/skills/creating-wyvern-wizard/templates/vanilla-chrome/pages/two.html
rg -q 'data-wizard-terminal="true"' .claude/skills/creating-wyvern-wizard/templates/vanilla-chrome/pages/two.html
rg -q 'data-testid="wizard-back"' .claude/skills/creating-wyvern-wizard/templates/vanilla-chrome/pages/one.html
rg -q 'data-testid="wizard-next"' .claude/skills/creating-wyvern-wizard/templates/vanilla-chrome/pages/one.html
rg -q 'data-testid="wizard-error"' .claude/skills/creating-wyvern-wizard/templates/vanilla-chrome/pages/one.html
rg -q 'data-testid="field-file-path"' .claude/skills/creating-wyvern-wizard/templates/vanilla-chrome/pages/one.html
```

```bash
python3 - <<'PY'
import pathlib, re
agent = pathlib.Path(".claude/agents/wyvern-wizard-js.md").read_text()
reg = pathlib.Path(".claude/agents/registry.yaml").read_text()
m = re.search(r"^version:\s*([0-9.]+)", agent, re.M)
assert m, "agent frontmatter version missing"
assert f"wyvern-wizard-js:\n    version: {m.group(1)}" in reg
assert "path: .claude/agents/wyvern-wizard-js.md" in reg
print("registry version ok", m.group(1))
PY
```

```bash
git diff --name-only origin/integrate/phase-G...HEAD | python3 -c "import sys; p=[l.strip() for l in sys.stdin if l.startswith('crates/')];
assert not p, 'crates/ must not change: ' + ', '.join(p)"
git diff --check
```

## Authority

- REQ-0024, REQ-0025, REQ-0026 (page state + explicit next descriptors)
- REQ-0112 (`data-testid` on primary actions)
- REQ-0124, REQ-0125, REQ-0126; ADR-0023, ADR-0024, ADR-0006
- [wizard-workflow-architecture.md](wizard-workflow-architecture.md)
- `ui/shared/wyvern-api.js`, `ui/shared/wizard-nav.js`
- Golden: `share/wyvern/examples/template-picker`,
  `share/wyvern/examples/askuserquestion-hook`
- Skills/agents guidelines v0.7 (fenced JSON envelope + registry versions)
