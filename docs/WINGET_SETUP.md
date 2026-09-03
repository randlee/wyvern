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

## PATH after install

| Channel | Command on PATH | Notes |
|---------|-----------------|-------|
| **winget** | `wyvern` | Requires `PortableCommandAlias: wyvern` in the installer manifest (see below) |
| Homebrew | `wyvern`, `wyvern-viewer` | Automatic via `bin.install` |
| Scoop | `wyvern` | Automatic via `"bin": "bin/wyvern.exe"` shim |
| `cargo install wyvern-cli` | `wyvern` | Only when `~/.cargo/bin` is on PATH (typical rustup setup) |

After `winget install randlee.wyvern`, open a **new terminal** if `wyvern` is not
found — winget may not refresh PATH in the current session.

## PortableCommandAlias (required)

Portable zip manifests must alias the nested binary to `wyvern`. Kit release
archives extract with a versioned top-level directory — `RelativeFilePath` is
relative to the **zip root**, not the inner folder:

```yaml
NestedInstallerFiles:
  - RelativeFilePath: wyvern_0.6.0_x86_64-pc-windows-msvc/bin/wyvern.exe
    PortableCommandAlias: wyvern
```

(Scoop uses the same layout via `extract_dir` + `bin/wyvern.exe`.)

Without `PortableCommandAlias`, winget defaults the shim name to the filename
(`wyvern.exe`). Reference: `release/winget-bootstrap/0.5.0/randlee.wyvern.installer.yaml`.

`winget-releaser` / komac does **not** emit this field today. After each
automated submission (or before merging a manual PR), verify the installer
manifest includes `PortableCommandAlias: wyvern`.

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

Same secret name and value as on every other sc-publish consumer repo (see
[docs/RELEASE_SECRETS.md](RELEASE_SECRETS.md)).

Recommended: classic PAT with `public_repo` (or fine-grained equivalent on fork
target). Preflight checks token liveness via GitHub `GET /user`; fork capability
is validated at submit time by `winget-releaser`.

The repository **`GITHUB_TOKEN` is not sufficient** for winget submit.
Preflight fails closed if `WINGET_GITHUB_TOKEN` is missing or not live.

See also: [docs/RELEASE_SECRETS.md](RELEASE_SECRETS.md),
`release/publish-channel-contracts.toml` `[channels.winget]`.

## First-release bootstrap (one-time)

Before the automated leg can succeed, `randlee.wyvern` must exist in
`microsoft/winget-pkgs`. **j.2 closeout (2026-08-27):** the path
`manifests/r/randlee/wyvern` is **absent**. Owner completes this bootstrap
**before j.3**:

1. Ship a GitHub Release with the Windows ZIP asset (kit archive name
   `wyvern_<version>_x86_64-pc-windows-msvc.zip`).
2. Prepare the initial three-file manifest set (version, installer, locale).
3. Submit manually to `microsoft/winget-pkgs` (or via maintainer fork PR)
   using `WINGET_GITHUB_TOKEN` (never repository `GITHUB_TOKEN`).
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
