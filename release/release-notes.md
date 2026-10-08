# Wyvern v0.7.0

## Summary

- **version:** 0.7.0
- **release date:** 2026-10-08
- **release owner:** publisher

Minor release on top of **v0.6.0** (first sc-publish kit-managed production cut). Adds **sc-lint 0.5.0** boundary enforcement, **sc-observability v2** binary logging, and the **atm-core Send-To** picker example with cross-repo contract CI. Dialog JSON schemas and host IPC are unchanged for integrators on 0.6.x.

## Included Changes

### atm-core Send-To integration

- **`examples/wizards/atm-pick-member/`** — reference `PickerInput` / `PickerOutput` wizard page (atm-core vendors matching HTML)
- **`release/atm-core-send-to-pin.toml`** — pins atm-core **v1.6.1** fixture SHA256s; CI verifies vendored page bytes and runs Send-To surface tests + real `probe_wyvern.py`
- Wizard contract: roster travels as wizard **`config`**; terminal stdout is **`WizardResult`** — callers read **`.data`** as PickerOutput

### Tooling / CI / observability

- **sc-lint 0.5.0** — published tools, boundary TOML refresh, matrix native clippy, consumer Just/bootstrap lanes, Python/JSON gates (`docs/linting.md`)
- **sc-observability v2** — wyvern CLI binary logger migration (#161)
- **Winget PATH** — documents PortableCommandAlias behavior (`docs/WINGET_SETUP.md`)

## Operator / User Impact

- **Desktop / default embedded viewer:** no intentional behavior change vs 0.6.x for standard dialog commands.
- **Headless / CI (`WYVERN_VIEWER=none`):** unchanged 30s idle budget and fail-fast undriven dialogs (since 0.5.0).
- **atm-core operators:** optional Wyvern picker stays aligned with pinned fixtures; advance `atm-core-send-to-pin.toml` when the wire contract changes.

## Packaging / Distribution Notes

- **crates.io:** `wyvern-schema`, `wyvern-wizard`, `wyvern-host`, `wyvern-viewer`, `wyvern-cli` → **0.7.0** (dependency order preserved)
- **GitHub Releases:** tag **`v0.7.0`** — `wyvern_0.7.0_<target>.{tar.gz,zip}` plus checksums (`bin/wyvern`, `bin/wyvern-viewer`, `share/wyvern/ui/`)
- **Homebrew:** `randlee/homebrew-tap` formula bumped for v0.7.0
- **Scoop:** `randlee/scoop-bucket` manifest updated
- **winget:** submission workflow dispatched for `randlee.wyvern` (Microsoft catalog visibility may lag)

## Known Issues / Waivers

- None recorded for this tag.

## Follow-Up

- **`main` → `develop` back-merge** after production cut (this release)
- Refresh **`release/release-notes.md`** on `main` before the next RC (done in back-merge PR)
- **`site/announcements/wyvern-v0.7.0.md`** — user-facing announce (consider PR to `main` if not already merged)
