## Decision: clean break (locked)

**Delete → verify → rebuild.** No parallel stack, no port-in-place, no “we’ll remove it later.” Refactor baggage from half-deleted code is the primary risk this phase avoids.

| Crate | Fate |
|-------|------|
| `wyvern-window` | **Delete entire crate in c.9** — no port, no rename |
| `wyvern-host` | **New in c.10** — HTTP server + session |
| `wyvern-viewer` | **New in c.15** — optional; opens host URL in minimal webview (`wry`/`winit` URL-only) |

`wry` / `winit` return only via `wyvern-viewer`, never in `wyvern-host`.

---

## (c) Delete in c.9 — no value on HTTP host path

> **Authority:** §c is the **normative deletion inventory** for c.9 QA and `scripts/verify-c9-deletion.sh`. §d below is **non-normative** — rework pointers for implementers only; do not treat §d paths as additional deletion gates.

### Entire crate: `crates/wyvern-window/` (delete directory)

Remove `wyvern-window` from workspace `members` and delete **all** of the following in the **first c.9 commit** (before `wyvern-host` compiles).

#### Rust — `src/` (25 files)

| File | Reason |
|------|--------|
| `crates/wyvern-window/src/lib.rs` | wry entry; replaced by `wyvern-host` |
| `crates/wyvern-window/src/error.rs` | `RunError` event-loop/window-create |
| `crates/wyvern-window/src/window.rs` | winit attrs, modal chrome |
| `crates/wyvern-window/src/run/mod.rs` | `run()` dispatch to winit apps |
| `crates/wyvern-window/src/run/chrome.rs` | ChromeApp + IPC |
| `crates/wyvern-window/src/run/message.rs` | MessageApp + `with_html` |
| `crates/wyvern-window/src/run/input.rs` | InputApp + picker IPC |
| `crates/wyvern-window/src/run/markdown.rs` | MarkdownApp |
| `crates/wyvern-window/src/chrome/mod.rs` | chrome module root |
| `crates/wyvern-window/src/chrome/ipc.rs` | wry IPC parse |
| `crates/wyvern-window/src/chrome/platform.rs` | PlatformChrome matrix |
| `crates/wyvern-window/src/chrome/render.rs` | `include_str!` + placeholders |
| `crates/wyvern-window/src/icons/mod.rs` | `include_bytes!` icon catalog |
| `crates/wyvern-window/src/message/mod.rs` | message render glue |
| `crates/wyvern-window/src/message/render.rs` | `render_message_html` |
| `crates/wyvern-window/src/message/media.rs` | base64/path icon embed |
| `crates/wyvern-window/src/input/mod.rs` | input module |
| `crates/wyvern-window/src/input/render.rs` | `render_input_html` |
| `crates/wyvern-window/src/input/picker.rs` | rfd — **re-home to `wyvern-host` in c.11**, not keep |
| `crates/wyvern-window/src/markdown/mod.rs` | markdown module |
| `crates/wyvern-window/src/markdown/render.rs` | `render_markdown_html` |
| `crates/wyvern-window/src/question/mod.rs` | question module |
| `crates/wyvern-window/src/question/handler.rs` | QuestionApp |
| `crates/wyvern-window/src/question/render.rs` | `render_question_html` |
| `crates/wyvern-window/src/question/sanitize.rs` | ammonia sanitize |

#### Rust — `tests/` (17 files)

| File | Reason |
|------|--------|
| `crates/wyvern-window/tests/support.rs` | GUI flock, blank window helper |
| `crates/wyvern-window/tests/blank_window.rs` | winit smoke |
| `crates/wyvern-window/tests/chrome_ipc.rs` | wry IPC |
| `crates/wyvern-window/tests/chrome_minimize_ipc.rs` | wry IPC |
| `crates/wyvern-window/tests/message_ipc.rs` | wry IPC |
| `crates/wyvern-window/tests/message_minimize_ipc.rs` | wry IPC |
| `crates/wyvern-window/tests/input_ipc.rs` | wry IPC |
| `crates/wyvern-window/tests/input_minimize_ipc.rs` | wry IPC |
| `crates/wyvern-window/tests/input_file_ipc.rs` | wry IPC + picker |
| `crates/wyvern-window/tests/input_file_multi_ipc.rs` | wry IPC + picker |
| `crates/wyvern-window/tests/input_folder_ipc.rs` | wry IPC + picker |
| `crates/wyvern-window/tests/markdown_minimize_ipc.rs` | wry IPC |
| `crates/wyvern-window/tests/question_ipc.rs` | wry IPC |
| `crates/wyvern-window/tests/question_dismiss_ipc.rs` | wry IPC |
| `crates/wyvern-window/tests/question_minimize_ipc.rs` | wry IPC |
| `crates/wyvern-window/tests/question_multi_ipc.rs` | wry IPC |

#### Crate manifest

| File | Reason |
|------|--------|
| `crates/wyvern-window/Cargo.toml` | crate removed |

