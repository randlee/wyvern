---
name: Template picker
description: Pick a bundled template and apply it through workflow.post.
---

# Template picker

Single-page wizard listing pytest, GitHub Actions, NUnit, xUnit, Benchmark .NET,
and wizard skeleton templates. Finish runs `apply-template.py` to write files
from the selected template — there is no shipped `cp -R` install path.

## Run

```bash
wyvern {wyvern_share}/examples/template-picker/wizard.json \
  --ui-root {wyvern_share}/examples/template-picker
```

Repo checkout:

```bash
wyvern share/wyvern/examples/template-picker/wizard.json \
  --ui-root share/wyvern/examples/template-picker
```

Lint:

```bash
wyvern wizard lint {wyvern_share}/examples/template-picker
```
