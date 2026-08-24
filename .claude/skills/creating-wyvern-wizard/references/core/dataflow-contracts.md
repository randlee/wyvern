# Dataflow contracts (`exports` / `requires`)

Normative declaration for **WIZARD-LINT-005–008** (g.9 implements; this file is the spec). Host and schema treat `config` and finish `data` as opaque (ADR-0006). Lint is the only consumer of these declarations.

Nav rules **WIZARD-LINT-001–004** stay in `wyvern wizard lint` as shipped on `feature/phase-G-wizard-lint`. They do not read this contract.

**Load this file when:** declaring or reviewing page-to-page data, wiring `workflow.post` stdin, or implementing dataflow lint.

## 1. Why a declaration exists

Page JS writes an opaque blob via `collectCurrentPageData()`. The host does not type-check it. Without a declared contract, lint cannot know whether step N's output satisfies step N+1 or the post script.

Static JS analysis is **partial**. Lint catches **declared** violations. Integration tests prove behavior.

## 2. Declaration site (version 1)

Normative location: `wizard.json` → `config.dataflow` (no new top-level schema field).

```json
{
  "config": {
    "dataflow": {
      "version": 1,
      "pages": {
        "<page-id>": {
          "exports": { "<key>": "<type>" },
          "requires": ["<key>"],
          "terminal": false,
          "post_input": { "<key>": "<type>" }
        }
      }
    }
  }
}
```

| Field | Required | Meaning |
|-------|----------|---------|
| `version` | yes | Integer `1` for this spec |
| `pages` | yes | Map keyed by page `id` (entry id from `page.id`; later ids from `wizardNextDescriptor`) |
| `pages.*.exports` | no | Keys this page writes onto `stack[].data` / finish `data` |
| `pages.*.requires` | no | Keys that must already exist on some prior page `exports` on **every reachable path** to this page |
| `pages.*.terminal` | no | `true` if this page may finish the wizard (`data-wizard-terminal="true"`) |
| `pages.*.post_input` | terminal only | Shape `workflow.post` reads from finish `data` (or assembled finish `data` for canvas) |

Omitted `config.dataflow`: g.9 **skips** WIZARD-LINT-005–008 for that package (nav rules still run). Authors adding a new wizard in Wave 3 **must** declare dataflow (gate G4).

### Type tokens

| Token | JSON value |
|-------|------------|
| `string` | JSON string |
| `number` | JSON number |
| `boolean` | JSON boolean |
| `object` | JSON object (not array, not null) |
| `array` | JSON array |
| `any` | present, any JSON value |

Types are **presence + JSON kind** only. Nested object schemas are out of v1 except the documented `dag` object (see §6).

### Optional HTML overlay

A page may repeat or narrow the declaration:

```html
<meta name="wyvern-dataflow-exports" content="template_id:string,variables:object,output_path:string">
<meta name="wyvern-dataflow-requires" content="template_id">
```

If both `config.dataflow` and meta exist for the same page id, **union exports** and **union requires**. Conflicting types for the same key → WIZARD-LINT-005 (treat as unsatisfied).

## 3. Universal stack shape (all stacks)

Finish JSON (REQ-0024 / d.7):

```json
{
  "button": "finish",
  "data": {},
  "stack": [{ "page": { "id": "…", "title": "…", "html": "…" }, "data": {} }]
}
```

| Rule | Detail |
|------|--------|
| Stack entry | `{ page, data }` — `data` is an object, never `undefined` |
| Terminal included | Finish stack is `window.wyvern.stack` plus the current page `{ page, data }` |
| Empty page data | `{}` |
| `cancel` / `dismissed` | Skip `workflow.post` and `next_wizard` (g.4) |

`workflow.pre` stdout `{ "config_patch": { … } }` merges into `config` before host bind (REQ-0124). Pre keys are **not** page `exports`. Pages that read `config.*` do not declare those keys as `requires`.

## 4. Reachability

g.9 builds the same page graph nav lint already uses:

- Entry = `wizard.json` `page.id`
- Edges = static hops in local scripts (`wizardNextDescriptor`, `wyvernWizardNext(`, `wizardNextWizard`)

A **reachable path** is a walk from the entry to the page under check. `requires` must be satisfied on **every** such path (not merely some path).

`next_wizard` is a **cross-package** edge. It does not satisfy in-wizard `requires`. See WIZARD-LINT-007.

## 5. Lint codes (reserved for g.9)

| Code | When it fires | Recovery |
|------|---------------|----------|
| **WIZARD-LINT-005** | A `requires` key is missing from `exports` of all prior pages on at least one reachable path, or export/require types conflict | Export the key on every inbound page, or drop the require |
| **WIZARD-LINT-006** | Terminal page declares `post_input` and that shape is not a subset of the terminal `exports` (plus keys the terminal itself exports on finish) | Align `collectCurrentPageData` / finish `data` with `post_input`, or fix the declaration |
| **WIZARD-LINT-007** | This package emits `wizardNextWizard` / finish `next_wizard.input` keys that the **target** wizard's `config.dataflow` does not list as `requires` or documented `config` readers | Remove unused input keys, or declare them on the target |
| **WIZARD-LINT-008** | Local JS reads `stack[i].data.<key>` or `page_data.<key>` and no reachable inbound page exports that key | Export the key or stop reading it |

