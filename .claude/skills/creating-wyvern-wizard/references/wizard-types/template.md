# Template wizards (`vanilla-chrome`)

Layer 3 type recipe. Load **after** `references/stacks/vanilla-chrome.md`.

**Agent:** `wyvern-wizard-js`  
**Stack:** `vanilla-chrome`  
**Lint profile:** `nav + dataflow-v1`  
**Golden example:** `share/wyvern/examples/template-picker/`

Use when the author needs a **catalog picker → form → review** flow that
finishes with path strings and opaque variables for a `workflow.post` script.

## When to use

| Intent | This type? |
|--------|------------|
| Pick catalog row, customize fields, review output path | Yes |
| Browse native file/folder paths in-page | No — [path-picker.md](path-picker.md) |
| Toggle hook files / settings | No — [hook.md](hook.md) |
| Hub card that chains to another wizard | No — [welcome-bridge.md](welcome-bridge.md) |
| Canvas / agent DAG | No — [dag-wizards.md](dag-wizards.md) |

## `config.templates` (no directory scan)

Page JS reads **`config.templates` only** (g.6). Never scan the filesystem
for catalog rows.

```json
{
  "config": {
    "templates": [
      {
        "id": "pytest",
        "label": "pytest",
        "default_output_path": "tests/test_example.py",
        "variables": [{ "name": "module_name", "default": "example" }]
      }
    ]
  }
}
```

## Dataflow (declare on new wizards)

When g.9+ `wyvern` is available, declare `config.dataflow`:

| Page | exports | requires |
|------|---------|----------|
| pick | `template_id`, `variables`, `output_path` | — |
| form | same keys | `template_id` |
| review (terminal) | same + `post_input` matching finish `data` | `template_id`, `output_path` |

See [dataflow-contracts.md](../core/dataflow-contracts.md).

## Finish + post

- `collectCurrentPageData()` returns `{ template_id, variables, output_path }`.
- `workflow.post` reads finish stdin (e.g. `apply-template.py`).
- Page JS **must not** write files.

## Optional sc-compose HTML

Authors may render static page bodies with J2 — copy
`templates/sc-compose/` and run:

```bash
wyvern compose render --root my-wizard --file pages/pick.j2 --var title="Pick"
```

See [templates/sc-compose/README.md](../../templates/sc-compose/README.md).

## Authority

- Golden: `share/wyvern/examples/template-picker/`
- [g6-template-wizard.md](../../../../../docs/plans/phase-G/g6-template-wizard.md)
