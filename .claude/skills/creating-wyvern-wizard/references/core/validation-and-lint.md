# Validation and lint (schema + `wyvern wizard lint`)

Gate G1 (schema) and gate G4 (lint). Load from the Layer 0 router. Do not invent
a second schema or a second lint tool.

**Dataflow (g.9 — implemented):**

## 1. Two commands

| Gate | Command | What it checks |
|------|---------|----------------|
| G1 | `wyvern path/to/wizard.json --viewer none` | Wizard **schema** (same load path as a real run) |
| G4 | `wyvern wizard lint path/to/wizard-dir` | Nav (001–004) + dataflow (005–008 when `config.dataflow` declared) + stack export-contract |

There is no `wyvern wizard schema-validate`. Schema is the default command
loader. `--viewer none` avoids opening a window after a valid load.

Help:

```bash
wyvern wizard --help
wyvern wizard lint --help
```

## 2. G1 — schema validate

```bash
wyvern path/to/wizard.json --viewer none
```

Inline JSON uses the same validator:

```bash
wyvern '{"type":"wizard","page":{"id":"one","title":"One","html":"pages/one.html"}}'
```

Required fields: `type` is `"wizard"`; `page.id`, `page.title`, `page.html`
present. Optional `workflow.pre` / `workflow.post` must be allowlisted path
strings. Optional `width` / `height` / `config` as today. See
[platform-contract.md](platform-contract.md) §3.

CLI `ErrorCode` exits that authors will see:

| Exit | Meaning |
|------|---------|
| 0 | Schema accepted (bind / viewer-none path continues) |
| 2 | Parse / usage |
| 3 | I/O (missing file) |
| 4 | Schema validation (`validation`) |
| 9 | Workflow (not a G1 schema failure) |

`--workflow-dry-run` does **not** skip validate. G1 fails closed on schema
errors — fix `wizard.json` before HTML/JS work.

## 3. G4 — `wyvern wizard lint`

```bash
wyvern wizard lint path/to/wizard-dir
wyvern wizard lint path/to/wizard.json
wyvern wizard lint path/a path/b
```

`<path>` is a directory containing `wizard.json`, or a path to `wizard.json`.

| Exit | Meaning |
|------|---------|
| 0 | Clean (or `--help`) |
| 1 | Findings, or I/O / missing path |
| 2 | Usage (no paths, unknown subcommand) |

Fix findings in HTML / JS / `config.dataflow`. Do not disable rules to close
a sprint.

Nav lint WIZARD-LINT-001–004 shipped on `feature/phase-G-wizard-lint`
(`0578ebe`). That branch is **not** this skill's merge target; authors need a
`wyvern` that includes the subcommand (integrate after that PR, or a local
build of that branch).

## 4. Lint codes

### Nav (implemented)

| Code | When |
|------|------|
| **WIZARD-LINT-001** | Inner (non-entry) page missing Back |
| **WIZARD-LINT-002** | Terminal page missing Cancel |
| **WIZARD-LINT-003** | Chrome opt-in missing `[data-wizard-nav]` |
| **WIZARD-LINT-004** | Non-terminal chrome page missing Next |

### Dataflow (implemented — g.9)

| Code | When |
|------|------|
| **WIZARD-LINT-005** | `requires` unsatisfied or type conflict on a reachable path |
| **WIZARD-LINT-006** | Terminal `post_input` not a subset of terminal `exports` |
| **WIZARD-LINT-007** | `next_wizard.input` keys the target does not declare |
| **WIZARD-LINT-008** | Local JS reads a stack / `page_data` key nobody exports |

Algorithms: [dataflow-contracts.md](dataflow-contracts.md) §5. Omitted
`config.dataflow` → skip 005–008 (nav still runs). Wave 3 authors **must**
declare dataflow (G4).

## 5. Stack lint profiles

From `references/stacks/registry.yaml`:

| Stack | Profile | Notes |
|-------|---------|-------|
| `vanilla-chrome` | `nav + dataflow-v1` | Full 001–004; 005–008 when declared |
| `workspace-canvas` | `nav-limited + export-contract` | 002 still required on terminal; 001/003/004 only if `data-wizard-chrome` is present; `dag` export must mention `layout_id`, `nodes`, `edges` in local JS |

Known Wave 2 finding: template-picker `pages/review.html` missing Cancel
(WIZARD-LINT-002) — **fixed in g.14**; CI gate enforces clean lint.

## 6. Author loop

1. G1 — `wyvern path/to/wizard.json --viewer none` (exit 0 or 4).
2. Declare `config.dataflow` ([dataflow-contracts.md](dataflow-contracts.md) §8).
3. G4 — `wyvern wizard lint path/to/wizard-dir` (005–008 when dataflow declared).
4. **Fix every finding.** Re-run G4 until exit **0**. Do not present the wizard
   to the user with open lint findings — that is why lint exists.
5. G5 — happy-path finish JSON test; `--workflow-dry-run` when `workflow.post`
   exists (writes nothing).

## 7. Mandatory before presenting to the user

Lint is not optional polish. It is the checklist that catches missed Back/Cancel
buttons, broken dataflow, and stack export gaps.

```bash
wyvern wizard lint path/to/wizard-dir
# exit 1 → read stdout, fix HTML/JS/config.dataflow, repeat until exit 0
```

| Role | Obligation |
|------|------------|
| Skill router (`creating-wyvern-wizard`) | Block hand-off until G4 clean |
| `wyvern-wizard-js` | Run lint after `scaffold` / `author`; fix findings before success JSON |
| `wyvern-dag-wizard-js` | Same for workspace-canvas (`nav-limited` + export-contract) |

If lint fails, list rule id + file + message, fix in-tree, re-run. Never ask the
user to discover nav or dataflow gaps manually.

Page-JS disk I/O is a **skill / agent** lint (`VALIDATION.DISK_IO` in g.11),
not a `wyvern wizard lint` code. Refuse it at author time.

## Authority

- [author-workflow.md](author-workflow.md) gates G1 and G4
- [platform-contract.md](platform-contract.md)
- [dataflow-contracts.md](dataflow-contracts.md)
- [g8-wizard-authoring-foundation.md](../../../../../docs/plans/phase-G/g8-wizard-authoring-foundation.md)
- REQ-0124, REQ-0125, REQ-0126, ADR-0006
