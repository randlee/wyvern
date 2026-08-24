# Installation (`wyvern`, `sc-compose`, `python3`)

Load from the Layer 0 router when Step 1 fails because a tool is missing
or the wrong binary is on `PATH`. Report preexec requires **python3**.
Panel templates require **sc-compose** (or `wyvern compose render`).
Viewing requires **wyvern** built with Phase H extensions.

## Check first

```bash
which wyvern && wyvern --version
command -v python3 && python3 --version
command -v sc-compose && sc-compose --version
```

Then confirm the report extensions:

```bash
wyvern extensions list
wyvern extensions show report-xhtml
wyvern extensions show report-xhtml-review
wyvern extensions show xhtml-suffix
```

If `report-xhtml` is missing from the catalog, the `wyvern` on `PATH` is
older than Phase H. Build from this repo (or `integrate/phase-H`).

## wyvern

From a clone of this monorepo:

```bash
cargo build --release -p wyvern
export PATH="$(pwd)/target/release:$PATH"
```

Or install:

```bash
cargo install --path crates/wyvern --locked
```

**Common mistake:** `/usr/local/bin/wyvern` or `~/.cargo/bin/wyvern` is
ahead of the worktree build. Use `type -a wyvern` and prepend
`target/release`.

Minimum: a CLI that expands `xhtml-suffix`, `report-xhtml`, and
`report-xhtml-review`, and whose `extensions list --json` shows
`expands_to: "report"` for those ids.

## python3

Preexec runs `scripts/ext/xhtml_report.py` (also shipped under
`{wyvern_share}/scripts/ext/`). Requires Python 3.9+ on `PATH` as
`python3`. The catalog `requires` field marks `python3` for
`xhtml-suffix` and `report-xhtml`; `[missing]` on `extensions list`
means install Python before opening reports.

Validate a manifest without opening a window:

```bash
python3 scripts/ext/xhtml_report.py --validate-manifest path/to/review.json
```

## sc-compose

Used to render [templates/panel.xhtml.j2](../../templates/panel.xhtml.j2).
Either call `sc-compose` directly or go through the shipped compose
extension:

```bash
wyvern compose render --help
wyvern extensions show compose-render
```

If `sc-compose` is not on `PATH`, `compose-render` shows `[missing]`.
Install sc-compose, or write the `.xhtml` fragment by hand from the
template. Hand-authored fragments are valid; the template is a starter,
not a required toolchain for every panel.

## Known issues

| Symptom | Likely cause | Fix |
|---------|----------------|-----|
| `wyvern: command not found` | Not on PATH | Build `-p wyvern` and export `target/release` |
| `report-xhtml` unknown | Pre-Phase-H binary | Rebuild from this tree |
| `python3: command not found` / `[missing]` | No Python 3 | Install `python3`; re-run `extensions list` |
| `compose-render` `[missing]` | No `sc-compose` | Install sc-compose or author `.xhtml` by hand |
| Schema passes, preexec fails | Missing panel file | Paths are relative to the manifest directory |
| Worktree vs installed binary | Wrong `wyvern` first on PATH | `type -a wyvern`; use the tree you author from |
