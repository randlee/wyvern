# Wyvern — Architecture

Architecture decisions are recorded as ADRs. Cross-cutting ADRs live here. Crate-specific ADRs live in `docs/<crate>/architecture.md` — follow the links below for progressive disclosure.

---

## Crate Architecture Map

| Crate | Responsibility | ADRs |
|-------|---------------|------|
| `wyvern` | CLI entry point, arg parsing, `--interactive` loop | [docs/wyvern/architecture.md](wyvern/architecture.md) |
| `wyvern-schema` | JSON types, validation, error messages | [docs/wyvern-schema/architecture.md](wyvern-schema/architecture.md) |
| `wyvern-host` | HTTP dialog server, static UI, session/result (c.10+) | [docs/wyvern-host/architecture.md](wyvern-host/architecture.md) |
| `wyvern-viewer` | Optional URL-only webview launcher (c.15) | [docs/wyvern-viewer/architecture.md](wyvern-viewer/architecture.md) |
| `wyvern-wizard` | Wizard navigation state machine | [docs/wyvern-wizard/architecture.md](wyvern-wizard/architecture.md) |
| `wyvern-mcp` | MCP server, tool mapping, persistent host | [docs/wyvern-mcp/architecture.md](wyvern-mcp/architecture.md) |
| ~~`wyvern-window`~~ | **Deleted in c.9** — crate removed from the workspace; archival docs only under `docs/wyvern-window/` | [docs/wyvern-window/architecture.md](wyvern-window/architecture.md) (archival) |

---

## Cross-Cutting ADRs

### ADR-0003: Rust as the implementation language

**Status:** Accepted

`wry` is a Rust crate. Rust gives a single statically-linked binary, small footprint, fast startup, and strong type-safety on the schema layer. `serde_json` for JSON I/O; `strsim` for Levenshtein validation suggestions.

---

### ADR-0004: JSON as the sole protocol — stdin/stdout

**Status:** Accepted

JSON in (stdin, file, or inline arg), JSON out (stdout). Errors on stderr as structured JSON. One command per line in interactive mode.

**Consequences:** Works from any shell, language, or agent. MCP tool parameters map 1:1 — no restructuring. Binary data passed by file path or base64. The protocol stays intentionally small: blocking dialog commands plus a few lifecycle actions in `--interactive`.

---

### ADR-0012: Prefer the smallest coherent API surface

**Status:** Accepted

Wyvern should solve the product with the fewest command shapes that preserve clear semantics. If an interaction starts to feel complicated, treat that as a documentation, scoping, or boundary problem first.

**Consequences:** `message` remains a blocking modal. Persistent transports (`--interactive`, MCP) do not silently change dialog semantics. Modeless behavior belongs in a separate future `notification` command rather than overloading existing commands.

---

### ADR-0005: Wizard navigation uses browser-history model

**Status:** Accepted — implementation d.2; regression tests d.3

Cursor-over-array model: back moves cursor without discarding forward entries; forward to the same page restores cached data; forward to a different page truncates stale forward history. Full text: [docs/wyvern-wizard/architecture.md](wyvern-wizard/architecture.md).

---

### ADR-0006: Host is domain-agnostic — wizard data is opaque

**Status:** Accepted — NFR-0008

Host stores and passes through page `data` without inspection. Domain branching lives in page JS. Full text: [docs/wyvern-wizard/architecture.md](wyvern-wizard/architecture.md).

---

### ADR-0007: Single `WizardSession` type hides history internals

**Status:** Accepted — implemented (Phase D d.1–d.2)

`wyvern-wizard` exposes one concrete `WizardSession`; private `history` holds `entries` + `cursor`. `wyvern-host` holds the session and serializes `snapshot()`. Full text: [docs/wyvern-wizard/architecture.md](wyvern-wizard/architecture.md).

---

### ADR-0011: Cargo workspace crate structure and boundaries

**Status:** Accepted — **amended c.9** (HTTP host delivery)

**Target workspace** (after c.16 delivery):

