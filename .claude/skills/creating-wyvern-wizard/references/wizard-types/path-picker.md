# Path-picker wizards (`vanilla-chrome`)

Layer 3 type recipe. Load **after** `references/stacks/vanilla-chrome.md`.

**Agent:** `wyvern-wizard-js`  
**Stack:** `vanilla-chrome`  
**Lint profile:** `nav + dataflow-v1`  
**Golden example:** `share/wyvern/examples/path-picker/`

Use when the author needs **in-page native file/folder pickers** during a wizard
session (ADR-0026). Host `POST /api/picker/file` and `POST /api/picker/folder`
accept `Command::Wizard` (Phase I i.1 / #99). Page JS collects path strings
only — no filesystem reads or writes in the browser.

## When to use

| Intent | This type? |
|--------|------------|
| Browse files/folders on a wizard page, review, finish with path arrays | Yes |
| Catalog picker from `config.templates` only | No — [template.md](template.md) |
| Toggle hook files / settings | No — [hook.md](hook.md) |
| Hub card that chains to another wizard | No — [welcome-bridge.md](welcome-bridge.md) |
| Canvas / agent DAG | No — [dag-wizards.md](dag-wizards.md) |

## `config.seed_paths` (agent pre-fill)

Page JS reads **`config.seed_paths`** to pre-fill the in-page lists. Agents
pass absolute path strings; the user can browse additional paths on the same
page via `WyvernApi.postPickerFile` / `postPickerFolder`.

```json
{
  "config": {
    "seed_paths": {
      "file_paths": ["/abs/path/to/file.csproj"],
      "folder_paths": ["/abs/path/to/root"]
    }
  }
}
```

Never scan the filesystem for seeds. Paths are opaque strings.

## Platform contract

Page JS calls **`WyvernApi.postPickerFile`** / **`postPickerFolder`** from
packaged `wyvern-api.js`. Host must allow **`Command::Wizard(_)`** on picker
routes (REQ-HOST-0150). Collect **path strings only** — no filesystem
reads/writes in the browser (ADR-0006).

```javascript
const picked = await WyvernApi.postPickerFile({
  multiple: true,
  filter: ["*.csproj"],
});
if (!picked.ok || picked.cancelled) return;
// update DOM + stash in collectCurrentPageData()
```

## Dataflow (declare on new wizards)

When g.9+ `wyvern` is available, declare `config.dataflow`:

| Page | exports | requires |
|------|---------|----------|
| sources | `file_paths` (array), `folder_paths` (array) | — |
| review (terminal) | same keys | `file_paths`, `folder_paths` |

See [dataflow-contracts.md](../core/dataflow-contracts.md).

## Finish shape (normative)

- `collectCurrentPageData()` returns `{ file_paths, folder_paths }`.
- Wizard picker bodies use request-only defaults (`filter` → `[]`,
  `multiple` → `false`, `start_path` → omitted) when fields are omitted.

```json
{
  "button": "finish",
  "data": {
    "file_paths": ["/abs/path/to/file.csproj"],
    "folder_paths": ["/abs/path/to/root"]
  },
  "stack": []
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
- [i1-wizard-path-picker.md](../../../../../docs/plans/phase-I/i1-wizard-path-picker.md) — sprint authority
- [http-post-schema.md](../../../../../docs/plans/phase-C/http-post-schema.md) — picker routes
- GitHub [#99](https://github.com/randlee/wyvern/issues/99)
