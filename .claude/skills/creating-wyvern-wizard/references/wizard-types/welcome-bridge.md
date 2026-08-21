# Welcome bridge wizards (`vanilla-chrome`)

Layer 3 type recipe. Load **after** `references/stacks/vanilla-chrome.md`.

**Agent:** `wyvern-wizard-js`  
**Stack:** `vanilla-chrome`  
**Lint profile:** `nav + dataflow-v1`  
**Golden example:** `share/wyvern/welcome/`

Use when a **hub card** explains a feature and chains to a dedicated example
wizard via `wizardNextWizard` → finish `next_wizard` (REQ-0126).

## When to use

| Intent | This type? |
|--------|------------|
| Topic card with copy + chain to g.5/g.6/g.7 example | Yes |
| Full template picker inside one package | No — [template.md](template.md) |
| Hook toggle workflow | No — [hook.md](hook.md) |
| Canvas DAG | No — [dag-wizards.md](dag-wizards.md) |

## Hub vs bridge pages

| Page kind | Role |
|-----------|------|
| **Hub** (`home`) | Lists topics from `config.topics`; no `wizardNextWizard` on hub itself |
| **Overview** | Terminal explainer (`data-wizard-terminal="true"`) — Back/Finish only |
| **Bridge** | Prose + `wizardNextWizard` hop to another wizard package |

Bridge pages set finish `next_wizard` by assigning `window.wizardNextWizard`:

```js
window.wizardNextWizard = {
  path: "{wyvern_share}/examples/template-picker/wizard.json",
  input: {},
  ui_root: "{wyvern_share}/examples/template-picker"
};
```

Chrome: Back + Finish on bridge pages; Finish copies `next_wizard` onto the
finish JSON for the CLI chain loop.

## `config.topics`

```json
{
  "config": {
    "topics": [
      { "id": "templates", "label": "Template wizard", "html": "pages/templates.html" }
    ]
  }
}
```

Hub JS navigates to topic HTML paths; bridge pages live at those paths.

## Lint notes

- Bridge pages are terminal for **nav** when they only chain (`Finish` → next package).
- WIZARD-LINT-007 applies when `input` keys must match target `config.dataflow`
  (target undeclared → rule skipped).

## Authority

- Golden: `share/wyvern/welcome/`
- [g4-welcome-guide-wizard.md](../../../../../docs/plans/phase-G/g4-welcome-guide-wizard.md)
