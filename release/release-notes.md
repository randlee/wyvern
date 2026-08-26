# Wyvern v0.5.0

## Summary

- **version:** 0.5.0
- **release date:** 2026-08-26
- **release owner:** publisher

Headless CI and agent hardening on top of v0.4.0 (Phase H XHTML reporting, Phase I wizard pickers, g.15 examples catalog). Undriven blocking dialogs in `--viewer none` mode now fail fast with exit **6** instead of silently emitting dismissed JSON.

## Included Changes

### Headless / CI / agents

- **`WYVERN_VIEWER=none`** uses a **30s** idle session budget (embedded viewer unchanged at **600s**)
- Undriven blocking dialogs exit **`SESSION_TIMEOUT_ERROR`** (exit **6**) — CI misconfiguration is a hard fail
- `docs/plans/phase-C/c9-testing-headless.md` — active-drive rules; Playwright timeouts are hang detectors only
- Playwright input picker specs wait for mock picker field population before OK

## Operator / User Impact

- **Agents / CI:** If a headless blocking command hangs then exits **6**, the harness did not drive `WYVERN_DIALOG_URL` — fix the test, do not raise timeouts.
- **Desktop users:** No behavior change for embedded viewer (default product path).
- **Instant headless smoke:** `WYVERN_VIEWER=none wyvern examples list` (no dialog host).

## Packaging / Distribution Notes

- **crates.io:** `wyvern-schema`, `wyvern-wizard`, `wyvern-host`, `wyvern-viewer`, `wyvern-cli` → 0.5.0 (dependency order preserved)
- **GitHub Releases:** tag `v0.5.0` — macOS aarch64/x86_64, Windows, Linux archives (`wyvern` + `wyvern-viewer` + `share/wyvern/ui/`)
- **Homebrew:** `randlee/homebrew-tap` formula updated by release workflow
- **winget:** automated step requires prior bootstrap of `randlee.wyvern` in `microsoft/winget-pkgs` (see `docs/WINGET_SETUP.md`)

## Known Issues / Waivers

- **winget:** first automated submission may fail until manual bootstrap manifest is merged; crates.io and GitHub Release assets are unaffected.
- **Homebrew:** formula `brew test` may expect `--help` exit **2** (v0.5.0 exits **0**); Intel Mac tarball URL may need tap-side fix.

## Follow-Up

- Phase E: `--interactive` argv expansion and MCP server binary
- User extension registry (`~/.config/wyvern/extensions.json`)
- Winget bootstrap PR to `microsoft/winget-pkgs`
- Homebrew tap test + Intel tarball fixes
