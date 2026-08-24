# Path-picker wizards (`vanilla-chrome`)

Layer 3 type recipe. Load **after** `references/stacks/vanilla-chrome.md`.

**Agent:** `wyvern-wizard-js`  
**Stack:** `vanilla-chrome`  
**Lint profile:** `nav + dataflow-v1`  
**Golden example:** `share/wyvern/examples/path-picker/`

Use when the author needs **in-page native file/folder pickers** during a wizard
session (ADR-0026). Typical flow: agent pre-fills seed paths in `config`, user
browses for additional paths on one page, review page summarizes, finish JSON
carries path strings for `workflow.post`.

## When to use

| Intent | This type? |
|--------|------------|
| Browse files/folders with native OS picker on a wizard page | Yes |
| Catalog picker from `config.templates` only | No — [template.md](template.md) |
| Toggle hook files / settings | No — [hook.md](hook.md) |
| Canvas / agent DAG | No — [dag-wizards.md](dag-wizards.md) |

## Platform contract

Page JS calls **`WyvernApi.postPickerFile`** / **`postPickerFolder`** from
packaged `wyvern-api.js`. Host must allow **`Command::Wizard`** on picker routes
(REQ-HOST-0150). Collect **path strings only** — no filesystem reads/writes in
the browser (ADR-0006).

```javascript
const picked = await WyvernApi.postPickerFile({
  multiple: true,
  filter: ["*.csproj"],
});
if (!picked.ok || picked.cancelled) return;
// update DOM + stash in collectCurrentPageData()
```

## Finish shape (normative)

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

## Headless smoke

```bash
WYVERN_MOCK_PICKER_PATH=/tmp/fixture.txt \
  WYVERN_VIEWER=none \
  wyvern share/wyvern/examples/path-picker/wizard.json \
  --ui-root share/wyvern/examples/path-picker
```

## Cross-links

- [platform-contract.md](../core/platform-contract.md) — picker + wizard seams
- [i1-wizard-path-picker.md](../../../../docs/plans/phase-I/i1-wizard-path-picker.md) — sprint authority
- GitHub [#99](https://github.com/randlee/wyvern/issues/99)
