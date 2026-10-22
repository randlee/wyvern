# `checksums.txt` release contract

Kit GitHub Releases from **v0.6.0** onward publish a `checksums.txt` asset next to the binary archives. v0.5.0 is not backfilled. That decision is **wontfix** in [`docs/plans/phase-k/issue-closeout-evidence.md`](plans/phase-k/issue-closeout-evidence.md).

## Contract

### Line format

The file is GNU `sha256sum` output. Each line is:

```text
<64 lowercase hex SHA-256><two spaces><basename>
```

- The hash is SHA-256 of the asset bytes.
- The separator is two spaces (binary mode). There is no text-mode `*` marker.
- The filename is the upload basename, with no directory prefix.
- Lines end in `\n`, including the last line.
- `checksums.txt` does not list itself.

v0.7.0 (`gh release download v0.7.0 --repo randlee/wyvern --pattern checksums.txt`):

```text
51bb9c96c81a7da0abc282f65187d2a9c18b65cccf3bbead6d15417e6c9e214f  wyvern_0.7.0_aarch64-apple-darwin.tar.gz
7f16ad1fa1e8e412307ee55c019594032b4b22f6b6a6d4911adf0e60eddd6b8b  wyvern_0.7.0_x86_64-apple-darwin.tar.gz
92235dd9097890d1ffabcff4b33a4d19fafc231165668ceb961f05e1593f486f  wyvern_0.7.0_x86_64-unknown-linux-gnu.tar.gz
a4e94411b9a21acd7431ff94231d1974179cfd8852e7641c972ac8a4117d5d17  wyvern_0.7.0_x86_64-pc-windows-msvc.zip
```

### Asset coverage

Every regex printed by

```bash
python3 .github/scripts/release_artifacts.py release-asset-patterns \
  --manifest release/publish-artifacts.toml
```

must name files the release job checksums. `.github/workflows/release.yml` step **Generate checksums** runs `sha256sum` on `*.tar.gz`, `*.zip`, and `*.whl` in the collected release directory and writes `checksums.txt`. The published-release probe also requires `^checksums\.txt$`.

Coverage rule: a manifest asset pattern is uncovered when its required filename suffix is not one of those globs. `test_checksum_step_covers_every_manifest_asset_pattern` in `.github/scripts/tests/test_release_artifacts.py` fails in that case.

Wyvern kit binary archives come from `release_targets` in [`release/install.json`](../release/install.json):

| Target | Archive | v0.7.0 asset |
|--------|---------|--------------|
| `aarch64-apple-darwin` | `tar.gz` | `wyvern_0.7.0_aarch64-apple-darwin.tar.gz` |
| `x86_64-apple-darwin` | `tar.gz` | `wyvern_0.7.0_x86_64-apple-darwin.tar.gz` |
| `x86_64-pc-windows-msvc` | `zip` | `wyvern_0.7.0_x86_64-pc-windows-msvc.zip` |
| `x86_64-unknown-linux-gnu` | `tar.gz` | `wyvern_0.7.0_x86_64-unknown-linux-gnu.tar.gz` |

The archive name is `wyvern_<version>_<target>.<archive>`. v0.7.0 `checksums.txt` lists those four files and no others.

`.whl` stays in the checksum globs for manifests that publish Python wheels. This repository's manifest declares none, so current checksum files have no wheel lines. Notes, `install.json`, and pin files attached to the same GitHub Release are not checksum inputs.

### atm-core cross-check

atm-core `tools/bootstrap.py` treats the SHA-256 map in `tools/bootstrap.toml` `[wyvern.checksums]` as the trust anchor, keyed by asset filename.

1. The downloaded archive must match that pin.
2. Bootstrap then downloads `checksums.txt` from the same tag.
3. When the file is present, the line for the pinned filename must carry the same hash. A mismatch raises `checksums.txt does not confirm the pinned Wyvern checksum`.
4. When the file is absent, bootstrap warns `upstream checksums.txt missing; verified against pinned SHA256 only` and continues.

The parser splits each line on whitespace. `parts[0]` is the hash and `parts[-1]` is the filename, which matches the two-space format above.

Inspected pin: atm-core `a9d0efcd37d2a5946086e98be75a2e8780837d2b` (2026-10-20). `[wyvern] version = "0.6.0"` uses kit filenames only. Those four hashes equal the v0.6.0 `checksums.txt` lines. [`release/atm-core-send-to-pin.toml`](../release/atm-core-send-to-pin.toml) sets `wyvern_pin = "0.6.0"` and does not name release archives.

`wyvern_asset_name` selects the legacy v0.5.0 names (`wyvern-windows.zip`, `wyvern-linux.tar.gz`, `wyvern-macos-aarch64.tar.gz`, `wyvern-macos-x86_64.tar.gz`) only when the pin version is `0.5.0`. The current pins are not that version. v0.5.0 has no `checksums.txt`, and this repository does not add one.
