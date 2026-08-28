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
   input; `install.py` renders `release/publish-artifacts.toml` (wyvern channel
   set). `release/publish-channel-contracts.toml` is the **full kit protocol**
   (PyPI table may remain); preflight secret/environment checks follow channels
   declared in `publish-artifacts.toml`, not omission from the contracts file.
3. **Production tags:** Immutable `vX.Y.Z` tags are created **only** by kit
   `release.yml` `workflow_dispatch` on **`main`**. No local `git tag` / `git push
   --tags` for releases.
4. **Trigger cutover:** Tag-push `on.push.tags: v*` is removed when kit workflows
   land on **`integrate/phase-J`** (j.1). There is no parallel emergency tag-push
   workflow after j.1 merges.
5. **Winget credential:** `WINGET_GITHUB_TOKEN` (PAT with fork/PR rights to
   `microsoft/winget-pkgs`) is **required**. Repository `GITHUB_TOKEN` is **not**
   used for winget submit.
6. **Scoop credential:** `SCOOP_BUCKET_TOKEN` (PAT with `contents:write` on
   `randlee/scoop-bucket`) is **required**. Repository `GITHUB_TOKEN` is **not**
   used to push Scoop manifest updates.
7. **Product gates:** `ci.yml` retains wyvern-only checks (boundaries, ui-sync,
   share-sync). Kit preflight does not replace them.
8. **Distribution channels:** Wyvern publishes to crates.io, GitHub Releases,
   Homebrew, Scoop, and winget. **PyPI is excluded** until Python bindings exist;
   `install.json` omits `channels.pypi` so `publish-artifacts.toml` has no
   `[channels.pypi]` and preflight skips PyPI credential checks (contracts file
   may still document PyPI for kit parity).
9. **Shared destinations:** All kit repos use the same org publish targets —
   `randlee/homebrew-tap`, `randlee/scoop-bucket`, `microsoft/winget-pkgs`
   (via `WINGET_GITHUB_TOKEN`), and one crates.io account. Per-repo
   `install.json` only names the product slot (formula path, bucket manifest,
   winget identifier, crate list). See [docs/RELEASE_SECRETS.md](../../RELEASE_SECRETS.md).

## Consequences

- First kit release (j.3) uses a **real semver** and production channels; there is
  no separate “safe rehearsal” semver in the current kit.
- CR-001 (Linux webview deps) and CR-002 (Homebrew **and Scoop** renderer via
  sc-compose bootstrap) must be **resolved** before j.3; waivers block the phase.

## References

- [docs/plans/phase-J/README.md](plans/phase-J/README.md)
- [README.sc-publish.md](../../README.sc-publish.md)
