# Phase K plan — Distribution hygiene & CLI docs (`integrate/phase-K`)

**Status:** hardened  
**Target release:** **v0.8.0** (patch **v0.7.1** optional if k.3 requires crates.io republish)  
**Integration branch:** `integrate/phase-K`  
**Prerequisite:** `main` / `develop` @ **v0.7.0+** (Phase J kit complete)

---

## Phase goal

1. Close **#141** with evidence; ship **#62** / **#63** fixes in k.3 and close those issues
   only after **v0.7.1** `crates-publish.yml` shows README + Documentation on crates.io.
2. Keep regression coverage for already-shipped work (#115 → v0.4.0, #139 → v0.7.0).
3. Adopt **[docli](https://github.com/randlee/docli)** (`sc-docli` on crates.io) to generate
   **`site/cli/`** from `usage_message()` + extension registry via **`gen-cli-model`** (no clap).
4. After **phase-ending review** on k.4, merge **`integrate/phase-K` → `develop`**.

Issue inventory and background: [README.md § Issue inventory](README.md#issue-inventory-2026-10-19).

---

## Sprint sequence

| Sprint | Branch | Doc | Ships |
|--------|--------|-----|-------|
| k.1 | `feature/phase-K-k1-issue-closeout` | [k1-issue-closeout-xhtml-sendto.md](k1-issue-closeout-xhtml-sendto.md) | Regression tests + evidence doc |
| k.2 | `feature/phase-K-k2-checksums` | [k2-checksums-release-contract.md](k2-checksums-release-contract.md) | Checksum contract docs/tests; close #141 |
| k.3 | `feature/phase-K-k3-crates-io-metadata` | [k3-crates-io-readme-docs-rs.md](k3-crates-io-readme-docs-rs.md) | crates.io manifest fixes; defer #62/#63 close until post-publish |
| k.4 | `feature/phase-K-k4-docli` | [k4-docli-cli-html-docs.md](k4-docli-cli-html-docs.md) | docli → `site/cli/` + CI drift; phase-ending closeout |

**Merge order:** k.1 → k.2 → k.3 → k.4. Sprint PRs target **`integrate/phase-K`**.

**Integration merge:** No separate sprint. After k.4 lands and **phase-ending QA PASS**,
open **`integrate/phase-K` → `develop`** (coordinator). Closeout deliverables live in
[k4-docli-cli-html-docs.md § Phase-ending closeout](k4-docli-cli-html-docs.md#phase-ending-closeout).

---

## docli integration (summary)

**Sole normative authority:** [k4-docli-cli-html-docs.md](k4-docli-cli-html-docs.md).
Uses **`sc-docli` 0.2.0** (crates.io) as an installed binary → output **`site/cli/`**.

---

## Phase acceptance criteria (sole authority)

1. **#141** closed on GitHub with PR/evidence links; **#62** and **#63** closed only after
   successful **v0.7.1** crates.io publish verification (regression issues #115/#139 already closed).
2. Latest GitHub Release **`checksums.txt`** covers every kit binary archive declared in
   `release/install.json` / `publish-artifacts.toml` (k.2 documents contract).
3. Post-**v0.7.1** (or chosen patch) publish: crates.io pages for **`wyvern-cli`** and
   **`wyvern-schema`** show non-empty README and **Documentation** link (spot-check).
4. `fixtures/cli/wyvern-cli-model.json` + **`site/cli/`** exist; CI enforces docli drift.
5. `cargo test --workspace` and full CI green on `integrate/phase-K`.
6. **`integrate/phase-K` → `develop`** merged after phase-ending review; `docs/release/version-policy.md`
   records version decisions (0.7.1 patch + 0.8.0 minor as applicable).

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
