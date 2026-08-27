# Phase J — sc-publish migration (`integrate/phase-J`)

Phase J replaces Wyvern's bespoke release/publish stack with the shared
**[`sc-publish`](https://github.com/randlee/sc-publish)** kit so Rust repos
(wyvern, atm-core, …) use the same agents, workflows, manifest contract, and
per-channel retry semantics.

Implementation PRs target **`integrate/phase-J`**. Sprint docs are **sole
authority** for deliverables, acceptance criteria, and required validation.

**Planning worktree:** `plan/phase-J-sc-publish` (from `develop`).

**Prerequisite:** Phase I merged to `develop` (or current release line on
`main` @ v0.5.0+). Phase J is **tooling only** — no product dialog/host
changes unless a publish rehearsal exposes a packaging bug.

**Baseline branch:** Worktrees branch from `integrate/phase-J` (or `develop`
after planning lands).

---

## Problem

Wyvern today ships with a **one-off** publish surface:

| Area | Today (develop/main) | Target (sc-publish) |
|------|----------------------|---------------------|
| Workflows | Tag-push `release.yml`, inline Homebrew/winget jobs | `release-candidate` → preflight → `workflow_dispatch` release + per-channel retries |
| Scripts | `scripts/release_*.py` (removed on plan branch) | Vendored `.github/scripts/` (byte-for-byte from kit) |
| Manifest | Hand-edited `release/publish-artifacts.toml` | Rendered from caller-owned `release/install.json` |
| Agents | Single bespoke `publisher.md` | Kit publisher + channel playbooks (ATM + Cursor inline) |
| Winget | Inline step; `GITHUB_TOKEN`; asset `wyvern-windows.zip` | `winget-publish.yml`; **`WINGET_GITHUB_TOKEN`**; manifest asset pattern |
| Retry | `release-retry-distribution.yml` | `crates-publish.yml`, `homebrew-publish.yml`, `winget-publish.yml`, … |

This drift blocks **consistent publish ops** across Rust repos and is why v0.5.0
winget failed (no bootstrap + wrong token model).

---

## Phase goal

1. Vendor sc-publish into wyvern via `release/install.json` + `scripts/sync-sc-publish.sh`.
2. Retire bespoke publish workflows/agents without forking kit YAML.
3. Adopt the kit **release state machine** (candidate tag → `main` → dispatch).
4. Cut the **first kit-managed production release** with all declared channels
   verified (crates.io, GitHub Release assets, Homebrew, winget post-dispatch).

---

## sc-publish architecture (consumer view)

Source of truth: sibling repo `../sc-publish` (`plugins/sc-publish`).

```bash
# From wyvern repo root (any worktree):
./scripts/sync-sc-publish.sh
# → pulls ../sc-publish @ main, bootstraps sc-compose 1.5.0, runs install.py
# → copies agents/workflows/scripts verbatim; renders two manifests only
```

| Asset | Role |
|-------|------|
| `release/install.json` | **Only** hand-maintained publish contract (crates, targets, channels) |
| `release/publish-artifacts.toml` | Rendered — repo-specific artifacts & destinations |
| `release/publish-channel-contracts.toml` | Vendored unchanged — secret names, liveness checks |
| `.github/workflows/release*.yml` | Root + preflight + candidate |
| `.github/workflows/*-publish.yml` | Post-release per-channel retries |
| `.claude/agents/*-publisher.md` | ATM channel workers + coordinator |
| `.cursor/agents/publisher.md` | Cursor inline profile |

**Kit rule:** Do not hand-edit copied kit files. Fix upstream sc-publish or
change `install.json`, then re-sync.

Normative kit requirements:
[`README.sc-publish.md`](../../../README.sc-publish.md),
[`.claude/skills/publishing/ref/publish-kit-requirements.md`](../../../.claude/skills/publishing/ref/publish-kit-requirements.md).

---

## Sprint map

Sprint docs are **sole authority** for deliverables, acceptance criteria, and
required validation. This README is the phase index only.

| Sprint | Ships | Doc |
|--------|-------|-----|
| **j.1** | Vendor kit, `install.json`, sync script | [j1-vendor-sc-publish-kit.md](j1-vendor-sc-publish-kit.md) |
| **j.2** | Upstream blockers, secrets, winget/docs | [j2-upstream-blockers-and-docs.md](j2-upstream-blockers-and-docs.md) |
| **j.3** | Release rehearsal on kit state machine | [j3-release-rehearsal.md](j3-release-rehearsal.md) |
| **j.4** | Production cutover; retire tag-push | [j4-production-cutover.md](j4-production-cutover.md) |

**Merge order → `integrate/phase-J`:** j.1 → j.2 → j.3 → j.4.

---

## Winget (why sc-publish fixes it)

v0.5.0 winget failed because:

1. Inline job used repository `GITHUB_TOKEN` — cannot fork `microsoft/winget-pkgs`.
2. Asset regex targeted `wyvern-windows.zip`; kit archives rename to
   `wyvern_<version>_x86_64-pc-windows-msvc.zip`.
3. No idempotent post-release leg — retries were bundled with crates/Homebrew.

**Kit fix:**

- `winget-publish.yml` — standalone `workflow_dispatch` by tag.
- `WINGET_GITHUB_TOKEN` — PAT with rights to fork/submit to `winget-pkgs`
  (declared in `publish-channel-contracts.toml`).
- Manifest-driven `installers-regex` from `release_artifacts.py channel-config`.
- Detect-and-skip: probe existing manifest / open PR before submitting (fail closed).
- One-time bootstrap to `microsoft/winget-pkgs` still required for `randlee.wyvern`;
  after bootstrap, automated leg owns subsequent versions.

See [docs/WINGET_SETUP.md](../../../WINGET_SETUP.md) (updated in j.2).

---

## Critical review findings (Grok, planning)

Planning review **rejects** “simply copy/replace + re-publish on next tag.”
Findings below are **in scope** for Phase J sprints.

| ID | Finding | Plan response | Sprint |
|----|---------|---------------|--------|
| CR-001 | Linux webview deps missing from kit release/preflight/crates jobs | Upstream sc-publish hook **or** documented waiver + CI-only gate until landed | j.2 |
| CR-002 | `renderer_archive_path = bin/wyvern` invalid for Homebrew formula render | Upstream kit must bootstrap sc-compose on runner; do not use product binary as renderer | j.2 |
| CR-003 | Homebrew UI path may flatten `share/wyvern/ui` | Fix `install.json` `homebrew_destination_components` → `["share","wyvern","ui"]` | j.2 |
| CR-004 | Tag-push trigger vs kit `workflow_dispatch` | j.4 retires tag-push; until then keep old workflow on `develop` if emergency ship needed | j.4 |
| CR-005 | Archive rename breaks README/tap/docs | Update consumer docs in j.2; call out breaking URL change in j.4 release notes | j.2, j.4 |
| CR-006 | `WINGET_GITHUB_TOKEN` required | Provision secret; update `docs/RELEASE_SECRETS.md` | j.2 |
| CR-007 | Wyvern-only gates stay in `ci.yml` | Explicit boundary — kit preflight does not replace ui-sync/boundaries | j.1 |
| CR-008 | `scripts/validate_release.py` half-migrated | Delete or rewrite against kit CLI; not wired into kit workflows | j.2 |
| CR-009 | `sync-sc-publish.sh` mutates sibling repo checkout | Document; optional pin to SHA instead of `main` | j.1 |

**Hardening gate:** Each sprint doc passes plan-scope review and critical-plan
review (Grok) before implementation merge.

---

## Phase acceptance criteria

1. j.1–j.4 sprint acceptance criteria pass on `integrate/phase-J` (see sprint docs).
2. Plan hardening: `plan-scope-reviewer` + `critical-plan-reviewer` PASS; `quality-mgr` plan QA PASS.
3. `./scripts/sync-sc-publish.sh` dry-run exit **0** on final branch.
4. First kit-managed production release verified per j.4.

---

## Boundaries (Wyvern-owned, not kit)

| Stays in wyvern | Reason |
|-----------------|--------|
| `.github/workflows/ci.yml` | Product gates: boundaries, ui-sync, share-sync, audit/deny |
| `scripts/check-*.sh`, `check-boundaries.py` | Product integrity |
| `ui/`, crates, `third_party/wayland-scanner` | Product |
| `release/install.json` | Consumer contract |
| `scripts/sync-sc-publish.sh` | Re-vendor entrypoint |

Do **not** re-implement product gates inside kit YAML forks.

---

## Phase integration smoke (non-normative)

After j.4 on `integrate/phase-J`:

```bash
# Manifest
python3 .github/scripts/release_artifacts.py validate-manifest \
  --manifest release/publish-artifacts.toml --workspace-toml Cargo.toml

# Kit drift
./scripts/sync-sc-publish.sh   # ends with dry-run exit 0

# Channels (after production tag vX.Y.Z)
curl -fsS -A wyvern-check "https://crates.io/api/v1/crates/wyvern-cli/X.Y.Z"
gh release view vX.Y.Z --json assets
curl -fsS "https://raw.githubusercontent.com/randlee/homebrew-tap/main/Formula/wyvern.rb" | rg version
gh workflow run winget-publish.yml -f tag=vX.Y.Z   # when token provisioned
```

---

## Related

- [`release/install.json`](../../../release/install.json) — Wyvern publish contract
- [`scripts/sync-sc-publish.sh`](../../../scripts/sync-sc-publish.sh) — vendor entrypoint
- [docs/RELEASE_SECRETS.md](../../../docs/RELEASE_SECRETS.md) — updated j.2
- [docs/WINGET_SETUP.md](../../../WINGET_SETUP.md) — updated j.2
