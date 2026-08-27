# Windows Package Manager (`winget`) — sc-publish profile

Wyvern publishes to `winget` via the vendored **sc-publish** kit. This replaces
the legacy inline `release.yml` job and fixes the v0.5.0 failure mode (missing
fork token + wrong asset name).

## Package identity

| Field | Value |
|-------|-------|
| Identifier | `randlee.wyvern` |
| Installed binary | `wyvern` (from `bin/wyvern.exe` in release archive) |
| Source repo | `https://github.com/randlee/wyvern` |

Declared in `release/install.json` → rendered `release/publish-artifacts.toml`
`[channels.winget]`.

## Release model (kit)

1. **Root release** (`release.yml` on `main`) builds and uploads GitHub Release
   assets, including Windows ZIP:
   `wyvern_<version>_x86_64-pc-windows-msvc.zip`
2. **Post-release leg** — dispatch `winget-publish.yml` with input `tag=vX.Y.Z`.
3. Workflow reads manifest config, verifies release assets exist, then:
   - Probes `microsoft/winget-pkgs` for existing manifest or open PR (fail closed)
   - Submits via pinned `vedantmgoyal2009/winget-releaser@v2` if absent
4. **Retry:** Re-dispatch `winget-publish.yml` for the same tag; detect-and-skip
   prevents duplicate submissions.

## Required secret

| Secret | Purpose |
|--------|---------|
| **`WINGET_GITHUB_TOKEN`** | PAT that can **fork** `microsoft/winget-pkgs` and **open PRs** |

Recommended: classic PAT with `public_repo` (or fine-grained equivalent on fork
target). Preflight checks token liveness via GitHub `GET /user`; fork capability
is validated at submit time by `winget-releaser`.

The repository **`GITHUB_TOKEN` is not sufficient** for winget submit.
Preflight fails closed if `WINGET_GITHUB_TOKEN` is missing or not live.

See also: [docs/RELEASE_SECRETS.md](RELEASE_SECRETS.md),
`release/publish-channel-contracts.toml` `[channels.winget]`.

## First-release bootstrap (one-time)

Before the automated leg can succeed, `randlee.wyvern` must exist in
`microsoft/winget-pkgs`:

1. Ship a GitHub Release with the Windows ZIP asset (kit archive name above).
2. Prepare the initial three-file manifest set (version, installer, locale).
3. Submit manually to `microsoft/winget-pkgs` (or via maintainer fork PR).
4. After merge, use `winget-publish.yml` for all subsequent versions.

## Verification

Release verification checks **submission success**, not same-day
`winget install` visibility — Microsoft review typically adds 1–2 days lag.

```bash
# After dispatch
gh run list --workflow winget-publish.yml --limit 1
# Probe (optional)
gh api "repos/microsoft/winget-pkgs/contents/manifests/r/randlee/wyvern/<version>"
```

## Operator dispatch

```bash
gh workflow run winget-publish.yml -f tag=vX.Y.Z --ref main
gh run watch --exit-status
```

Cursor/ATM: follow `.claude/agents/winget-publisher.md` (inline in Cursor).

## Related

- Phase J plan: [docs/plans/phase-J/README.md](plans/phase-J/README.md)
- Kit workflow: `.github/workflows/winget-publish.yml` (vendored — do not edit)
