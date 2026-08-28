---
name: AskUserQuestion hook
description: Query, install, and toggle Claude Code AskUserQuestion PreToolUse hooks.
---

# AskUserQuestion hook

Single-page wizard that reads hook state with `workflow.pre`, lets the user
toggle global/repo installs, then applies changes with `workflow.post`.

## Run

```bash
wyvern {wyvern_share}/examples/askuserquestion-hook/wizard.json \
  --ui-root {wyvern_share}/examples/askuserquestion-hook
```

Repo checkout:

```bash
wyvern share/wyvern/examples/askuserquestion-hook/wizard.json \
  --ui-root share/wyvern/examples/askuserquestion-hook
```

Lint:

```bash
wyvern wizard lint {wyvern_share}/examples/askuserquestion-hook
```