#### Non-Rust assets under `wyvern-window/` (delete with crate)

- `src/**/template.html` (5) — replaced by `ui/*/index.html`
- `src/markdown/styles.css` — moves to `ui/markdown/` in c.10
- `assets/icons/**` (18 SVG) — icons live in packaged UI, not Rust

#### Boundary

| File | Reason |
|------|--------|
| `boundaries/wyvern-window/window.toml` | delete with crate (c.9) |

#### Archival docs (delete with c.9 or leave until PR merges)

- `docs/wyvern-window/**` — historical only; no new work

---

### `wyvern-schema` — delete (1 file + tests)

| File | Reason |
|------|--------|
| `crates/wyvern-schema/src/icons.rs` | Rust icon catalog (REQ-0030/0031 deprecated) |

**Delete tests inside other files** (not whole files):

| Location | What to remove |
|----------|----------------|
| `crates/wyvern-schema/src/icons.rs` | entire file |
| `crates/wyvern-schema/src/validate/helpers.rs` | `is_named_icon_spec`, `validate_named_icon`, `icons::` import |
| `crates/wyvern-schema/tests/validation_message.rs` | tests: `validation_message_icon_unknown_named_fails`, `validation_message_icon_variant_out_of_range_fails`, `validation_message_icon_non_numeric_variant_fails`, `validation_message_image_unknown_named_fails` (catalog enforcement) |
| `crates/wyvern-schema/tests/validation_input.rs` | parallel unknown/variant icon tests |
| `crates/wyvern-schema/src/validate/tests.rs` | any `NamedIconSpec` / catalog-only cases |

Keep: `icon` / `image` as optional **opaque strings** (path, URL, template hint).

---

### `wyvern` CLI — delete GUI integration tests (c.9)

Remove **`#[serial]` tests that spawn the binary and open a window** from `crates/wyvern/tests/cli_validation.rs`:

| Test function | Reason |
|---------------|--------|
| `cli_valid_chrome_emits_dismissed` | winit path gone |
| `cli_type_message_level_accepted` | spawns GUI |
| `cli_valid_message_emits_dismissed` | spawns GUI |
| `cli_valid_input_emits_dismissed` | spawns GUI |
| `cli_valid_input_file_mode_emits_dismissed` | spawns GUI |
| `cli_valid_markdown_file_emits_dismissed` | spawns GUI |
| `cli_markdown_md_shorthand_emits_dismissed` | spawns GUI |
| `cli_markdown_content_inline_emits_dismissed` | spawns GUI |
| `cli_question_auto_dismiss_emits_req_0068` | spawns GUI |

**Keep** in same file: pure validation/IO tests (no `#[serial]`, no window).

**Delete** `serial_test` dev-dependency from `crates/wyvern/Cargo.toml` if no serial tests remain.

---

### `wyvern` — delete code blocks (not whole files)

| File | Delete |
|------|--------|
| `crates/wyvern/src/error.rs` | `RunError` import/mapping; `is_icon_window_create_message` and icon-specific `WindowCreate` recovery text |
| `crates/wyvern/Cargo.toml` | `wyvern-window` dependency; `serial_test` if unused |

---

### Root workspace — delete deps (when window crate gone)

From root `Cargo.toml` `[workspace.dependencies]` (c.9):

- `wry`
- `winit`
- `pulldown-cmark` (optional in `wyvern-host` from c.12 if server-side `content_html`)

---

### Summary counts (c.9 first commit)

| Category | Count |
|----------|------:|
| `wyvern-window` `.rs` files | 42 |
| `wyvern-window` manifest + boundaries | 2 |
| `wyvern-schema/src/icons.rs` | 1 |
| `wyvern` GUI integration tests | 9 |
| **Total Rust files deleted outright** | **43** (+ 1 schema file) |

---

## (d) Keep vs rework — bottom layer up (non-normative pointers)

> **Non-normative:** §d documents what survives and what to rework in later sprints. Only §c paths are deletion-gated in c.9.

### Layer 0 — `wyvern-schema` (mostly keep; icon cleanup in **c.9**)

| Path | Sprint |
|------|----------|
| `src/command.rs` | **Keep** |
| `src/result.rs` | **Keep** |
| `src/error.rs` | **Keep** |
| `src/button.rs` | **Keep** |
| `src/chrome.rs` | **Keep** |
| `src/error_code.rs` | **Keep** — add host error codes if needed |
| `src/field_name.rs` | **Keep** |
| `src/stderr.rs` | **Keep** |
| `src/lib.rs` | **Rework** — drop `icons` module + `NamedIconSpec` re-export |
| `src/validate/mod.rs` | **Keep** |
| `src/validate/helpers.rs` | **Rework** — remove named-icon catalog validation |
| `src/validate/message.rs` | **Rework** — `icon`/`image` opaque string only |
| `src/validate/input.rs` | **Rework** — same |
| `src/validate/markdown.rs` | **Keep** |
| `src/validate/question.rs` | **Keep** |
| `src/validate/chrome.rs` | **Keep** |
| `src/validate/tests.rs` | **Rework** — drop catalog cases |
| `tests/validation_*.rs` | **Rework** — drop catalog-fail tests; keep shape tests |
| `tests/question_contract_examples.rs` | **Keep** |