```
wyvern-schema   →  (no internal deps — pure types + logic)
wyvern-wizard   →  wyvern-schema
wyvern-host     →  wyvern-schema [, wyvern-wizard for Phase D wizard routes], HTTP stack (axum/tokio)
wyvern-viewer   →  wry, winit [, serde/serde_json for OS-close dismiss JSON only — ADR-0021] (optional crate — URL navigate + dismiss)
wyvern          →  wyvern-host, wyvern-schema [, wyvern-wizard for `wyvern wizard lint` — Phase G g.9/g.14]
wyvern-mcp      →  wyvern-host, wyvern-schema
```

**Sprint timeline:** c.9 deletes `wyvern-window`; c.10 adds `wyvern-host`; c.15 adds `wyvern-viewer`; c.16 release.

**Boundary rules:**
- `wyvern-schema` and `wyvern-wizard` are pure logic — no I/O, no network, no async
- `wyvern-host` owns TCP/HTTP/static serve/dialog session — **no** `wry`/`winit`, **no** inline HTML templates, **no** embedded viewer spawn
- `wyvern-host` may depend on `wyvern-wizard` from Phase D (d.1) for wizard route state orchestration only — pure logic stays in `wyvern-wizard`
- `wry` and `winit` only in `wyvern-viewer` (optional) — not in `wyvern-host`
- **`wyvern` CLI** spawns `wyvern-viewer` as a **subprocess** for `--viewer embedded` — sibling binary discovery, not a required library dependency (dev builds may use `CARGO_BIN_EXE_wyvern-viewer`)
- **`wyvern` CLI** may depend on **`wyvern-wizard`** from Phase G (g.9/g.14) for **static** `wyvern wizard lint` only — no session/history APIs; lint modules are allowlisted in `boundaries/wyvern-wizard/wizard.toml`
- `wyvern-mcp` accesses dialogs only through `wyvern-host`'s public API
- `wyvern` binary is a thin entry point — logic belongs in library crates
- `wyvern-window` is **removed** — do not extend. Optional URL webview = **`wyvern-viewer`** (c.15).
- `wyvern-viewer` may use `serde`/`serde_json` **only** for OS-close dismiss JSON (ADR-0021) — no general JSON protocol ownership; still must not depend on `wyvern-schema` / `wyvern-host`

Boundary rules are encoded in `boundaries/` and enforced in CI.

---

### ADR-0022: CLI extension registry as argv preprocessor (Phase F)

**Status:** Accepted (Phase F f.1)

**Context:** Phase F adds declarative argv → `Command` JSON expansion (suffix/subcommand aliases). Phase E needs `--interactive` argv expansion; MCP tools may need equivalent commands without duplicating registry logic.

**Decision (Path A):**

1. Extension engine lives in **`wyvern` crate** as public `wyvern::extensions` module; used by `wyvern` binary and Phase E `--interactive` loop.
2. Extensions produce validated `Command` JSON; **Phase F shipped variants only** — no new schema variants until Phase H (see Amendment below).
3. **`wyvern-mcp` boundary unchanged:** MCP tools accept **pre-expanded `Command` JSON** (compose in tool handler or subprocess `wyvern` for expand-only). No `wyvern-mcp → wyvern-cli` dependency.

**Consequences:** Phase F f.1 lands ADR-0022 + contract. Phase E e.3 documents MCP tool pattern for CSV/HTML. Path B (mcp → wyvern lib) deferred unless explicitly re-opened.

**Amendment (Phase G — agent CLI surfaces):**

**Status:** Accepted (Phase G plan — g.1–g.3 implement)

**Context:** Phase F ships the extension engine; agents discover skills only via in-binary output (`--help`, `extensions list`, stderr). A shipped extension is incomplete if registry, help, and catalog diverge.

**Decision:**

1. **In-binary skill pack** is authoritative for agents: global `--help`, extension `--help` skill cards, `extensions list` / `--json`, `extensions show`, and near-miss stderr (see [agent-usability-contract.md](plans/phase-G/agent-usability-contract.md)).
2. **Registry/help parity:** every shipped `extensions.json` entry must appear in help and catalog output with `description` + `examples` (REQ-0137). CI tests enforce parity.
3. **README / plan docs** are informative for humans; they are not acceptance gates for agent discoverability.

