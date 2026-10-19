# Phase K — Distribution hygiene, issue closeout & CLI docs (`integrate/phase-K`)

Phase K clears the **open GitHub issue backlog** (as of 2026-10-19), adopts
**[docli](https://github.com/randlee/docli)** for generated CLI HTML documentation,
and leaves no stale “fixed in product but open on GitHub” tickets.

Implementation PRs target **`integrate/phase-K`**. **Sole planning authority:**
[plan-phase-k.md](plan-phase-k.md). Sprint docs are sole authority for deliverables
and acceptance criteria; this README is the phase index.

**Prerequisite:** Phase J cut complete on `main` (v0.7.0+ kit releases). Phase K is
mostly **docs, packaging, CI evidence, and closeout** — minimal dialog/host behavior
unless a closeout test exposes a regression.

**Baseline branch:** Worktrees from `develop` → `integrate/phase-K`.

---

## GitHub labels (release tracking)

| Label | Meaning |
|-------|---------|
| `release/0.8.0` | Planned for **v0.8.0** (Phase K) |
| `release/0.7.0` | Shipped in **v0.7.0** |
| `release/0.6.0` | Shipped in **v0.6.0** |
| `release/0.4.0` | Shipped in **v0.4.0** |

## Issue inventory (2026-10-19)

| Issue | Label(s) | Phase K disposition |
|-------|----------|---------------------|
| [#115](https://github.com/randlee/wyvern/issues/115) | `release/0.4.0` | **Closed** — PR [#119](https://github.com/randlee/wyvern/pull/119), commit `02454bb` |
| [#139](https://github.com/randlee/wyvern/issues/139) | `release/0.7.0` | **Closed** — PR [#140](https://github.com/randlee/wyvern/pull/140), commits `c6a51f0` / `cf25aec` |
| [#141](https://github.com/randlee/wyvern/issues/141) | `release/0.6.0`, `release/0.8.0` | **Open** — forward fix in v0.6.0+ (PR [#147](https://github.com/randlee/wyvern/pull/147)); k.2 closeout |
| [#62](https://github.com/randlee/wyvern/issues/62) | `release/0.8.0` | **Open** — k.3 |
| [#63](https://github.com/randlee/wyvern/issues/63) | `release/0.8.0` | **Open** — k.3 |

**Not in GitHub issues but in scope:** **docli** — not integrated in wyvern today (no
`docli` / `cargo docli` references in repo). Add generated CLI reference HTML and
wire into release/docs surface.

---

## Problem

1. **Stale issues** — Several requests landed in v0.7.0 (XHTML suffix, Send-To CI,
   kit `checksums.txt`) but issues stayed open; operators cannot trust the issue list.
2. **crates.io presentation** — README and documentation links still broken per #62/#63.
3. **CLI documentation** — Help text exists (`wyvern help`, extension catalog), but there
   is no **docli-generated HTML** bundle for offline/browser reference aligned with other
   Rust CLIs in the org.

---

## Phase goal

1. Close or resolve all five open issues with linked PR/CI evidence.
2. Integrate **docli** for maintainable CLI HTML docs (build in CI, ship or publish path
   documented).
3. Patch release **0.7.1** (or next patch) only if crates.io manifest fixes require a
   republish; otherwise ship fixes on the next scheduled minor.

---

## Sprint map

Authoritative breakdown: [plan-phase-k.md § Sprint sequence](plan-phase-k.md). Sprint doc
links: k.1–k.5 markdown files in this directory (see orchestration JSON).

**Merge order:** k.1 → k.2 → k.3 → k.4 on **`integrate/phase-K`**; k.5 → **`develop`**.

---

## Sprint authority pointers

- **k.1:** [k1-issue-closeout-xhtml-sendto.md](k1-issue-closeout-xhtml-sendto.md) (normative AC)
- **k.4:** [k4-docli-cli-html-docs.md](k4-docli-cli-html-docs.md) (normative docli integration)

---

## Cursor orchestration (`/cursor-orchestration`)

Phase K execution uses the same dev→QA loop as other Wyvern phases. State file: `.cursor/phase-k-orchestration.json` (repo root).

| Step | Action |
|------|--------|
| Bootstrap | Create `integrate/phase-K` from `develop` if absent; create sprint worktree via `/sc-git-worktree` |
| Dev | Render `dev-template.xml.j2` with `sprint_doc` = active k.* markdown; Task **`rust-developer`** |
| PR | Target **`integrate/phase-K`** (not `develop` directly) |
| Pre-QA-1 | Optional **`rust-best-practices-agent`** on planned `review_targets` |
| QA | Parent spawns reviewers → **`cursor-quality-mgr`** enforcer (never ATM `quality-mgr`) |
| Gates | `reviewer_spawn_gate: pass`, 0B+0I+0m findings, CI green, evidence chain on PR |
| Next | Merge sprint PR; advance `active_sprint` in orchestration JSON; repeat |

**Phase-ending QA:** Run on k.4 PR with full reviewer set; prefer Terra on `rust-qa-agent`
when available (see `.cursor/orchestration-agent-models.yaml`).

**Plan review (this document):** Before k.1 dev, optional plan QA with `review_mode: plan`
on `docs/plans/phase-k/` — parent spawns req-qa + arch-qa (+ plan-scope if used).

---

## Related

- Phase J release record: [../phase-J/.plan-hardening/first-release-record.md](../phase-J/.plan-hardening/first-release-record.md)
- Phase H (XHTML): [../phase-H/README.md](../phase-H/README.md) — #115 origin
- docli repo: https://github.com/randlee/docli
