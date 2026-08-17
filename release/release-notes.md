# Wyvern v0.3.0

## Summary

- **version:** 0.3.0
- **release date:** 2026-08-17
- **release owner:** publisher

Phase F ships declarative CLI extensions (suffix defaults, compose, CSV). Phase G adds agent-facing help, a skill catalog, and error-teaches recovery on top of that runtime. Builds on Phase D wizard flows (0.2.x).

## Included Changes

### Phase F — CLI extensions

- Extension runtime: bundled registry, argv match (suffix + subcommand), optional Python preexec, template expand → validated `Command` JSON
- Positional suffix defaults — open `.html` wizard pages and `wizard.json` roots without hand-authored JSON
- `compose render` — sc-compose preexec for slide/markdown composition workflows
- CSV — interactive HTML table viewer (`.csv` suffix) and `wyvern md` markdown variant

### Phase G — Agent usability

- First-class `--help` / `-h` and `wyvern help` listing shipped skills with copy-paste examples
- Extension-prefix `--help` skill cards at match time (`wyvern compose render --help`)
- Skill catalog: `wyvern extensions list` (text + `--json`) and `wyvern extensions show <id>`
- Near-miss diagnostics that name the skill and teach the next command
- Structured preexec failure recovery (child stderr in JSON; spawn vs exit vs missing-file)

## Operator / User Impact

- Agents and operators can discover skills via `wyvern help` and `wyvern extensions list` without reading source.
- Common file types (`.md`, `.html`, `.csv`, `wizard.json`) open via argv shorthands; JSON command strings remain supported.
- `compose render` and CSV table viewing require optional tools (`sc-compose`, `python3`) where documented.

## Packaging / Distribution Notes

- **crates.io:** `wyvern-schema`, `wyvern-wizard`, `wyvern-host`, `wyvern-viewer`, `wyvern-cli` → 0.3.0 (dependency order preserved)
- **GitHub Releases:** tag `v0.3.0` — macOS aarch64/x86_64, Windows, Linux archives (`wyvern` + `wyvern-viewer` + `share/wyvern/ui/`)
- **Homebrew:** `randlee/homebrew-tap` formula updated by release workflow
- **winget:** automated step requires prior bootstrap of `randlee.wyvern` in `microsoft/winget-pkgs` (see `docs/WINGET_SETUP.md`)

## Known Issues / Waivers

- **winget:** first automated submission may fail until manual bootstrap manifest is merged; crates.io and GitHub Release assets are unaffected.

## Follow-Up

- Phase E: `--interactive` argv expansion and MCP server binary
- User extension registry (`~/.config/wyvern/extensions.json`)
- Back-merge `main` → `develop` after release
