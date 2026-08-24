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

---

## Wizard workflow hooks (Phase G Wave 2 — g.4)

CLI-owned. Host and page JS do not execute these scripts and must not read or write the files they target. Architecture: [ADR-0023](../architecture.md#adr-0023-wizard-workflow-prepost-scripts-cli-layer), [wizard-workflow-architecture.md](../plans/phase-G/wizard-workflow-architecture.md). Lands in **g.4**; consumed by **g.5–g.7**.

**REQ-0124** — When a wizard command includes optional `workflow.pre`, the **`wyvern` CLI** runs that script **before** binding the host. The script writes one JSON object to stdout. The CLI deep-merges `config_patch` (object) into the wizard `config` so the first `GET /api/wizard/state` exposes the patched config. Invalid JSON, non-zero exit, timeout, or a path outside the workflow allowlist is a `workflow` / `WORKFLOW_ERROR` failure (exit `9`); the host does not start. Webview / page JS has no disk access for this step.

**REQ-0125** — When a wizard command includes optional `workflow.post` and the session ends with `button: "finish"`, the **`wyvern` CLI** runs that script **after** host finish and **before** any `next_wizard` hop. The CLI writes the full finish JSON to the script's stdin. Non-zero exit, timeout, or allowlist failure is `WORKFLOW_ERROR` (exit `9`). `cancel` and `dismissed` skip post. `--workflow-dry-run` appends `--dry-run` to the post (and pre) argv; scripts must not write side effects in that mode. Webview / page JS has no disk access for this step.

---

## Wizard chaining (Phase G Wave 2 — g.4)

**REQ-0126** — A wizard finish JSON may include optional `next_wizard`: `{ "path": "...", "input": { }, "ui_root": "..." }`. This extends the REQ-0066 envelope; `path` is required when the object is present; `input` defaults to `{}`; `ui_root` is optional. The **`wyvern` CLI** (not the host) loads the next `wizard.json`, merges `input` into that wizard's `config`, runs its `workflow.pre` (pre `config_patch` wins over `input`), and repeats. Honor `next_wizard` only when `button` is `finish`. Maximum **16** wizard sessions per CLI invocation; a 17th hop is `WORKFLOW_ERROR`. `path` must canonicalize inside the workflow allowlist (`{wyvern_share}`, process cwd, or the current wizard.json directory); reject `..` / symlink escape. Final stdout is the **last** finish JSON (omit `next_wizard` on the emitted line). Architecture: [ADR-0024](../architecture.md#adr-0024-next_wizard-chaining-cli-orchestration).

---

## Welcome guide extension (Phase G Wave 2 — g.4)

**REQ-0127** — `wyvern guide` is a shipped argv-prefix extension (`id: "guide"`) that expands to the bundled welcome wizard (`{wyvern_share}/welcome/wizard.json`, `ui_root` `{wyvern_share}/welcome`). It is **not** `wyvern help` / `--help` (those stay stdout). The extension is listed in shipped `extensions.json` and mentioned on the global `--help` surface (g.1). Sprint: [g4-welcome-guide-wizard.md](../plans/phase-G/g4-welcome-guide-wizard.md).

---

## XHTML report extensions (Phase H — h.1/h.2/h.3)

**REQ-0140** — `type: "report"` command JSON is validated in `wyvern-schema` with fields `title`, `page`, optional `mode` (`view` \| `review`), optional `panels` (required when `mode: "review"`), optional viewer hints. No wizard `config`, `workflow`, or stack fields.

**REQ-0141** — `.xhtml` suffix expands via `xhtml-suffix` registry entry to `type: "report"`, `mode: "view"` (not wizard).

**REQ-0142** — `report-xhtml` and `report-xhtml-review` extensions expand via Phase F `command_from_file` from preexec-written `{tmpdir}/report-command.json`; match uses `argv_prefix` + `arg_suffix: ".json"`.

**REQ-0143** — View-mode report dismiss completes with stdout `{"button":"dismissed"}` via shared `POST /api/result` semantics.

**REQ-0144** — Review-mode finish completes with stdout JSON `{ "button": "finish", "data": { "approved", "comments", "panels" } }` from `POST /api/report/finish`. Contract: [xhtml-reporting-contract.md](../plans/phase-H/xhtml-reporting-contract.md).