### 005 algorithm (normative)

For each page P with `requires`:

1. Enumerate reachable paths entry → P.
2. For each path, collect the union of `exports` keys from pages **before** P on that path.
3. If any required key is missing on any path, emit 005 once per (page, key, missing-path suffix).
4. If a required key exists but the export type token ≠ require type (when the require lists a type in HTML meta `key:type` form), emit 005.

Pages with no `requires` are skipped.

### 006 algorithm (normative)

Applies only when `pages.<id>.terminal` is true **or** HTML has `data-wizard-terminal="true"`, **and** `post_input` is present.

Every `post_input` key must appear in that page's `exports` (finish `data` is the terminal collect blob). Missing key → 006.

`workflow.post` absent → skip 006 (nothing to contract against).

### 007 algorithm (normative)

When a hop names `next_wizard.path` (literal `{wyvern_share}/…` or repo-relative string that resolves under the lint package set):

1. Load the target `wizard.json`.
2. If the target has no `config.dataflow`, skip 007 (undeclared target).
3. Each key in `next_wizard.input` must appear in the target entry page `requires` **or** in a documented `config` reader list on the target (`config.dataflow.input_keys`, optional v1 extension). Otherwise 007.

Unresolved / dynamic paths → skip 007 (note in lint summary, not a finding).

### 008 algorithm (normative)

Static scan of local `*.js` in the package (same file set as hop extraction):

- Reads: `data.<id>`, `page_data.<id>`, `stack[…].data.<id>` with a **literal** property name.
- Computed keys (`data[name]`) are out of v1 (no finding).
- If the literal key is not in any `exports` of a page that can precede the read site on some reachable path, emit 008.

Undeclared packages (no `config.dataflow`) skip 008.

## 6. Stack-specific export rules

Cross-stack rules live in this file. Stack docs add packaging and chrome only.

### `vanilla-chrome` (`lint_profile: nav + dataflow-v1`)

Declare `exports` / `requires` per page id. Finish `data` is the object `collectCurrentPageData()` returns on the terminal page (template-picker: `template_id`, `variables`, `output_path`).

Golden package: `share/wyvern/examples/template-picker/`.

AskUserQuestion hook (g.5) finish `data.hook_config`:

```json
{
  "hook_config": {
    "global": { "enabled": true },
    "repo": { "enabled": false }
  }
}
```

Declare `exports: { "hook_config": "object" }` on the terminal page; `post_input` matches.

### `workspace-canvas` (`lint_profile: nav-limited + export-contract`)

Nav lint may be partial (custom toolbar). Dataflow focuses on the **export contract**:

Finish `data.dag` (flat under `data`, not wrapped again):

```json
{
  "layout_id": "pair",
  "nodes": [
    { "id": "node-1", "name": "planner", "role": "plan" },
    { "id": "node-2", "name": "reviewer", "role": "review" }
  ],
  "edges": [["node-1", "node-2"]]
}
```

| Key | Type | Required |
|-----|------|----------|
| `layout_id` | `string` | yes |
| `nodes` | `array` of `{ id, name, role }` | yes |
| `edges` | `array` of `[from, to]` string pairs | yes |

Declare `exports: { "dag": "object" }` on the page that assembles finish data (often `canvas` or `review`). `post_input: { "dag": "object" }` on the terminal page.

Golden package: `share/wyvern/examples/agent-dag/`.

g.9 **export-contract** extra check (workspace-canvas only): if `exports` includes `dag`, the finish assembler must mention the literal keys `layout_id`, `nodes`, and `edges` in local JS. Missing literal → 006 (reuse the post-input code; message names the missing dag field).

## 7. What lint does not do (v1)

- Runtime type checks inside the host
- Proving `collectCurrentPageData` actually writes the declared keys (tests do that)
- Evaluating computed property names
- Fetching `next_wizard` targets outside the lint path set
- Replacing `--workflow-dry-run` or integration tests

## 8. Author checklist

1. List page ids (entry + every `wizardNextDescriptor` hop).
2. For each page, list keys written in `collectCurrentPageData`.
3. For each later page, list keys read from `stack` / `page_data`.
4. Put those lists in `config.dataflow`.
5. If `workflow.post` exists, set `post_input` on the terminal page to the finish `data` keys the script reads.
6. After g.9: `wyvern wizard lint <package>` until WIZARD-LINT-005–008 are clean.

## Authority

- ADR-0006, REQ-0124, REQ-0125, REQ-0126
- [author-workflow.md](author-workflow.md) gate G4
- [vanilla-chrome.md](../stacks/vanilla-chrome.md), [workspace-canvas.md](../stacks/workspace-canvas.md)
- [g8-wizard-authoring-foundation.md](../../../../../docs/plans/phase-G/g8-wizard-authoring-foundation.md)
