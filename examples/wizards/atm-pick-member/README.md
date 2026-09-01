# atm-pick-member wizard example

Reference implementation of the `PickerInput`/`PickerOutput` contract that
[`atm-core`'s Send-To feature](https://github.com/randlee/atm-core/blob/develop/docs/plans/phase-aq/fixtures/wyvern-pick-member-contract.md)
uses Wyvern as an optional native picker for. Renders a roster grouped by
team, disables (not just greys) `idle`/`dead` members so they cannot be
selected, and returns the chosen recipient ids plus an optional note.

## Try it

```sh
wyvern wizard.json
```

Select a team member (only `active` rows are selectable — `idle`/`dead` rows
render as disabled checkboxes with their status), optionally add a note, and
click **Finish**.

## Real Wyvern invocation (important -- differs from the atm-core sprint doc's illustrative sketch)

`wyvern` has no `--picker <path>` flag, and its own stdin is reserved for
loading a bare `Command` JSON *only when no positional argument is given*
(`crates/wyvern/src/input.rs::load_command_input`). There is currently no
mechanism for a caller to both name a positional page/wizard file *and* pipe
arbitrary payload bytes to that page's process stdin.

The working integration point is the wizard command's `config` field
(`"Opaque wizard-wide config, never inspected by the host"` --
`WizardCommand::config`, `crates/wyvern-schema/src/wizard.rs`), fetched by
the page via `window.wyvern.config` once `wyvernWizardState()` resolves. A
caller that wants to hand this page a real, dynamic roster must therefore
generate a small `wizard.json` at invocation time --
`{"type":"wizard","page":{"id":"pick-member","title":"ATM Send-To","html":"pick-member.html"},"config":<PickerInput JSON>}`
-- and run `wyvern <generated-wizard.json> --ui-root <dir containing
pick-member.html>`, rather than piping to `wyvern pick-member.html` directly.

Symmetrically, the wizard's terminal stdout is the full `WizardResult`
envelope -- `{"button":"finish","data":<PickerOutput>,"stack":[...]}` -- not
a bare `PickerOutput` object. A caller must read `.data` (and treat
`button !== "finish"` as a cancel, matching the PRD's "cancel -> nonzero
exit, no stdout" contract at the picker-selection level).

This was a real, minimal gap between the sprint doc's illustrative
`wyvern --picker page.html < input.json > output.json` sketch and Wyvern
v0.5.0's actual CLI surface -- **now closed**: the atm-core adapter scripts
(`scripts/send-to/atm-send-to.sh` and `.ps1`) generate the `wizard.json` and
unwrap `.data` exactly as described above, and were run end to end against
a real `wyvern` build from this PR (see
`docs/plans/phase-aq/evidence/AQ5/wyvern-real-invocation-local.md` in
atm-core for the transcript). This page is Wyvern's real optional picker for
atm-core's Send-To feature, not only demonstrated by this example.

## Source of truth by reference (atm-core)

Per [atm-core issue #139](https://github.com/randlee/atm-core/issues/139)
(mirrors the comment there): the precise schema requirements live in
atm-core as the canonical files below. Wyvern-side integration tests/CI
should reference these by pinning an atm-core ref and hash-checking the
fetched fixtures, not maintain a divergent copy.

atm-core (branch `feature/aq-5-surface-evidence`, head
`8cb881dfe99879db3cc09bf089757bc947cbb523`; moves to `integrate/phase-aq` ->
`develop` at Phase AQ closeout):

| What | Path in atm-core |
|---|---|
| Contract doc (authoritative, incl. the invocation-shape correction) | `docs/plans/phase-aq/fixtures/wyvern-pick-member-contract.md` |
| PickerInput v1 canonical bytes | `docs/plans/phase-aq/fixtures/picker-input-v1.json` |
| PickerOutput v1 canonical bytes | `docs/plans/phase-aq/fixtures/picker-output-v1.json` |
| Unknown-schema rejection fixture | `docs/plans/phase-aq/fixtures/picker-output-unknown-schema.json` |
| Adapter (macOS/Linux) -- generates `wizard.json`, invokes Wyvern, unwraps `WizardResult.data` | `scripts/send-to/atm-send-to.sh` |
| Adapter (Windows) | `scripts/send-to/atm-send-to.ps1` |
| Vendored `pick-member.html` asset (byte-identical copy of this PR's page, used as the `--ui-root` asset) | `scripts/send-to/pick-member.html` |
| Bounded version/contract probe (1.5 s deadline) | `scripts/send-to/probe_wyvern.py` |
| Six-case degradation harness (absent / below-pin / unparsable / hang / missing-asset / unknown-schema) | `scripts/phase-aq/run_aq5_wyvern_degradation_evidence.py` |
| Wyvern-degradation + generated-`wizard.json`-shape tests | `.just/tests/test_send_to_surface.py` (`test_wyvern_degradation_cases_fall_back_and_still_send`, `test_generated_wizard_json_matches_contract_shape`) |
| R4 dead/idle-exclusion contract tests (all four picker adapters; not Wyvern-degradation-specific) | `.just/tests/test_picker_exclusion.py` |
| Real end-to-end local transcript against this PR's build | `docs/plans/phase-aq/evidence/AQ5/wyvern-real-invocation-local.md` |
| Pinned Wyvern version | the literal `"0.5.0"` in both `scripts/send-to/atm-send-to.sh` (`WYVERN_PIN`) and `.ps1` (`$wyvernPin`) -- two copies kept in sync by convention/AQ6 preflight, not one shared constant |

### Corrected process contract (real Wyvern, verified against v0.5.0)

- There is **no** `wyvern --picker <page>` flag. The picker runs as a
  wizard: the adapter writes `wizard.json` with `config` = PickerInput and
  runs it; Wyvern's terminal stdout is the full `WizardResult` envelope
  `{"button":"finish","data":<PickerOutput>,"stack":[...]}`; the adapter
  reads `.data`.
- Cancel = non-`finish` button / nonzero exit with no PickerOutput;
  diagnostics on stderr only; `--version` parseable `MAJOR.MINOR.PATCH`
  within the 1.5 s probe.
- idle/dead members must be rendered `disabled` (non-routable), not merely
  styled; unknown `schema_version` is rejected, never guessed.

### Requested Wyvern CI shape

1. Keep this PR's Playwright L2 test
   (`tests/l2/wizard-atm-pick-member.spec.ts`) as the page-level contract
   test.
2. Add an integration job that checks out atm-core at the pinned ref, runs
   `scripts/phase-aq/run_aq5_wyvern_degradation_evidence.py` and
   `.just/tests/test_send_to_surface.py` against the **real** Wyvern build
   on `PATH`, and fails on any wire drift.
3. When the contract evolves, bump `schema_version` and the atm-core ref
   together; the hash check makes silent widening impossible.
