---
name: creating-wyvern-wizard
version: 0.1.0
description: >-
  Create or lint vanilla-chrome Wyvern wizard pages (wizard.json, app.js, HTML).
  Use when authoring a wizard UI, wiring collectCurrentPageData / wizard-nav,
  collecting file paths for workflow.post, or when the user mentions wizard
  pages, next_wizard, or vanilla chrome. Do not use for Rust crates or
  rust-developer work.
---

# Creating Wyvern Wizards

Discovery layer for vanilla-chrome wizard authoring. Execution is
`wyvern-wizard-js`.

## When to use

- Scaffold a two-step wizard from the bundled template
- Author pages that must speak `wyvern-api.js` / `wizard-nav.js`
- Lint page JS for disk I/O leaks (paths belong in finish `data` only)

Do not use for Rust, host routes, or `crates/**`.

## Agent Delegation

| Operation | Agent | Returns |
|-----------|-------|---------|
| Scaffold / author / lint | `wyvern-wizard-js` | Fenced JSON: files, hooks, disk_io |

Invoke `.claude/agents/wyvern-wizard-js.md` (registry
`.claude/agents/registry.yaml`) with:

```json
{
  "action": "scaffold",
  "target_dir": "my-wizard/"
}
```

Receive the agent's fenced JSON. Present a short summary to the user; do not
re-wrap the envelope.

## Template

Copy or adapt:

`.claude/skills/creating-wyvern-wizard/templates/vanilla-chrome/`

- `wizard.json` — entry page
- `app.js` — `collectCurrentPageData` + `wizardNextDescriptor`
- `pages/one.html`, `pages/two.html` — chrome + `data-testid`

Goldens: `share/wyvern/examples/template-picker`,
`share/wyvern/examples/askuserquestion-hook`.

## Rules (do not relax)

- Page JS never reads or writes disk. Use `workflow.pre` / `workflow.post`.
- File paths: text field and/or `<input type="file">`; strings in finish `data`.
- Required testids: `wizard-back`, `wizard-next`, `wizard-error`.
- `wizard-nav.js` must have `data-wizard-chrome`.
