---
name: wyvern-reporting
version: 0.1.0
description: >-
  Author sc-compose XHTML report panels, write review manifests, and run
  wyvern report-xhtml (including --review). Use when creating or revising
  .xhtml fragments, review.json, or parsing report finish JSON. Do not use
  creating-wyvern-wizard for report panels — use this skill instead.
---

# Wyvern reporting

Layer 0 router (skills/agents guidelines v0.7). This file is the table of
contents. Load **one** core reference at a time. Report viewing is
`type: "report"` — a static XHTML document, optionally with one terminal
review action.

## Progressive disclosure load map

```
Layer 0  this SKILL.md
Layer 1  references/core/panel-authoring.md   author/fix .xhtml fragments
         references/core/review-manifest.md   review.json + CLI
         references/core/review-ux.md         Approve/Cancel + finish JSON
         references/core/installation.md      wyvern, sc-compose, python3
Layer 2  templates/                           panel.xhtml.j2 + review.json
```

Read Layer 1 in that order: **panel → manifest → review**. Open example
paths last (below). Do not load every reference up front.

## In scope / out of scope

| In scope | Out of scope |
|----------|--------------|
| XHTML panel fragments for report viewing | Wizard pages and wizard packages |
| `wyvern panel.xhtml`, `wyvern report-xhtml` | `creating-wyvern-wizard` — use this skill instead |
| `--review` agent loops parsing finish JSON | Published aggregate HTML reports |

## Agent workflow

1. Author or fix `.xhtml` panel(s) via the sc-compose template
   (`templates/panel.xhtml.j2`). See [panel-authoring.md](references/core/panel-authoring.md).
2. Write `review.json` listing those panels; set `role: "proposal"` on the
   fix panel. See [review-manifest.md](references/core/review-manifest.md).
3. Run `wyvern report-xhtml --review review.json`.
4. Parse finish JSON ([review-ux.md](references/core/review-ux.md)). If
   `data.approved` is not `true`, revise the panels and re-run from step 1.

## Example paths

| Path | Role |
|------|------|
| `.claude/skills/wyvern-reporting/templates/panel.xhtml.j2` | Minimal fragment starter |
| `.claude/skills/wyvern-reporting/templates/review.json` | Manifest starter (schema-valid) |
| `fixtures/xhtml/panel.xhtml` | Single-panel fragment fixture |
| `crates/wyvern/tests/fixtures/xhtml-review/` | Host/CLI review fixtures |
| `share/wyvern/examples/xhtml-review/` | Packaged example tree (lands in h.5) |

Until the share example tree exists, copy the skill templates and the
`xhtml-review` test fixtures. Do not invent a second manifest schema.

## Authority

Normative contract: [docs/plans/phase-H/xhtml-reporting-contract.md](../../../docs/plans/phase-H/xhtml-reporting-contract.md).
Manifest schema: [docs/plans/phase-H/review-manifest.schema.json](../../../docs/plans/phase-H/review-manifest.schema.json).
