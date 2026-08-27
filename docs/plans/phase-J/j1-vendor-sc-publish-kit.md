---
id: j.1
title: Vendor sc-publish kit
status: planning
branch: feature/phase-J-j1-vendor-sc-publish
worktree: ../wyvern-worktrees/feature/phase-J-j1-vendor-sc-publish
target: integrate/phase-J
---

# Sprint j.1 — Vendor sc-publish kit

## Goal

Vendor the shared [`sc-publish`](https://github.com/randlee/sc-publish) kit into
wyvern: workflows, scripts, agents, rendered manifests, and a repeatable sync
entrypoint. Remove legacy bespoke publish files.

## Hard dependencies

- `develop` @ v0.5.0+ (current release line)
- Sibling repo `../sc-publish` @ `main` (see `scripts/sync-sc-publish.sh`)

## Deliverables

| Path | Purpose |
|------|---------|
| `release/install.json` | Caller-owned publish contract (crates, targets, channels) |
| `scripts/sync-sc-publish.sh` | Pull `../sc-publish`, bootstrap sc-compose, run `install.py` |
| `.github/workflows/release.yml` | Kit root release (vendored) |
| `.github/workflows/release-preflight.yml` | Kit preflight (vendored) |
| `.github/workflows/release-candidate.yml` | Kit candidate tag (vendored) |
| `.github/workflows/crates-publish.yml` | Per-channel crates retry (vendored) |
| `.github/workflows/homebrew-publish.yml` | Per-channel Homebrew retry (vendored) |
| `.github/workflows/winget-publish.yml` | Per-channel winget retry (vendored) |
| `.github/scripts/release_artifacts.py` | Manifest CLI (vendored) |
| `.github/scripts/release_gate.sh` | Release gate (vendored) |
| `.claude/agents/publisher.md` + channel publishers | ATM publish profile (vendored) |
| `.cursor/agents/publisher.md` + publish skill/command | Cursor inline profile (vendored) |
| `release/publish-artifacts.toml` | Rendered from `install.json` |
| `release/publish-channel-contracts.toml` | Rendered from `install.json` (channel-filtered) |
| `README.sc-publish.md` | Kit README (does not replace Wyvern `README.md`) |
| `.gitignore` | Entry `.sc-publish-venv/` |

## Paths to delete

| Path | Reason |
|------|--------|
| `scripts/release_artifacts.py` | Replaced by `.github/scripts/` |
| `scripts/release_gate.sh` | Replaced by `.github/scripts/` |
| `.github/workflows/release-retry-distribution.yml` | Replaced by per-channel workflows |

## Consumer contract (normative sample)

`release/install.json` minimum shape (Wyvern values):

```json
{
  "schema_version": 1,
  "project": {
    "name": "wyvern",
    "archive_prefix": "wyvern",
    "readme_dependency_crate": "wyvern-schema",
    "renderer_archive_path": "bin/wyvern",
    "workspace_toml": "Cargo.toml",
    "rust_toolchain": "stable"
  },
  "channels": {
    "homebrew": { "tap_repository": "randlee/homebrew-tap", "...": "..." },
    "winget": { "identifier": "randlee.wyvern", "installer_target": "x86_64-pc-windows-msvc" }
  }
}
```

Full file is authoritative at `release/install.json`.

## Acceptance criteria

1. `./scripts/sync-sc-publish.sh` uses pinned `SC_PUBLISH_REF` (default `6aace27`), installs with `--input release/install.json`, dry-run exit **0**.
2. `release/install.json` declares five publishable crates (orders 1–5), `wyvern-mcp` unpublished, four release targets, binaries `wyvern` + `wyvern-viewer`, bundled `ui/`, channels **homebrew** + **winget** only.
3. Every deliverable path in the table above exists after sync (kit byte-for-byte copies).
4. Every path in **Paths to delete** is absent from the branch.
5. `python3 .github/scripts/release_artifacts.py validate-manifest --manifest release/publish-artifacts.toml --workspace-toml Cargo.toml` exits **0**.
6. `.github/workflows/ci.yml` has **zero diff** from pre-j.1 baseline (product gates unchanged).

## Non-closure (explicit)

- **j.1 removes tag-push release** when merged to `integrate/phase-J` — kit `release.yml` replaces legacy trigger (see [publish-architecture-decision.md](publish-architecture-decision.md)).
- **j.1 does not** resolve CR-001/CR-002 or provision secrets — **j.2**.
- **j.1 does not** cut a release — **j.3**.

## Required validation

```bash
./scripts/sync-sc-publish.sh
python3 .github/scripts/release_artifacts.py validate-manifest \
  --manifest release/publish-artifacts.toml --workspace-toml Cargo.toml
! git grep -n 'scripts/release_artifacts.py' -- ':!docs/' ':!.github/scripts/tests/'
test ! -f scripts/release_gate.sh
test ! -f .github/workflows/release-retry-distribution.yml
```
