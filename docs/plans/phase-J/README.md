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
| Retry | `release-retry-distribution.yml` | `crates-publish.yml`, `homebrew-publish.yml`, `scoop-publish.yml`, `winget-publish.yml`, … |

This drift blocks **consistent publish ops** across Rust repos and is why v0.5.0
winget failed (no bootstrap + wrong token model).

---

## Phase goal

1. Vendor sc-publish into wyvern via `release/install.json` + `scripts/sync-sc-publish.sh`.
2. Retire bespoke publish workflows/agents without forking kit YAML.
3. Adopt the kit **release state machine** (candidate tag → `main` → dispatch).
4. Cut the **first kit-managed production release** with all declared channels
   verified (crates.io, GitHub Release assets, Homebrew, Scoop, winget post-dispatch).

**Supported distribution channels (Wyvern):** crates.io, GitHub Releases, Homebrew,
Scoop, winget. **PyPI is not declared** — wyvern has no Python bindings yet; kit
PyPI legs remain vendored for upstream parity but are **inactive** when
`install.json` omits `channels.pypi` (no `[channels.pypi]` in
`publish-artifacts.toml`; preflight skips PyPI secrets/environments).

---

## sc-publish architecture (consumer view)

Source of truth: sibling repo `../sc-publish` (`plugins/sc-publish`).

```bash
# From wyvern repo root (any worktree):
./scripts/sync-sc-publish.sh
# → pulls ../sc-publish @ pinned SHA (default 6aace27), bootstraps sc-compose, runs install.py
# → copies agents/workflows/scripts verbatim; renders two manifests only
```

| Asset | Role |
|-------|------|
| `release/install.json` | **Only** hand-maintained publish contract (crates, targets, channels) |
| `release/publish-artifacts.toml` | Rendered from `install.json` — **wyvern channel set** (no PyPI section when omitted) |
| `release/publish-channel-contracts.toml` | Vendored kit protocol (full channel table incl. PyPI reference); preflight filters checks via `publish-artifacts.toml` |
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
| **j.2** | Upstream blockers, secrets, Scoop/winget docs | [j2-upstream-blockers-and-docs.md](j2-upstream-blockers-and-docs.md) |
| **j.3** | First kit-managed release (`vX.Y.Z`) | [j3-first-kit-release.md](j3-first-kit-release.md) |
| **j.4** | Merge `integrate/phase-J` → `develop` | [j4-production-cutover.md](j4-production-cutover.md) |

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
| CR-001 | Linux webview deps missing from kit jobs | **Must resolve** upstream before j.3; waiver blocks phase | j.2 |
| CR-002 | Homebrew/Scoop renderer uses product binary | **Must resolve** upstream for **both** `homebrew-publish.yml` and `scoop-publish.yml`; keep valid `renderer_archive_path` in `install.json` | j.2 |
| CR-003 | Homebrew UI path flattening | `homebrew_destination_components` → `["share","wyvern","ui"]` | j.2 |
| CR-004 | Tag-push vs kit dispatch | Tag-push removed when j.1 lands on `integrate/phase-J` | j.1, [ADR](publish-architecture-decision.md) |
| CR-005 | Archive rename | Update README in j.2; release notes in j.3 | j.2, j.3 |
| CR-006 | `WINGET_GITHUB_TOKEN` required | Provision in j.2; bootstrap before j.3 if needed | j.2 |
| CR-010 | `SCOOP_BUCKET_TOKEN` required | Provision in j.2; bucket bootstrap before j.3 if needed | j.2 |
| CR-007 | Wyvern gates stay in `ci.yml` | j.1 zero-diff on `ci.yml` | j.1 |
| CR-008 | Legacy `validate_release.py` | Delete in j.2 | j.2 |
| CR-009 | Moving sc-publish head | Pin `SC_PUBLISH_REF` in sync script | j.1 |

**Hardening gate:** Each sprint doc passes plan-scope review and critical-plan
review (Grok) before implementation merge.

---

## Phase acceptance criteria

1. j.1–j.4 sprint acceptance criteria pass on `integrate/phase-J` (see sprint docs).
2. Plan hardening: `plan-scope-reviewer` + `critical-plan-reviewer` PASS; `quality-mgr` plan QA PASS.
3. `./scripts/sync-sc-publish.sh` dry-run exit **0** on final branch.
4. First kit-managed production release verified per **j.3**; `develop` integration per **j.4**.

Architecture: [publish-architecture-decision.md](publish-architecture-decision.md).

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
curl -fsS "https://raw.githubusercontent.com/randlee/scoop-bucket/main/bucket/wyvern.json" | rg version
gh workflow run winget-publish.yml -f tag=vX.Y.Z --ref main
gh workflow run scoop-publish.yml -f tag=vX.Y.Z --ref main
```

---

## Related

- [`release/install.json`](../../../release/install.json) — Wyvern publish contract
- [`scripts/sync-sc-publish.sh`](../../../scripts/sync-sc-publish.sh) — vendor entrypoint
- [docs/RELEASE_SECRETS.md](../../../docs/RELEASE_SECRETS.md) — updated j.2
- [docs/WINGET_SETUP.md](../../../WINGET_SETUP.md) — updated j.2
- [docs/SCOOP_SETUP.md](../../../docs/SCOOP_SETUP.md) — updated j.2
