# Stack: `vanilla-chrome` (default)

Dialog-frame wizards: welcome topics, template picker, AskUserQuestion hook, simple forms. **Use this stack unless the wizard is a canvas workspace.**

**Registry:** `status: default` · `lint_profile: nav + dataflow-v1` · agent `wyvern-wizard-js`  
**Golden:** `share/wyvern/examples/template-picker/`, `share/wyvern/examples/askuserquestion-hook/`, `share/wyvern/welcome/`

## When to use

- Multi-step pick → form → review
- Toggle / settings installers (`workflow.pre` / `workflow.post`)
- Welcome hub cards and bridge pages (`wizardNextWizard`)
- Any wizard that should run with `wyvern path/to/wizard.json` and **no npm build**

Do not use this stack for Agent DAG / turbo-flow canvases — see [workspace-canvas.md](workspace-canvas.md).

## Required includes

Host serves `ui/shared/*` at `/shared/…` (Phase D). Entry and inner pages opt in:

```html
<link rel="stylesheet" href="/shared/embedded-chrome.css">
<link rel="stylesheet" href="/shared/wizard-chrome.css">
<script src="/shared/wyvern-api.js"></script>
<script src="/shared/wizard-nav.js" data-wizard-chrome></script>
<script src="../app.js"></script>
```

| File | Role |
|------|------|
| `wyvern-api.js` | `window.wyvern` state, `wyvernWizardNext` / `Back` / `Finish`, viewport |
| `wizard-nav.js` | Opt-in chrome (`data-wizard-chrome`): Back / Next / Finish wiring |
| `embedded-chrome.css` / `wizard-chrome.css` | Dialog frame + nav bar |
| `app.js` | IIFE: `collectCurrentPageData`, `wizardNextDescriptor`, stack readers |

Page UI is plain HTML plus optional inline `<style>`. No React / Vue / Svelte source in the package.

## Navigation contract

Opt-in chrome (d.7):

- Region: `[data-wizard-nav]`
- Back: `[data-wizard-back]` or `[data-testid="wizard-back"]` — hidden on stack[0] (entry)
- Next: `[data-wizard-next]` or `[data-testid="wizard-next"]` — label flips to Finish on terminal
- Cancel: `[data-wizard-cancel]`, `[data-testid="wizard-cancel"]`, or `<button>Cancel</button>` on **terminal** pages
- Terminal: `data-wizard-terminal="true"` on a root in the page

`wizard-nav.js` calls:

- Next → `wyvernWizardNext(collectCurrentPageData(), nextDescriptor)`
- Finish → `wyvernWizardFinish({ button, data, stack, next_wizard? })`
- Back → `wyvernWizardBack()`

Authors supply:

```js
function collectCurrentPageData() {
  return { /* object, never undefined */ };
}
var wizardNextDescriptor = function () {
  return { id: "form", title: "Configure", html: "pages/form.html" };
};
// Welcome / chain pages only:
var wizardNextWizard = {
  path: "{wyvern_share}/examples/template-picker/wizard.json",
  input: { from: "welcome" },
  ui_root: "{wyvern_share}/examples/template-picker"
};
```

## Dataflow

Declare `config.dataflow` per [dataflow-contracts.md](../core/dataflow-contracts.md).

Template-picker finish `data`:

```json
{
  "template_id": "pytest",
  "variables": { "module_name": "example" },
  "output_path": "tests/test_example.py"
}
```

AskUserQuestion finish `data.hook_config`: `{ "global": { "enabled": true }, "repo": { "enabled": false } }`.

Read prior steps with a `selectionFromStack`-style walk of `window.wyvern.stack` (last writer wins). Do not scan the filesystem for catalogs — read `config.templates` / `config.hook_state` only.

## Workflow scripts

| Phase | Author does | Script does |
|-------|-------------|-------------|
| Pre | Nothing in page JS | stdout `{ "config_patch": { … } }` |
| Page | Collect path strings / toggles | — |
| Post | — | stdin = finish JSON; write files (`--dry-run` writes nothing) |

Timeout 30s. Paths under `{wyvern_share}`, cwd, or the wizard directory (g.4 allowlist).

## Lint profile (`nav + dataflow-v1`)

| Code | Applies |
|------|---------|
| WIZARD-LINT-001 | Inner pages need Back |
| WIZARD-LINT-002 | Terminal pages need Cancel |
| WIZARD-LINT-003 | Chrome opt-in needs `[data-wizard-nav]` |
| WIZARD-LINT-004 | Non-terminal chrome pages need Next |
| WIZARD-LINT-005–008 | When `config.dataflow` is declared (g.9) |

Known Wave 2 finding: template-picker `pages/review.html` is terminal without Cancel (WIZARD-LINT-002). Fix is **g.14**, not this stack doc.

## Tests

- `data-testid` on picker rows, toggles, and primary nav
- Workflow tests assert finish `data` keys and `--workflow-dry-run`
- In-repo examples: keep `share/wyvern/` and `crates/wyvern/embedded/` in sync

## Non-goals

- Bundler / `package.json` in the wizard package
- `page.layout: "workspace"`
- Custom canvas toolbars
- Page-JS `fetch` to write repo files
