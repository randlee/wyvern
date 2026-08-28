# Org publish destinations

`release/org-destinations.toml` is vendored **byte-for-byte** with the publish kit.
It is the sole authority for:

1. **Required channels** — every entry in `required_channels` must appear in
   `release/install.json` → `channels`. Agents **must not** omit them when
   bootstrapping or migrating a consumer repo.
2. **Fixed destinations** — `tap_repository`, `bucket_repository`, and the
   winget `publisher_id` prefix are **not overridable** in `install.json`.
   Wrong values fail `install.py`; omitted Homebrew/Scoop destination fields
   are injected from this file at sync time.

## Agent rules

- Read `release/org-destinations.toml` before authoring or editing
  `release/install.json`.
- Never propose a per-repo Homebrew tap, Scoop bucket, or non-`randlee.*` winget
  identifier to “simplify” setup.
- Product-specific slots remain in `install.json`: formula path, Scoop manifest
  path, winget package name suffix, crate list, binaries, bundled paths.
- PyPI and other channels not listed in `required_channels` stay opt-in per repo.

## Verification

```bash
python plugins/sc-publish/install.py --input release/install.json --dry-run .
```

Sync fails closed when mandatory channels or destinations drift.

See also: `release/publish-channel-contracts.toml`, `ref/channel-contracts.md`.