**Consequences:** New or changed extensions in any phase must update `usage_message()`, registry prose fields, and help/catalog tests in the same PR. Phase G g.1–g.3 land REQ-0134–REQ-0137. Phase E `--interactive` inherits the same argv surfaces.

**Amendment (Phase H — ADR-0025):**

**Status:** Accepted (planning — Phase H h.1)

**Context:** Phase H adds static XHTML/HTML report viewing via a new `Command::Report` variant (ADR-0025). Extensions still expand to validated JSON; MCP Path A unchanged.

**Decision:**

1. Extensions may expand to **`type: "report"`** — the sole Phase-H-added `Command` variant.
2. Report uses shared host routes in `wyvern-host` (`/report/*`, optional `/api/report/finish`) — **no per-extension host handlers**.
3. **`wyvern-mcp` boundary unchanged:** MCP tools accept pre-expanded `Command` JSON including `report`.

**Consequences:** Amends pre-H Decision point 2. Full report semantics: [ADR-0025](#adr-0025-report-command-static-xhtmlhtml-review-surfaces).

---

### ADR-0023: Wizard workflow pre/post scripts (CLI layer)

**Status:** Accepted (planning — Phase G Wave 2, g.4)

**Context:** Wave 2 examples must query and apply on-disk state (Claude Code hooks, template copy, DAG export). Page JS in the webview must not read or write those files. Extension `preexec` already spawns trusted scripts; a second ad-hoc subprocess stack would drift. Host-side I/O would violate ADR-0006 and ADR-0011.

**Decision:**

1. Optional `workflow: { "pre": "<path>", "post": "<path>" }` is a **known** field on wizard command JSON (REQ-0124, REQ-0125). `wyvern-schema` validates shape and non-empty path strings; it is not a REQ-0053 unknown field. **`wyvern-host` ignores `workflow` and never spawns scripts.**
2. The **`wyvern` CLI** owns execution in `crates/wyvern/src/workflow/`: `pre` after validate and **before** host bind; `post` after host finish when `button` is `finish`, **before** any `next_wizard` hop. `cancel` / `dismissed` skip post. Failure → `workflow` / `WORKFLOW_ERROR`, exit `9`; pre failure does not start the host.
3. Reuse Phase F `extensions/preexec.rs` spawn and stderr-tail helpers. Do not add a second subprocess stack. Timeout is **`WORKFLOW_SCRIPT_TIMEOUT` = 30s** (g.4).
4. **Allowlist:** resolved paths must canonicalize under `{wyvern_share}`, process cwd, or the current `wizard.json` directory. Reject `..` and symlink escape. `.py` scripts invoke `python3 <path>`; other paths execute as argv[0].
5. Pre stdout is one JSON object `{ "config_patch": { ... } }`; CLI deep-merges object keys into wizard `config` (patch wins; arrays/scalars replace). Post receives the full finish JSON on stdin.
6. `--workflow-dry-run` is a CLI flag (parsed with other host-adjacent flags). When set, CLI appends `--dry-run` to pre and post argv. Scripts must not apply side effects in that mode.
7. Spawned scripts receive `WYVERN_SHARE` (canonical share root), `WYVERN_REPO_ROOT` (existing value, else process cwd), and `WYVERN_BIN` (canonical wyvern executable from `current_exe`).

**Consequences:** g.4 lands the runner; g.5–g.7 only add scripts + `workflow` blocks. No new dialog types. MCP / `--interactive` auto-chain stays Phase E. Full text: [wizard-workflow-architecture.md](plans/phase-G/wizard-workflow-architecture.md).

---

### ADR-0024: `next_wizard` chaining (CLI orchestration)

**Status:** Accepted (planning — Phase G Wave 2, g.4)

**Context:** Welcome → example wizards need a full new `wizard.json` (new `ui_root`, new `workflow`), not another page inside one session. A host graph engine would violate ADR-0006 and ADR-0012.

**Decision:**

1. Optional `next_wizard: { "path": "<wizard.json>", "input": {}, "ui_root": "<optional>" }` is a **known** sibling of `button` / `data` / `stack` on finish request and result (REQ-0126; extends REQ-0066). `path` is required when the object is present; `input` defaults to `{}`; `ui_root` is optional. **Host copies the field through and does not resolve, load, or execute it.**
2. The **`wyvern` CLI** owns the loop in `crates/wyvern/src/workflow/chain.rs` + `pipeline.rs`: finish → post → resolve next → load → merge `input` into next `config` → pre (**pre `config_patch` wins** over `input`) → host → repeat.
3. Honor `next_wizard` only when `button` is `finish`. Maximum **16** wizard sessions per CLI invocation; a 17th hop is `WORKFLOW_ERROR` (exit `9`).
4. `path` and optional `ui_root` use the ADR-0023 allowlist. Relative `path` resolves `{wyvern_share}` first, then cwd, then the current wizard.json directory. Missing `ui_root` uses existing wizard-root inference.
5. Final stdout is the last finish JSON with `next_wizard` **omitted**. `--emit-all` is **out of scope** for Wave 2 (non-closure).
6. No `wyvern chain` subcommand. `wyvern guide` is an argv-prefix **extension** (`id: "guide"`, REQ-0127), not a built-in early return.

**Consequences:** Chaining is data-driven. Host remains a single-session server but **must passthrough** `next_wizard` on finish (g.4 `wyvern-host` deliverable) or the CLI loop never sees page-supplied hops. DAG *execution* stays out of Wyvern (g.7 export only). Full text: [wizard-workflow-architecture.md](plans/phase-G/wizard-workflow-architecture.md), [workflow-chain-contract.md](plans/phase-G/workflow-chain-contract.md).

---

### ADR-0025: Report command (static XHTML/HTML review surfaces)

**Status:** Accepted (planning — Phase H h.1)

**Context:** Agents need ad-hoc sc-compose XHTML panel review (single pane, arrays, optional Approve/Cancel) outside wizard stack semantics. Overloading `type: "wizard"` confuses authoring skills and WIZARD-LINT profiles. Phase F ADR-0022 forbade new schema variants; Phase H is the first deliberate exception.

**Decision:**

1. Add **`type: "report"`** to `wyvern-schema` — fields: `title`, `page`, optional `mode` (`view` \| `review`), optional `panels` (required when `mode: "review"` — manifest panel list for finish validation), optional viewer hints. No `config`, `workflow`, or stack fields.
2. **`wyvern-host`** binds report via a **third bind discriminant** (`dialog` \| `wizard` \| `report`) — **not** `is_wizard=true`. Report arm: `require_report_page`, dialog URL `/report/{page}`, `ServeDir` at `/report`; **forbidden:** `/wizard/` URLs or wizard static mounts for report sessions. Mounts `/shared/*` for report CSS/JS; registers `POST /api/report/finish` only when `mode: "review"`. `GET /api/dialog` rejected for report sessions.
3. **Extensions** (`xhtml-suffix`, `report-xhtml`, `report-xhtml-review`) expand via existing Phase F runtime. Multi-panel flows use preexec → **`command_from_file`** (`{tmpdir}/report-command.json`); match uses `arg_suffix: ".json"` with `argv_prefix` (same as `table`/`md` CSV extensions). No new template placeholder vars.
4. Report surfaces are **not** wizard lint targets. Authoring guidance lives in **`wyvern-reporting`** skill (not `creating-wyvern-wizard`).

**Consequences:** Amends ADR-0022 §2 (see below). MCP Phase E still accepts pre-expanded `Command` JSON including `report`. Contract: [xhtml-reporting-contract.md](plans/phase-H/xhtml-reporting-contract.md).

**Amendment to ADR-0022 (Phase H):** Point 2 now reads: extensions produce validated `Command` JSON; **`report` is the only Phase-H-added variant**. No per-extension host handlers; report uses the shared report route family in `wyvern-host`.

---

### ADR-0021: Minimal serde_json in wyvern-viewer for wizard dismiss

**Status:** Accepted (Phase D d.8)

**Context:** OS-close on wizard sessions must `GET /api/wizard/state`, build the full visited stack, and `POST /api/wizard/finish` with a JSON body before process exit (REQ-0097 / d.8). ADR-0011 framed `wyvern-viewer` as wry/winit URL-only with no JSON dependency.

**Decision:** Authorize **minimal-scope** `serde` + `serde_json` in `wyvern-viewer` solely for dismiss JSON (parse wizard state DTO, serialize finish/result bodies). Do **not** import `wyvern-schema` or grow a general HTTP/JSON client stack (`reqwest` remains forbidden). Boundary record: `boundaries/wyvern-viewer/viewer.toml`.

**Consequences:** Viewer stays URL-navigate + dismiss; host remains the JSON schema authority. Further JSON surface in the viewer requires a new ADR amendment.

---

### ADR-0020: Viewport-fit sizing with slack; workspace layout mode

**Status:** Accepted (Phase D d.6)

**Context:** Agent-driven dialogs are high-churn (many unique payloads per day). Fixed pixel tiers and measure-time width caps cause manual resize iteration. Some wizard HTML pages need large viewports (e.g. canvas editors — **HTML-side only**).

**Decision:**

1. **Dialog layout (default):** intrinsic DOM measure + ~25% slack → clamp to available viewport → internal scroll on overflow.
2. **Workspace layout:** optional `page.layout: "workspace"` — opaque passthrough + `wyvern-api.js` sizing. **Not part of the stack model.**

**Consequences:** Wizard Rust code is `WizardSession` + HTTP glue. d.3–d.4 are tests/bootstrap. Viewport sizing (d.6) is separate from stack semantics.

---

### ADR-0016: HTTP dialog host with packaged, pluggable UI

**Status:** Accepted (planning — c.10+)

Ephemeral HTTP server serves packaged UI from disk; any HTTP client may be the viewer; JSON command surface unchanged. Icons, chrome, and templates live in UI files — not in Rust.

**Viewer policy (amendment):** **Interim (c.10–c.14):** omitted `--viewer` defaults to **`none`** (only `none` is implemented). **Product default from c.15:** **`embedded`** (`wyvern-viewer`). CI and headless tests use **`none`** via `WYVERN_VIEWER=none` or explicit flag. Users may select **`system`** or named browsers via **`--viewer <id>`** backed by a **local browser registry** (hardcoded catalog, cached on first run). See [http-viewer-contract.md](plans/phase-C/http-viewer-contract.md).

**Consequences:** Supersedes inline `with_html`, wry IPC, `render_*_html`, REQ-0030/0031 Rust icon catalog, REQ-0080–0087 platform chrome in `wyvern-window`. See [docs/wyvern-host/architecture.md](wyvern-host/architecture.md).

---

### ADR-0017: HTTP transport replaces wry IPC for dialogs

**Status:** Accepted (planning — c.10+)

Dialog pages use `GET /api/dialog` and `POST /api/result`. Authoritative contract: [docs/plans/phase-C/http-dialog-contract.md](plans/phase-C/http-dialog-contract.md). Phase B IPC and [chrome-ipc-contract.md](plans/phase-C/chrome-ipc-contract.md) are **historical** for the deleted `wyvern-window` stack.

**Rust types:** [HTTP-TYPES.md](plans/phase-C/HTTP-TYPES.md).

---

### ADR-0018: Delete → verify → rebuild (no refactor-in-place)

**Status:** Accepted (planning — c.9)

**Context:** Porting `wyvern-window` incrementally leaves dual stacks, feature flags, and agent thrash. Forgetting to delete dead code is harder than deleting first.

**Decision:** c.9 removes the entire `wyvern-window` crate and related assets. Merge gate is **deletion inventory**, not compile. c.10+ rebuilds on `wyvern-host` greenfield.

**Consequences:** Temporary workspace breakage after c.9 is acceptable. No `wyvern-host` code lands before deletion completes. See [c9-deletion.md](plans/phase-C/c9-deletion.md).

---

### ADR-0019: Local browser registry for named `--viewer` targets

**Status:** Accepted (planning — c.15)

**Decision:** Named browsers (`chrome`, `firefox`, …) resolve via a Wyvern-owned cache file built from a hardcoded per-OS catalog on first run / refresh. `system` uses `webbrowser::open`; `embedded` uses **`wyvern` CLI subprocess spawn** of `wyvern-viewer` (not host).

**Consequences:** `wyvern browsers list` / `refresh` CLI; no cross-platform OS browser enumeration API required. See [http-viewer-contract.md](plans/phase-C/http-viewer-contract.md).

---

### Superseded ADRs (wyvern-window — archival only)

The following remain documented under [docs/wyvern-window/architecture.md](wyvern-window/architecture.md) for history. **Do not implement** on the HTTP host path:

| ADR | Topic | Superseded by |
|-----|-------|---------------|
| ADR-0001 | `wry` engine | ADR-0016 — `wry` only in optional `wyvern-viewer` (c.15) |
| ADR-0002 | HTML chrome in webview | ADR-0016 — chrome in packaged `ui/` |
| ADR-0010 | macOS transparent title bar | `wyvern-viewer` lifts platform attrs only (c.15) |
| ADR-0010a | Win/Linux HTML window controls | Packaged `ui/chrome/` + template JS (c.14) |
| ADR-0015 | Icon assets in Rust | REQ-0102/0103 — icons in UI files; no Rust catalog |

---

### ADR-0013: Direct type dispatch — one handler per command

**Status:** Accepted

**Context:**
Wyvern accepts many JSON command shapes over time. A common failure mode is accumulating mode flags, stub handlers, and nested routing that makes it hard to trace JSON input to stdout output.

**Decision:**
After validation, each command becomes a typed `Command` enum variant. Execution is a single `match` (or equivalent) on `type` with one handler function per variant. Handlers return a `CommandResult` serialized to stdout.

**Amendment (HTTP delivery, c.10+):** `wyvern-schema` validates all shipped dialog `type` values regardless of host implementation progress. Types not yet on the `wyvern-host` handler matrix return **`HostError::UnsupportedType` at run time** (after validation passes, before or without completing the dialog). This is not a validation-time phase gate and not a stub handler — the host rejects the command explicitly with stderr `host_error` / exit `6`. Pre-HTTP phases (A–B) may still gate types at validation until the host exists.

**Pipeline (c.15+):**

```
load → validate(value) → Command → host bind → DialogHandle
  → [CLI spawn wyvern-viewer when embedded]
  → [host browser_launch when system/named — inside run() / run_dialog]
  → await_result → CommandResult → stdout
```

Parse is owned by `load`; dispatch is internal to host bind + await. Viewer spawn for **`embedded`** is owned by **`wyvern` CLI** — not `HostSession`. System/named open is owned by **`wyvern-host`**. `wyvern-host::run` covers none/system/named only; embedded uses DialogHandle composition in the CLI.

**Amendment (Phase F / ADR-0022):** The CLI inserts an **argv preprocessor** before `load_command_input`. Host-only flags are stripped; `ExtensionRegistry::match_argv` may expand the remainder to validated `Command` JSON and optional `host.ui_root`. That expanded value then enters this same `validate → host → emit` chain — no new `Command` variants and no per-extension host handlers. See [ADR-0022](#adr-0022-cli-extension-registry-as-argv-preprocessor-phase-f) and the local pipeline note in [`docs/wyvern/architecture.md`](wyvern/architecture.md).

**Amendment (Phase H / ADR-0025):** Phase H adds **`Command::Report`** as the sole new dialog variant after Phase F. Report uses the shared host static-route family (`/report/*`, optional `/api/report/finish`) — not per-extension host handlers. Extension preexec still expands to validated report JSON via `command_from_file`.

**Consequences:**
- Phase A validates and executes only `chrome`
- Each later phase adds one enum variant, one validator module, one handler — not a routing table refactor
- `--interactive` reuses the same `validate → dispatch` path inside the read loop; lifecycle `action` values are a separate small enum, not mixed into dialog `type` routing
- If implementation needs complicated branching to pick a path, treat that as a design smell and simplify before merging
- Each pipeline stage uses a **discriminated union** for errors; re-map to stderr JSON at scope boundaries only — do not merge unlike variants into one generic error type
