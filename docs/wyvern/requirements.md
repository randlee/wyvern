# `wyvern` (CLI) — Requirements

*Part of the [principal requirements](../requirements.md).*

---

## CLI Invocation

**REQ-0001** — Accept a JSON command as an inline string argument: `wyvern '{"type":"message",...}'`

**REQ-0002** — Accept a `.json` file path and load it as the command: `wyvern input.json`

**REQ-0003** — Accept a `.md` file path and open it as a markdown viewer: `wyvern my-doc.md`

**REQ-0004** — Accept JSON via stdin when no argument is provided: `echo '{...}' | wyvern`

**REQ-0005** — Support `--interactive` (alias `--persistent`) to enter a readline loop on stdin, processing one JSON command per line until `{"action":"exit"}` or window close.

**REQ-0006** — Support `--mcp` to start Wyvern as an MCP server using stdio transport.

**REQ-0007** — `show`, `hide`, and `exit` are valid only inside the `--interactive` command loop. They are not valid as single-shot CLI commands.

## Host options (c.10+)

**REQ-0115** — `--bind <ADDR:PORT>` sets the dialog HTTP bind address (default `127.0.0.1:0`). Passed to `wyvern-host`.

**REQ-0116** — `--ui-root <PATH>` sets the static UI directory (default: packaged `share/wyvern/ui/`). Passed to `wyvern-host`.

**REQ-0117** — `--viewer <embedded|none|system|chrome|safari|edge|firefox>`. **Default: `embedded`** (c.15). Env `WYVERN_VIEWER` overrides. CI uses `none`. c.10: parse enum, implement `none` only. Registry: [http-viewer-contract.md](../plans/phase-C/http-viewer-contract.md).

**REQ-0118** — `wyvern browsers list` / `wyvern browsers refresh` (c.15).

## CLI Extensions (Phase F + G)

**REQ-0130** — After host-flag strip: (1) if remainder starts with `--help`, `-h`, or `help`, print global usage to stdout and exit 0; (2) if remainder matches an extension prefix with help-only tokens, print a skill card to stdout and exit 0 (before `requires` skip); (3) `ExtensionRegistry::match_with_diagnostics` on the remainder; (4) on no match, emit near-miss diagnostics (REQ-0136) before JSON / `.json` / stdin fallback; (5) otherwise expand matched extension to validated `Command` JSON (ADR-0022, [agent-usability-contract.md](../plans/phase-G/agent-usability-contract.md)).

**REQ-0131** — A matched extension expands to validated `Command` JSON. When the extension sets `host.ui_root`, that value replaces CLI `--ui-root`.

**REQ-0132** — `wyvern extensions list` emits a skill index: rich text blocks with match invocation, requires availability, expand type, and example; `wyvern extensions list --json` prints a JSON **array** of skill records per [skills-catalog-contract.md](../plans/phase-G/skills-catalog-contract.md); `wyvern extensions show <id>` prints one skill (text or `--json` object).

**REQ-0133** — Wizard finish from extension-hosted pages (including the CSV table viewer) calls `wyvernWizardFinish` with the full visited stack: `window.wyvern.stack` plus `{ page, data }` (REQ-0024).

## Agent CLI surfaces (Phase G)

Primary user: AI agents with **no checkout docs**. Authoritative contract: [agent-usability-contract.md](../plans/phase-G/agent-usability-contract.md). README and plan docs are informative, not blocking for agent discovery.

**REQ-0134** — `wyvern --help`, `wyvern -h`, and `wyvern help` exit **0** and print usage to stdout listing **every** shipped extension skill with copy-paste examples (including `.csv`, `table`, `md <file.csv>`, and `compose render` with declared flags).

**REQ-0135** — When argv matches an extension `argv_prefix` and the remainder is only `--help` or `-h`, Wyvern prints that extension's skill card to stdout and exits **0** — including when required binaries are absent from `PATH` and without requiring a suffix path token (e.g. `wyvern md --help`, `wyvern compose render --help`).

**REQ-0136** — Near-miss argv (unknown file suffix, skipped `requires`, incomplete prefix, bare prefix without suffix) emits structured stderr JSON naming the skill and next argv. `UnknownInput` reuses `ErrorCode::ParseError` but `message`/`recovery` must not contain `Input was not valid JSON`. Recovery points at in-binary commands (`wyvern --help`, `wyvern extensions list`) before checkout-only doc paths.

**REQ-0137** — **Registry/help parity:** every `id` in shipped `share/wyvern/extensions.json` appears in global `--help` and in `extensions list --json`; each shipped entry has non-empty `description` and at least one `examples` string. CI tests enforce parity when the registry changes.

---

## Interactive Mode (Phase E)

**REQ-0120** — In `--interactive` mode, commands are processed sequentially. Blocking dialog commands retain normal modal behavior inside the loop.

**REQ-0121** — In `--interactive` mode, a blocking dialog command writes its normal JSON result to stdout on completion, then the loop continues.

**REQ-0122** — `{"action":"show"}` and `{"action":"hide"}` toggle **`wyvern-viewer`** visibility via **`wyvern` CLI** (when embedded); return `{"action":"show|hide","ok":true}`. Host HTTP server stays up. Not `HostSession` methods.

**REQ-0123** — `{"action":"exit"}` shuts down `HostSession` (host) and CLI-owned viewer, returning `{"action":"exit","ok":true}` before shutdown.
