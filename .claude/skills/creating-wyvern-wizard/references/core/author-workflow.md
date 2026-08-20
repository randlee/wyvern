# Author workflow (gates G1–G6)

Fail-fast checklist for creating or changing a Wyvern wizard. Load this file from the skill Layer 0 router (g.10). Do not load every stack or type reference up front.

**Rule:** stop at the first failing gate. Fix it before continuing.

## Progressive load

```
Layer 0  SKILL.md                 Router (g.10)
Layer 1  references/core/         This file + dataflow + (later) platform / lint / test
Layer 2  references/stacks/       One stack doc
Layer 3  references/wizard-types/ One type doc (g.12–g.13)
Layer 4  templates/               Skeleton copy (g.11+)
```

After G2, open **one** Layer 2 stack doc and **one** Layer 3 type doc only.

## Gate G1 — Schema

`wizard.json` must be a valid wizard command **before** HTML or JS work.

- `type` is `"wizard"`
- `page.id`, `page.title`, `page.html` present
- Optional `workflow.pre` / `workflow.post` are allowlisted path strings (`{wyvern_share}`, cwd, or wizard dir)
- Optional `width` / `height` / `config` as today

How: run the same validate path the CLI uses (`wyvern <wizard.json>` fails closed on schema errors; g.10 documents the exact command). Do not invent a second schema.

Entry page lives only in `wizard.json`. Further pages are **not** a top-level `pages` map — they appear via `app.js` hops (`wizardNextDescriptor`, `wyvernWizardNext`).

## Gate G2 — Stack

Choose exactly one row from [registry.yaml](../stacks/registry.yaml).

| If the wizard is… | Stack | Doc | Agent |
|-------------------|-------|-----|-------|
| Dialog-frame form, picker, hook, welcome topic | **`vanilla-chrome`** (default) | [vanilla-chrome.md](../stacks/vanilla-chrome.md) | `wyvern-wizard-js` |
| Canvas / graph / Agent DAG workspace | **`workspace-canvas`** | [workspace-canvas.md](../stacks/workspace-canvas.md) | `wyvern-dag-wizard-js` |

If G2 is skipped, assume `vanilla-chrome`. Do not mix wizard-nav chrome and a custom canvas toolbar on the same page without documenting why in the stack doc.

## Gate G3 — Page graph

The package must be a complete, walkable graph:

1. Entry HTML at `page.html` exists.
2. Every hop target HTML file exists.
3. `app.js` (or the stack's page logic file) defines:
   - `collectCurrentPageData()` → object (never `undefined`)
   - `wizardNextDescriptor` on non-terminal pages (function or object)
   - `wizardNextWizard` only on welcome-bridge / chain pages (REQ-0126)
4. Terminal pages set `data-wizard-terminal="true"`.
5. `config.dataflow.pages` keys match the graph ids (see [dataflow-contracts.md](dataflow-contracts.md)).

Page JS runs in the webview. **No disk I/O.** Read `window.wyvern` (`config`, `page`, `page_data`, `stack`). Write outcomes through next/finish. Persist with `workflow.post`; pre-fill with `workflow.pre` `config_patch`.

File/path picking in the UI is a **string in finish data** (`<input type="file">` or a text field). The post script writes bytes.

## Gate G4 — Lint

```bash
wyvern wizard lint path/to/wizard-dir
```

| Profile | Rules |
|---------|--------|
| `nav` | WIZARD-LINT-001–004 (back / cancel / nav region / next) |
| `dataflow-v1` | WIZARD-LINT-005–008 when `config.dataflow` is declared (g.9) |
| `nav-limited` | Workspace-canvas may omit full wizard-nav; cancel/export still required on terminal |
| `export-contract` | `data.dag` literals for workspace-canvas |

Exit 0 = clean, 1 = findings, 2 = usage error (existing CLI contract).

Fix findings in HTML/JS/declarations. Do not disable rules to close a sprint.

Until g.9 merges, G4 is **nav-only** plus a manual review of the `config.dataflow` object against [dataflow-contracts.md](dataflow-contracts.md).

## Gate G5 — Tests and dry-run

Minimum:

- Integration or workflow test that drives the happy path to finish JSON
- `data-testid` on primary controls (grid rows, toggles, finish)
- If `workflow.post` exists: `wyvern … --workflow-dry-run` prints the plan / writes nothing (REQ-0125)

Optional: Playwright L2 for chrome/layout. Not a substitute for finish-JSON asserts.

Shipped examples under `share/wyvern/` often need `crates/wyvern/embedded/` parity (`scripts/check-share-sync.sh`). Authors of in-repo examples run that sync; third-party wizards outside the repo do not.

## Gate G6 — Doc / REQ hygiene

If shipped behavior changed (finish shape, workflow paths, welcome `next_wizard`, lint codes):

- Update the owning sprint doc so QA cannot fail on doc drift (ATM-QA-002)
- Cite REQ-0124 / 0125 / 0126 when touching pre, post, or chain
- Do not leave the sprint doc listing stubs that the tree no longer ships

If this is a new in-repo example, add or amend a Phase G sprint doc (or a follow-up issue) before calling the work done.

## Agent delegation

| Task | Agent |
|------|--------|
| Dialog pages, forms, hooks, welcome bridges | `wyvern-wizard-js` |
| Canvas DAG / workspace | `wyvern-dag-wizard-js` |
| Lint failures in the package | Same agent as the stack |

Do not send authoring work to CLI/host implementers. Those changes are a different skill.

## Manual smoke (not a QA gate)

```bash
wyvern share/wyvern/examples/template-picker/wizard.json \
  --ui-root share/wyvern/examples/template-picker --viewer none
```

Or `wyvern guide` and hop a welcome topic.

## Authority

- [g8-wizard-authoring-foundation.md](../../../../../docs/plans/phase-G/g8-wizard-authoring-foundation.md)
- [dataflow-contracts.md](dataflow-contracts.md)
- [registry.yaml](../stacks/registry.yaml)
- REQ-0124, REQ-0125, REQ-0126, ADR-0006
