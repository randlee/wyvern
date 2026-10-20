# Issue closeout evidence — k.1 (#115, #139)

Issues are already closed. This sprint does not re-close them. It records the
test or job that keeps each checklist row true, and the sprint PR CI run that
executed those jobs.

Pin for every #139 row: [`release/atm-core-send-to-pin.toml`](../../../release/atm-core-send-to-pin.toml)

| Field | Value |
|-------|-------|
| repository | `https://github.com/randlee/atm-core` |
| tag | `v1.6.1` |
| revision | `1272d4c8a0256982fa550e592f77771e6d5a8bb2` |
| wyvern_pin | `0.6.0` |

CI job: **`atm-send-to-contract`** in [`.github/workflows/ci.yml`](../../../.github/workflows/ci.yml).

| Step | Script |
|------|--------|
| Verify atm-core pin, fixtures, and picker HTML | `scripts/verify_atm_core_send_to_contract.sh` (exports `ATM_CORE_CHECKOUT`) |
| Run atm-core Send-To surface tests and real Wyvern probe | `scripts/run_atm_core_send_to_surface_tests.sh` (reuses `ATM_CORE_CHECKOUT`) |
| Run atm-pick-member L2 contract spec | `npx playwright test wizard-atm-pick-member.spec.ts` |

The surface-test step runs, against the pinned checkout:

- `python3 -m unittest discover` of `.just/tests/test_send_to_surface.py`
- `scripts/phase-aq/run_aq5_wyvern_degradation_evidence.py`
- `scripts/send-to/probe_wyvern.py` (`PROBE_SECONDS = 1.5`)

#115 is covered by the workspace **`build`** job step **`cargo test`**
(`cargo test --workspace`), which runs integration test target
`extensions_xhtml_suffix`.

## Sprint PR CI run

- Workflow run: `https://github.com/randlee/wyvern/actions/runs/39388028136`
- #115 job on that run: `build` (`cargo test`)
- #139 job on that run: `atm-send-to-contract`

## #115 — `.xhtml` suffix

Closed by PR [#119](https://github.com/randlee/wyvern/pull/119). Regression
fixture: `fixtures/xhtml/panel.xhtml`.

| Checklist row | Test or job verified this sprint | CI run |
|---------------|----------------------------------|--------|
| Shipped `xhtml-suffix` matches a repo `.xhtml` fixture | `extensions_xhtml_suffix_matches_fixture` | `https://github.com/randlee/wyvern/actions/runs/39388028136` (`build` / `cargo test`) |
| Expanded command JSON includes `"type":"report"` (no `PARSE_ERROR`, no "No shipped extension matches") | `extensions_xhtml_suffix_expand_command_is_report` | `https://github.com/randlee/wyvern/actions/runs/39388028136` (`build` / `cargo test`) |
| `WYVERN_VIEWER=none wyvern <fixture>.xhtml` exits 0 on the headless report path | `extensions_xhtml_suffix_headless_viewer_none_exits_zero` | `https://github.com/randlee/wyvern/actions/runs/39388028136` (`build` / `cargo test`) |

## #139 — ATM Send-To picker contract

Closed by PR [#140](https://github.com/randlee/wyvern/pull/140). Checklist rows
are the coverage list in the issue body. Wyvern's real stdout is the wizard
`WizardResult` envelope; `PickerOutput` is `.data`. That correction is the
contract pinned in `wyvern-pick-member-contract.md` and checked by the job below.

| Checklist row | `atm-send-to-contract` step | Pinned evidence |
|---------------|-----------------------------|-----------------|
| Confirmation | L2 `atm-pick-member disables dead/idle rows and returns only the active recipient` (Finish → `.data.schema_version` 1, one active recipient, note). Surface `SendToSurfaceTests::test_multiple_files_and_recipients_reach_one_final_send` and `test_reference_picker_emits_versioned_output`. | `picker-output-v1.json` sha256 in the pin; `wyvern_pin = 0.6.0` |
| Cancel (nonzero, no completed send) | Surface `SendToSurfaceTests::test_cancel_exits_without_invoking_send` (nonzero exit, send log absent) | same revision `1272d4c8a0256982fa550e592f77771e6d5a8bb2` |
| Unknown schema rejection | L2 `atm-pick-member rejects an unrecognized PickerInput schema_version`. Surface `test_wyvern_degradation_cases_fall_back_and_still_send` case `unknown-schema`, plus AQ5 `run_aq5_wyvern_degradation_evidence.py` case `unknown-schema`. Verify step hashes `picker-output-unknown-schema.json`. | pin fixture sha256 `3851036fb52a924f09e94dc3f5c6636e84e71b2b3a67b880f1cd1dddaf38f162` |
| Exact stdout shape | L2 confirmation asserts `.data` equals `{schema_version, recipients, note}`. Surface `test_reference_picker_emits_versioned_output` and `test_generated_wizard_json_matches_contract_shape` (`config` is the `PickerInput`, page id `pick-member`). Verify step requires `examples/wizards/atm-pick-member/wizard.json` `config` to match `picker-input-v1.json`. | pin fixtures `picker-input-v1.json`, `picker-output-v1.json` |
| Stderr-only diagnostics | Surface `test_reference_picker_emits_versioned_output` (exclusion text on stderr, versioned JSON on stdout) and `test_command_wrapper_propagates_exit_code_and_forwards_stderr_on_failure` (failure detail remains on stderr) | pin revision above; scripts `scripts/send-to/atm-send-to.sh` / `.ps1` |
| Version probe (`MAJOR.MINOR.PATCH`, bounded) | Surface step `probe_wyvern.py --pin` against the built `wyvern` (`PROBE_SECONDS = 1.5`). Verify step checks `WYVERN_PIN` in `atm-send-to.sh` equals pin `wyvern_pin`. | `wyvern_pin = 0.6.0` |
| Cold-start budget | Same probe step. The pinned gate is the 1.5s `--version` deadline in `probe_wyvern.py`, not a separate launch-to-interactive timer. | `tag = v1.6.1`, revision above |

Fixture bytes checked by the verify step (paths relative to the atm-core pin):

| Pin path | Role |
|----------|------|
| `docs/plans/phase-aq/fixtures/picker-input-v1.json` | stdin `PickerInput` bytes |
| `docs/plans/phase-aq/fixtures/picker-output-v1.json` | confirmation `PickerOutput` bytes |
| `docs/plans/phase-aq/fixtures/picker-output-unknown-schema.json` | unknown-schema fixture |
| `docs/plans/phase-aq/fixtures/wyvern-pick-member-contract.md` | invocation-shape contract |
| `scripts/send-to/pick-member.html` | vendored page must match `examples/wizards/atm-pick-member/pages/pick-member.html` |
