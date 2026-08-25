# Hook and settings wizards (`vanilla-chrome`)

Layer 3 type recipe. Load **after** `references/stacks/vanilla-chrome.md`.

**Agent:** `wyvern-wizard-js`  
**Stack:** `vanilla-chrome`  
**Lint profile:** `nav + dataflow-v1`  
**Golden example:** `share/wyvern/examples/askuserquestion-hook/`

Use when the author toggles **settings represented as files** (hook JSON,
feature flags) with `workflow.pre` seeding state and `workflow.post` applying
changes. Page JS toggles UI only — no disk I/O.

## When to use

| Intent | This type? |
|--------|------------|
| Enable/disable hook files from pre-filled config | Yes |
| Template catalog + output path | No — [template.md](template.md) |
| Browse native file/folder paths in-page | No — [path-picker.md](path-picker.md) |
| Welcome hub chain hop | No — [welcome-bridge.md](welcome-bridge.md) |
| Agent DAG canvas | No — [dag-wizards.md](dag-wizards.md) |

## `workflow.pre` + `config`

Pre script stdout merges `{ "config_patch": { … } }` before bind (REQ-0124).
The hook example fills `config.hook_state` with installed/enabled flags and
display paths. Pages read `window.wyvern.config` — do **not** re-declare
pre keys as page `exports`.

```json
{
  "workflow": {
    "pre": "{wyvern_share}/scripts/ext/query-askuserquestion-hook.py",
    "post": "{wyvern_share}/scripts/ext/apply-askuserquestion-hook.py"
  }
}
```

## Finish shape

Terminal page exports `hook_config` object (g.5):

```json
{
  "hook_config": {
    "global": { "enabled": true },
    "repo": { "enabled": false }
  }
}
```

Declare on the terminal page in `config.dataflow` when using g.9 lint.

## Page pattern

- Single dialog or short multi-step with chrome opt-in.
- Toggles write boolean/state into finish `data` only.
- `data-testid` on each toggle (`field-*`).

## Authority

- Golden: `share/wyvern/examples/askuserquestion-hook/`
- [g5-askuserquestion-claude-code.md](../../../../../docs/plans/phase-G/g5-askuserquestion-claude-code.md)
