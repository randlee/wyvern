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
| [#141](https://github.com/randlee/wyvern/issues/141) | `release/0.6.0`, `release/0.8.0` | **Closed** — k.2 wontfix for a v0.5.0 backfill; contract [`docs/release-checksums.md`](../../release-checksums.md) |
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

Authoritative breakdown: [plan-phase-k.md § Sprint sequence](plan-phase-k.md). Sprints
**k.1–k.4**; orchestration state: `.cursor/phase-k-orchestration.json`.

**Merge order:** k.1 → k.2 → k.3 → k.4 on **`integrate/phase-K`**. **`integrate/phase-K` →
`develop`** only after **phase-ending review** on k.4 (see k.4 § Phase-ending closeout).

---

## Sprint authority pointers

- **k.1:** [k1-issue-closeout-xhtml-sendto.md](k1-issue-closeout-xhtml-sendto.md) (normative AC)
- **k.4:** [k4-docli-cli-html-docs.md](k4-docli-cli-html-docs.md) (normative docli integration)

---

## Execution

Use `/cursor-orchestration` with `.cursor/phase-k-orchestration.json`. Sprint PRs target
**`integrate/phase-K`** until k.4 phase-ending closeout merges to **`develop`**.

---

## Related

- Phase J release record: [../phase-J/.plan-hardening/first-release-record.md](../phase-J/.plan-hardening/first-release-record.md)
- Phase H (XHTML): [../phase-H/README.md](../phase-H/README.md) — #115 origin
- docli repo: https://github.com/randlee/docli
