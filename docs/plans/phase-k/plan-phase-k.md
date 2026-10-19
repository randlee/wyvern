# Phase K plan — Distribution hygiene & CLI docs (`integrate/phase-K`)

**Status:** planning (hardening in progress)  
**Target release:** **v0.8.0** (patch **v0.7.1** optional if k.3 requires crates.io republish)  
**Integration branch:** `integrate/phase-K`  
**Prerequisite:** `main` / `develop` @ **v0.7.0+** (Phase J kit complete)

---

## Phase goal

1. Close remaining GitHub hygiene issues **#141**, **#62**, **#63** with evidence.
2. Keep regression coverage for already-shipped work (#115 → v0.4.0, #139 → v0.7.0).
3. Adopt **[docli](https://github.com/randlee/docli)** (`sc-docli` on crates.io) to generate
   **`site/cli/`** from `usage_message()` + extension registry via **`gen-cli-model`** (no clap).
4. Merge **`integrate/phase-K` → `develop`** when all sprints PASS QA + CI.

---

## Problem

| Gap | Impact |
|-----|--------|
| Open issues despite shipped fixes | Operators cannot trust the issue backlog |
| crates.io README / `documentation` metadata broken (#62, #63) | Poor discoverability on crates.io |
| #141 forward fix in v0.6.0+ but issue open | atm-core / bootstrap consumers unclear on contract |
| No docli pipeline | CLI docs drift from `wyvern help`; no `site/cli/` parity with docli site |

---

## GitHub issue map (labels)

| Issue | Label(s) | Sprint |
|-------|----------|--------|
| #115 | `release/0.4.0` | **Closed** — k.1 regression only |
| #139 | `release/0.7.0` | **Closed** — k.1 evidence doc |
| #141 | `release/0.6.0`, `release/0.8.0` | **k.2** closeout |
| #62, #63 | `release/0.8.0` | **k.3** |

---

## Sprint sequence

| Sprint | Branch | Doc | Ships |
|--------|--------|-----|-------|
| k.1 | `feature/phase-K-k1-issue-closeout` | [k1-issue-closeout-xhtml-sendto.md](k1-issue-closeout-xhtml-sendto.md) | Regression tests + evidence doc |
| k.2 | `feature/phase-K-k2-checksums` | [k2-checksums-release-contract.md](k2-checksums-release-contract.md) | Checksum contract docs/tests; close #141 |
| k.3 | `feature/phase-K-k3-crates-io-metadata` | [k3-crates-io-readme-docs-rs.md](k3-crates-io-readme-docs-rs.md) | crates.io manifest fixes; close #62, #63 |
| k.4 | `feature/phase-K-k4-docli` | [k4-docli-cli-html-docs.md](k4-docli-cli-html-docs.md) | docli → `site/cli/` + CI drift gate |
| k.5 | `feature/phase-K-k5-merge-develop` | [k5-merge-develop.md](k5-merge-develop.md) | Merge integration branch → `develop` |

**Merge order:** k.1 → k.2 → k.3 → k.4 → k.5. Each PR targets **`integrate/phase-K`**.

---

## docli integration (summary)

**Sole normative authority:** [k4-docli-cli-html-docs.md](k4-docli-cli-html-docs.md).
Uses **`sc-docli` 0.2.0** (crates.io) as an installed binary → output **`site/cli/`**.

---

## Phase acceptance criteria (sole authority)

1. **#141**, **#62**, **#63** closed on GitHub with PR/evidence links (regression issues
   #115/#139 already closed).
2. Latest GitHub Release **`checksums.txt`** covers every kit binary archive declared in
   `release/install.json` / `publish-artifacts.toml` (k.2 documents contract).
3. Post-**v0.7.1** (or chosen patch) publish: crates.io pages for **`wyvern-cli`** and
   **`wyvern-schema`** show non-empty README and **Documentation** link (spot-check).
4. `fixtures/cli/wyvern-cli-model.json` + **`site/cli/`** exist; CI enforces docli drift.
5. `cargo test --workspace` and full CI green on `integrate/phase-K`.
6. **`integrate/phase-K` → `develop`** merged; `docs/release/version-policy.md` records
   version decisions (0.7.1 patch + 0.8.0 minor as applicable).
7. Plan hardened: plan-scope + critical-plan + plan QA **PASS** under
   [`.plan-hardening/`](.plan-hardening/).

---

## Non-goals (Phase K)

- New dialog types or host protocol changes (unless regression fix).
- winget-pkgs #425477 (0.5.0 bootstrap) — out of scope.
- docli MCP wrapper or NuGet leg.

---

## References

- [README.md](README.md) — phase index
- [docs/plans/project-plan.md](../project-plan.md) — dependency map
- [docs/requirements.md](../../requirements.md), [docs/architecture.md](../../architecture.md)
- docli: `https://github.com/randlee/docli` (develop @ sc-docli 0.2.0)
