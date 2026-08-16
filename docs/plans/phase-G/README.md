# Phase G — CLI Extension Agent Usability (`integrate/phase-G`)

Phase G implementation PRs target **`integrate/phase-G`**. Sprint docs (`g.1`–`g.3`) are the **sole authority** for deliverables, acceptance criteria, and validation. `docs/plans/project-plan.md` carries phase-level goals only.

**Ordering:** Phase G runs **after Phase F** and **before Phase E**. Phase E agents benefit from discoverable extension help, skill catalog JSON, and error-teaches recovery on argv near-misses.

## Input artifact

Scope derives from the Phase F post-ship review:

- [phase-F-usability-review.md](../phase-F/phase-F-usability-review.md) — agent-usability score 2/5; P0/P1 recommendations

Phase F shipped the extension **engine**; Phase G makes the CLI **speak skill** so a cold agent needs no checkout docs.

## Core model (unchanged engine)

```
argv → match extension → optional preexec → template expand → validate → pipeline → host
```

Phase G adds **surfaces** around that path — no new dialog types, no new host behavior:

```
host flags stripped → global/extension help → match_with_diagnostics → expand → pipeline
                      → near-miss diagnostics (g.2) on no match
```

| Surface | Phase G adds |
|---------|----------------|
| `--help` / `-h` | First-class, exit 0; lists every shipped skill with copy-paste examples |
| Extension prefix `--help` | Skill card at **match time** (before requires skip; no suffix path required) |
| Fallthrough errors | Unknown suffix, incomplete prefix, skipped `requires` name the skill |
| `extensions list` | Skill index (text + `--json`); optional `show <id>` |
| Preexec failures | Child stderr in JSON envelope; spawn vs exit vs missing-file recovery |

Registry remains declarative in `share/wyvern/extensions.json`. Phase G may add optional **`description`** and **`examples`** fields to the schema for catalog output.

## Phase goal

An AI agent with **zero prior documentation** can discover every Phase F extension invocation, recover from near-misses, and inspect one skill — using only `wyvern --help`, `wyvern extensions list`, and stderr JSON.

**Target agent-usability score:** 4 / 5 (per review rubric in phase-F-usability-review.md).

## Phase acceptance (smoke)

```bash
# Help lists CSV, table, md, compose — exit 0
wyvern --help
wyvern -h

# Extension-local help — exit 0, not UnexpectedArg
wyvern compose render --help

# Skill catalog — machine-readable JSON array
wyvern extensions list --json | jq 'length >= 7'

# Near-miss teaches next command — not PARSE_ERROR "not valid JSON"
wyvern notes.txt
PATH=/usr/bin wyvern sample.csv    # when python3 absent: names csv-suffix skip + install hint

# Preexec failure — structured recovery, not "install binaries" when binary ran
wyvern md /nonexistent/file.csv
```

## Sprint map

| Sprint | Adds | Touches |
|--------|------|---------|
| **g.1** | Help surface — global `--help`, `wyvern help`, enriched usage, extension `--help` skill cards, built-in subcommand `--help` | `main.rs`, `cli_args.rs`, `expand.rs`, `list.rs`, `browsers` cmd |
| **g.2** | Error-teaches — skipped-requires diagnostics, unknown suffix / incomplete prefix, MissingArgs/UnexpectedArg caller recovery, preexec stderr capture | `mod.rs`, `main.rs`, `error.rs`, `preexec.rs`, `expand.rs` |
| **g.3** | Skill catalog — rich list text, `--json`, `extensions show <id>`, registry `description`/`examples` | `list.rs`, `extensions/mod.rs`, `extensions.json` |

## What Phase G does not close

- `--interactive` argv expansion — **Phase E**
- MCP tool wrappers — **Phase E**
- User registry (`~/.config/wyvern/extensions.json`) — post-G
- `wyvern skills` argv alias — P2; defer unless trivial
- `extensions dump` / `--raw` merged registry — P2; defer unless g.3 has capacity
- README quickstart CSV lines — docs track; optional g.3 non-closure note

## Boundaries

- All changes stay in **`wyvern` CLI crate** (`crates/wyvern/src/**`) plus `share/wyvern/extensions.json` schema fields
- No new `Command` enum variants
- No new `ErrorCode` variants in `wyvern-schema` (near-misses reuse `ParseError` / `ValidationError` with new message text)
- No `wyvern-host` behavior changes
- Principal requirements: [REQ-0134–REQ-0137](../../wyvern/requirements.md) (agent CLI surfaces); amended REQ-0130, REQ-0132
- ADR-0022 Phase G amendment in [docs/architecture.md](../../architecture.md) — registry/help parity is a merge gate for new extensions

## Sprint index

| Sprint | Doc |
|--------|-----|
| g.1 | [g1-help-surface.md](g1-help-surface.md) |
| g.2 | [g2-error-teaches.md](g2-error-teaches.md) |
| g.3 | [g3-skill-catalog.md](g3-skill-catalog.md) |

## Contract reference

- Extension match/expand semantics: [cli-extensions-contract.md](../phase-F/cli-extensions-contract.md)
- Phase G amendment: [agent-usability-contract.md](agent-usability-contract.md)
- Skills catalog JSON/text: [skills-catalog-contract.md](skills-catalog-contract.md) (g.3)
- Usability findings and rubric: [phase-F-usability-review.md](../phase-F/phase-F-usability-review.md)

## Plan hardening

Round table: [plan-hardening-rounds.md](plan-hardening-rounds.md) (populated by `/plan-hardening` runs)
