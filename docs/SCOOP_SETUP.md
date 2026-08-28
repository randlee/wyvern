# Scoop bucket — sc-publish profile

Wyvern publishes to Scoop via the vendored **sc-publish** kit. The post-release
leg is `scoop-publish.yml`; it is independently retryable by tag.

## Package identity

| Field | Value |
|-------|-------|
| Bucket repo | [`randlee/scoop-bucket`](https://github.com/randlee/scoop-bucket) |
| Manifest path | `bucket/wyvern.json` |
| Installed binary | `bin/wyvern.exe` (from the Windows release ZIP) |
| Source repo | `https://github.com/randlee/wyvern` |

Declared in `release/install.json` → rendered `release/publish-artifacts.toml`
`[channels.scoop]`.

The bucket repository is **public and cloneable**. An empty `bucket/` directory
is acceptable on first run: `scoop-publish.yml` creates `bucket/wyvern.json`
from `release/scoop/manifest.json.j2` when it first succeeds.

## Release model (kit)

1. **Root release** (`release.yml` on `main`) builds and uploads GitHub Release
   assets, including the Windows ZIP:
   `wyvern_<version>_x86_64-pc-windows-msvc.zip`
2. **Post-release leg** — dispatch `scoop-publish.yml` with input `tag=vX.Y.Z`.
3. Workflow reads `[channels.scoop]` from the release manifest, verifies the
   Windows ZIP exists on the published GitHub Release, bootstraps the kit
   `sc-compose` renderer on the runner (not the product `wyvern` binary),
   renders `bucket/wyvern.json`, and pushes to `randlee/scoop-bucket`.
4. **Retry:** Re-dispatch `scoop-publish.yml` for the same tag; an unchanged
   manifest is a no-op commit skip.

## Required secret

| Secret | Purpose |
|--------|---------|
| **`SCOOP_BUCKET_TOKEN`** | PAT with **`contents:write`** on `randlee/scoop-bucket` |

Same secret name and value as on every other sc-publish consumer repo (see
[docs/RELEASE_SECRETS.md](RELEASE_SECRETS.md)).

The repository **`GITHUB_TOKEN` is not sufficient** and is **not** used to push
Scoop manifest updates. Preflight fails closed if `SCOOP_BUCKET_TOKEN` is
missing or not live (GitHub `GET /user`).

Recommended: classic PAT with `public_repo`, or a fine-grained PAT on
`randlee/scoop-bucket` with Contents: Read and write.

See also: [docs/RELEASE_SECRETS.md](RELEASE_SECRETS.md),
`release/publish-channel-contracts.toml` `[channels.scoop]`.

## First-run bootstrap checklist

Complete this once before j.3 / the first kit production release:

1. **Bucket repo exists and is public**
   - URL: `https://github.com/randlee/scoop-bucket`
   - Clone probe: `git ls-remote https://github.com/randlee/scoop-bucket.git`
2. **`SCOOP_BUCKET_TOKEN` is present** on `randlee/wyvern` (shared org PAT — same
   name/value on all kit repos)
   - Confirm with `gh secret list | rg SCOOP_BUCKET_TOKEN`
   - Do **not** use repository `GITHUB_TOKEN`
3. **Authenticated push capability**
   - Token must allow a push to the bucket default branch (`main`)
   - First successful `scoop-publish.yml` run seeds `bucket/wyvern.json` if
     the path does not exist yet (empty `bucket/` is fine)
4. **Windows asset name** on the GitHub Release must be
   `wyvern_<version>_x86_64-pc-windows-msvc.zip` (kit archive naming)
5. **Dispatch the leg** after the GitHub Release exists:

```bash
gh workflow run scoop-publish.yml -f tag=vX.Y.Z --ref main
gh run watch --exit-status
```

## Consumer install (after the first manifest exists)

```bash
scoop bucket add randlee https://github.com/randlee/scoop-bucket
scoop install wyvern
```

## Operator dispatch

```bash
gh workflow run scoop-publish.yml -f tag=vX.Y.Z --ref main
gh run watch --exit-status
# Probe
curl -fsS "https://raw.githubusercontent.com/randlee/scoop-bucket/main/bucket/wyvern.json"
```

Cursor/ATM: follow `.claude/agents/scoop-publisher.md` (inline in Cursor).

## Related

- Phase J plan: [docs/plans/phase-J/README.md](plans/phase-J/README.md)
- Kit workflow: `.github/workflows/scoop-publish.yml` (vendored — do not edit)
