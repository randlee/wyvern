# Phase H — XHTML reporting & review (`integrate/phase-H`)

Phase H adds **non-wizard** viewing surfaces for sc-compose XHTML panels: single
panel, panel arrays, and optional **`--review`** (comments + Approve/Cancel).
Implementation PRs target **`integrate/phase-H`**. **Sprint docs are sole
authority** for deliverables, acceptance criteria, and required validation.

**Prerequisite:** Phase G complete on `integrate/phase-G` (extension help, skill
catalog, wizard lint, authoring skill — [#117](https://github.com/randlee/wyvern/pull/117)
→ `develop`). Phase H does **not** require Phase E.

**Baseline branch:** Implementation worktrees branch from `integrate/phase-G` (or
`develop` after #117 merges).

**Related:** [GitHub #115](https://github.com/randlee/wyvern/issues/115) (`.xhtml`
suffix) — closed by **h.1**.

---

## Problem

Agents and operators need **ad-hoc** XHTML review outside the normal “assemble
many panels → publish one HTML report” pipeline (atm-core benchmark/fuzz flow).

Example: five generated panels, three fail → agent writes `proposed-fix.xhtml`
→ user reviews `[fail-1, fail-2, fail-3, proposed-fix]` → structured JSON
(`approved`, `comments`) drives the next agent step.

Today:

- `.html` suffix works; `.xhtml` does not (#115).
- No multi-panel stitcher in wyvern.
- No first-class **review** finish contract.
- No **`wyvern-reporting`** skill for panel authoring + review manifests.

---

## Core model (not wizard)

Phase H introduces a **`report`** command — single static HTML/XHTML page served
from `--ui-root`, **no wizard stack**, **no** `wizard-nav.js`, **no**
`/api/wizard/*`.

```
argv → extension match → preexec (optional frame stitch) → expand type: report
     → validate → host /report/* static + optional /api/report/finish
     → CommandResult JSON
```

**Host hardening inheritance:** report sessions inherit `session_timeout`,
`REQUEST_TIMEOUT` (310s), and preexec script timeout from `wyvern-host` /
`extensions/preexec.rs` (same as Phase F compose/CSV extensions). See h.5 CI notes.

| Surface | CLI | Frame |
|---------|-----|-------|
| Single panel | `wyvern panel.xhtml` | Basic document shell (charset, viewport) |
| Panel array | `wyvern report-xhtml review.json` | Basic shell + labeled `<section class="pane">` per panel |
| Review mode | `wyvern report-xhtml --review review.json` | Review shell + comment textarea + Cancel / Approve |

Authoring guidance lives in **`.claude/skills/wyvern-reporting/`** (sc-compose
panel fragments, manifest schema, review UX) — **not** `creating-wyvern-wizard`.

Contract: [xhtml-reporting-contract.md](xhtml-reporting-contract.md).

---

## Sprint map

| Sprint | Ships | Doc |
|--------|-------|-----|
| **h.1** | `report` host + `xhtml-suffix` + basic single-panel frame | [h1-xhtml-single-panel.md](h1-xhtml-single-panel.md) |
| **h.2** | `report-xhtml` extension + panel-array basic frame | [h2-xhtml-panel-array.md](h2-xhtml-panel-array.md) |
| **h.3** | `--review` frame + finish JSON contract | [h3-xhtml-review-mode.md](h3-xhtml-review-mode.md) |
| **h.4** | `wyvern-reporting` skill + reference docs | [h4-wyvern-reporting-skill.md](h4-wyvern-reporting-skill.md) |
| **h.5** | Synthetic example package + CI smoke | [h5-synthetic-xhtml-example.md](h5-synthetic-xhtml-example.md) |

**Merge order → `integrate/phase-H`:** h.1 → h.2 → h.3 → h.4 → h.5  
(h.4 may start after h.2 doc contract is stable; must merge after h.3 so refs
include `--review` finish shape.)

---

## Phase integration smoke (non-normative)

Sprint docs remain **sole authority** for acceptance criteria and required validation.
This checklist mirrors h.1–h.5 smoke paths for phase closeout only.

1. `WYVERN_VIEWER=none wyvern share/wyvern/examples/xhtml-review/panels/fail-1.xhtml` exits 0 (h.1).
2. `WYVERN_VIEWER=none wyvern report-xhtml share/wyvern/examples/xhtml-review/review-view.json` exits 0 (h.2).
3. `wyvern report-xhtml --review share/wyvern/examples/xhtml-review/review-review.json` returns finish JSON (h.3).
4. Extension registry includes `xhtml-suffix`, `report-xhtml`, and `report-xhtml-review`.
5. Per-sprint tests pass (run separately — cargo does not accept multiple `--test` filters):

```bash
cargo test -p wyvern-cli --test extensions_xhtml_single
cargo test -p wyvern-cli --test extensions_xhtml_array
cargo test -p wyvern-cli --test extensions_xhtml_review
cargo test -p wyvern-host report_
```

Optional operator walkthrough (copy/paste):

```bash
WYVERN_VIEWER=none wyvern share/wyvern/examples/xhtml-review/panels/fail-1.xhtml
WYVERN_VIEWER=none wyvern report-xhtml share/wyvern/examples/xhtml-review/review-view.json
wyvern report-xhtml --review share/wyvern/examples/xhtml-review/review-review.json
wyvern extensions list --json | jq -e '.[] | select(.id=="xhtml-suffix" or .id=="report-xhtml" or .id=="report-xhtml-review")'
```

---

## Boundaries

- **No new wizard** pages, `wizard-nav.js`, or `creating-wyvern-wizard` changes
  for report surfaces (skill lives in `wyvern-reporting`).
- **No** replacement of atm-core “publish aggregate HTML report” pipeline — Phase
  H is **non-standard viewing** only.
- **`wyvern-schema`:** adds `type: "report"` per **ADR-0025**; no opaque page-data graph (ADR-0006 wizard dataflow does not apply).
- Host changes in **`wyvern-host`** + CLI in **`wyvern`**; preexec scripts in
  **`scripts/ext/`**.

## Non-closure

- Paginated “one panel per step” UX — post-H (`--mode pages` follow-up)
- MCP / `--interactive` wrappers — **Phase E**
- Auto-discovery of all `*.xhtml` in a directory without manifest — post-H
- Full sc-compose runtime in wyvern (authors use `sc-compose` externally; wyvern
  consumes rendered `.xhtml` files)

## Authority

- [xhtml-reporting-contract.md](xhtml-reporting-contract.md)
- [cli-extensions-contract.md](../phase-F/cli-extensions-contract.md)
- [http-wizard-contract.md](../phase-C/http-wizard-contract.md) (contrast only —
  report is a separate host route family)