---

### Layer 1 — `wyvern-host` (new, **c.10**)

Greenfield. No port from `wyvern-window`.

| Path | c.10 action |
|------|----------|
| `src/lib.rs` | **Add** — `pub fn run(command, options) -> Result<CommandResult, HostError>` |
| `src/error.rs` | **Add** |
| `src/server.rs` | **Add** |
| `src/session.rs` | **Add** |
| `src/static_files.rs` | **Add** |
| `src/routes/dialog.rs` | **Add** — `GET /api/dialog` |
| `src/routes/result.rs` | **Add** — `POST /api/result` |
| `tests/http_message.rs` | **Add** — no winit |
| `Cargo.toml` | **Add** — axum, tokio, tower-http |

**c.10 behavior:** `run()` handles `Command::Message` only. All other variants → `HostError::UnsupportedType` at run time (validation passes) until c.11–c.14.

---

### Layer 2 — `wyvern` CLI (rework run wiring, **c.10**)

| Path | c.10 action |
|------|----------|
| `src/input.rs` | **Keep** — load stdin/file/md |
| `src/observability.rs` | **Keep** — rename log `window_open` → `host_start` when touched |
| `src/pipeline.rs` | **Rework** — `wyvern_host::run`; markdown file load unchanged |
| `src/error.rs` | **Rework** — `HostError` mapping; drop `RunError` |
| `src/main.rs` | **Rework** — parse `--bind`, `--ui-root`, `--viewer`; c.10 implements `none` only |
| `src/lib.rs` | **Keep** exports |
| `tests/cli_validation.rs` | **Rework** — HTTP client tests for `message`; keep validation tests |

---

### Layer 3 — `wyvern-wizard` (untouched until Phase D)

| Path | c.10 action |
|------|----------|
| `src/lib.rs` | **Keep** — pure state; Phase D wires to host |

---

### Layer 4 — `wyvern-mcp` (stub compile, **c.10**)

| Path | c.10 action |
|------|----------|
| `Cargo.toml` | **Rework** — drop `wyvern-window` dep; keep `wyvern-schema` only (or no deps) so workspace builds |
| `src/lib.rs` | **Keep** empty stub — Phase E rewires to `wyvern-host` |

No MCP behavior until Phase E; stub exists so `cargo build --workspace` stays green from c.10.

---

### Layer 5 — packaged UI (new, not Rust)

| Path | Sprint |
|------|--------|
| `ui/message/*` | **c.10** |
| `ui/shared/wyvern-api.js` | **c.10** |
| `ui/input/*` | **c.11** |
| `ui/markdown/*` | **c.12** |
| `ui/question/*` | **c.13** |
| `ui/chrome/*` | **c.14** |

---

### Layer 6 — repo infra

| Path | Sprint |
|------|--------|
| `Cargo.toml` — remove `wyvern-window` | **c.9** |
| `Cargo.toml` — add `wyvern-host` | **c.10** |
| `boundaries/`, `ci.yml` HTTP tests | **c.10** (baseline); extend per type sprint |

---

## Rework sequence (by sprint)

```text
c.9   DELETE  — wyvern-window tree + icons.rs + GUI cli tests (§c); compile optional
c.10  HOST    — wyvern-host skeleton + message + ui/message/ + workspace green
c.11  INPUT   — ui/input/ + picker routes
c.12  MARKDOWN— ui/markdown/ + content_html helper
c.13  QUESTION— ui/question/
c.14  CHROME  — ui/chrome/; full dialog matrix closed
c.15  VIEWER  — wyvern-viewer + browser registry
c.16  RELEASE — tarball + v0.1.0 tag
```

**c.9 merge rule:** deletion inventory passes; **`cargo build --workspace` not required**.

**c.10+ merge rule:** `cargo build --workspace` + `cargo test --workspace` green.

Do **not** add `wyvern-host` before c.9 DELETE completes.

---

## c.11–c.14 — one dialog type per sprint

| Sprint | Type | Doc |
|--------|------|-----|
| c.11 | `input` (+ picker) | [c11-host-input.md](c11-host-input.md) |
| c.12 | `markdown` | [c12-host-markdown.md](c12-host-markdown.md) |
| c.13 | `question` | [c13-host-question.md](c13-host-question.md) |
| c.14 | `chrome` | [c14-host-chrome.md](c14-host-chrome.md) |

Each sprint: one type, one Playwright spec, narrow QA fence. Fast if scope stays tight.

---

## QA scope fence (c.9)

**In review:** deletion paths in §c only.

**Must not exist post c.9 PR:** any path under `crates/wyvern-window/`.

**c.10+ QA:** per-type sprint doc + prior types regression.
