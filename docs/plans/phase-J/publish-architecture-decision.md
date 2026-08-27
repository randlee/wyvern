# Phase J — Publish architecture decision

**Status:** Accepted (planning)  
**Phase:** J — sc-publish migration

## Context

Wyvern moves from bespoke tag-push release workflows to the shared
[`sc-publish`](https://github.com/randlee/sc-publish) kit for consistency with
other Rust repos.

## Decision

1. **Kit source:** Vendored byte-for-byte from `../sc-publish` at a **pinned SHA**
   recorded in `scripts/sync-sc-publish.sh`. Local edits to copied kit files are
   forbidden; changes go to upstream sc-publish or `release/install.json`.
2. **Consumer contract:** `release/install.json` is the only wyvern-owned publish
   input; `install.py` renders `release/publish-artifacts.toml` and
   `release/publish-channel-contracts.toml` (channel-filtered).
3. **Production tags:** Immutable `vX.Y.Z` tags are created **only** by kit
   `release.yml` `workflow_dispatch` on **`main`**. No local `git tag` / `git push
   --tags` for releases.
4. **Trigger cutover:** Tag-push `on.push.tags: v*` is removed when kit workflows
   land on **`integrate/phase-J`** (j.1). There is no parallel emergency tag-push
   workflow after j.1 merges.
5. **Winget credential:** `WINGET_GITHUB_TOKEN` (PAT with fork/PR rights to
   `microsoft/winget-pkgs`) is **required**. Repository `GITHUB_TOKEN` is **not**
   used for winget submit.
6. **Product gates:** `ci.yml` retains wyvern-only checks (boundaries, ui-sync,
   share-sync). Kit preflight does not replace them.
7. **Distribution channels:** Wyvern publishes to crates.io, GitHub Releases,
   Homebrew, Scoop, and winget. **PyPI is excluded** until Python bindings exist;
   `install.json` omits `channels.pypi` and leaves Python package lists empty so
   rendered manifests and preflight skip PyPI credential checks.

## Consequences

- First kit release (j.3) uses a **real semver** and production channels; there is
  no separate “safe rehearsal” semver in the current kit.
- CR-001 (Linux webview deps) and CR-002 (Homebrew renderer) must be **resolved**
  before j.3; waivers block the phase.

## References

- [docs/plans/phase-J/README.md](plans/phase-J/README.md)
- [README.sc-publish.md](../../README.sc-publish.md)
