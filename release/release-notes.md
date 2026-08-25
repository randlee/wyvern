# Wyvern v0.4.0

## Summary

- **version:** 0.4.0
- **release date:** 2026-08-24
- **release owner:** publisher

Phase H ships XHTML reporting (view + review). Phase I adds wizard native path pickers. g.15 adds `wyvern examples list` for bundled example discovery. Builds on the Phase F/G extension runtime and Phase D wizard flows (0.2.x / 0.3.x).

## Included Changes

### Phase H — XHTML reporting

- Report host bind (`{"type":"report", ...}`): dedicated `/report/{page}` route, no wizard APIs on report surfaces
- `.xhtml` suffix opens a single panel in view mode
- `wyvern report-xhtml manifest.json` stitches an ordered panel array
- `wyvern report-xhtml --review` review shell with per-panel comments, Cancel / Approve, structured finish JSON
- `wyvern-reporting` skill pack and `share/wyvern/examples/xhtml-review/`

### Phase I — Wizard native path pickers

- In-page OS file/folder choosers via `WyvernApi.postPickerFile()` / `postPickerFolder()` (ADR-0026)
- Host routes `POST /api/picker/file` and `POST /api/picker/folder`
- `share/wyvern/examples/path-picker/` two-page wizard — closes #99

### Phase G (g.15) — Examples catalog

- `wyvern examples list` / `wyvern examples list --json` discovers bundled examples from README frontmatter (`name`, `description`)
- Shipped examples live under `share/wyvern/examples/` (path-picker, template-picker, agent-dag, askuserquestion-hook, xhtml-review)

## Operator / User Impact

- Agents can discover bundled examples via `wyvern examples list` without reading checkout docs.
- XHTML review finish JSON is `{ "button": "finish", "data": { "approved", "comments", "panels" } }`.
- Wizard pages can open native file/folder pickers without leaving the session.

## Packaging / Distribution Notes

- **crates.io:** `wyvern-schema`, `wyvern-wizard`, `wyvern-host`, `wyvern-viewer`, `wyvern-cli` → 0.4.0 (dependency order preserved)
- **GitHub Releases:** tag `v0.4.0` — macOS aarch64/x86_64, Windows, Linux archives (`wyvern` + `wyvern-viewer` + `share/wyvern/ui/`)
- **Homebrew:** `randlee/homebrew-tap` formula updated by release workflow
- **winget:** automated step requires prior bootstrap of `randlee.wyvern` in `microsoft/winget-pkgs` (see `docs/WINGET_SETUP.md`)

## Known Issues / Waivers

- **winget:** first automated submission may fail until manual bootstrap manifest is merged; crates.io and GitHub Release assets are unaffected.

## Follow-Up

- Phase E: `--interactive` argv expansion and MCP server binary
- User extension registry (`~/.config/wyvern/extensions.json`)
- Back-merge `main` → `develop` after release
