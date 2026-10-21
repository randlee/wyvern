# Issue closeout evidence — k.1 (#115, #139) and k.2 (#141)

k.1 records #115 and #139. Those issues were already closed; k.1 does not
re-close them. It records the test or job that keeps each checklist row true,
and the sprint PR CI run that executed those jobs. k.2 records the #141
checksum policy at the end of this file.

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

## #141 — `checksums.txt` (k.2)

**Close policy: wontfix.** Do not publish `checksums.txt` on the v0.5.0 GitHub Release.

Forward kit releases already satisfy the request. v0.6.0 and v0.7.0 each include `checksums.txt` covering every kit binary archive. The consumer contract is [`docs/release-checksums.md`](../../release-checksums.md) § Contract.

### atm-core legacy pin check

Inspected `https://github.com/randlee/atm-core` at `a9d0efcd37d2a5946086e98be75a2e8780837d2b` (2026-10-20) and the local Send-To pin [`release/atm-core-send-to-pin.toml`](../../../release/atm-core-send-to-pin.toml).

| Source | Legacy names (`wyvern-windows.zip`, `wyvern-linux.tar.gz`, `wyvern-macos-aarch64.tar.gz`, `wyvern-macos-x86_64.tar.gz`) |
|--------|------------------------------------------------------------------------------------------------------------------------|
| `tools/bootstrap.toml` `[wyvern]` | Absent. Pin is version `0.6.0` with kit filenames only. The four pinned SHA-256 values equal the v0.6.0 `checksums.txt` lines. |
| `tools/bootstrap.py` `wyvern_asset_name` | Present only in the `version == "0.5.0"` branch. |
| `release/atm-core-send-to-pin.toml` | Absent. `wyvern_pin = 0.6.0`. The file names Send-To fixtures, not release archives. |

v0.5.0 assets are those four legacy names and the release has no `checksums.txt`. atm-core warns and continues when `checksums.txt` is missing, after the pinned archive hash matches. No current pin requests the legacy names.

### Why wontfix

1. The live atm-core bootstrap pin is Wyvern 0.6.0 kit assets, and those assets already have a matching `checksums.txt`.
2. Legacy filenames are selected only when the pin version is exactly `0.5.0`. The bootstrap pin and the Send-To pin both use `0.6.0`.
3. A missing `checksums.txt` on v0.5.0 is the existing non-fatal path. There is no current consumer outage.
4. v0.6.0 and v0.7.0 are the releases this contract covers. v0.5.0 stays as published.

### v0.7.0 `checksums.txt`

```text
51bb9c96c81a7da0abc282f65187d2a9c18b65cccf3bbead6d15417e6c9e214f  wyvern_0.7.0_aarch64-apple-darwin.tar.gz
7f16ad1fa1e8e412307ee55c019594032b4b22f6b6a6d4911adf0e60eddd6b8b  wyvern_0.7.0_x86_64-apple-darwin.tar.gz
92235dd9097890d1ffabcff4b33a4d19fafc231165668ceb961f05e1593f486f  wyvern_0.7.0_x86_64-unknown-linux-gnu.tar.gz
a4e94411b9a21acd7431ff94231d1974179cfd8852e7641c972ac8a4117d5d17  wyvern_0.7.0_x86_64-pc-windows-msvc.zip
```

## #62 / #63 — crates.io README and Documentation (k.3)

| Item | Status |
|------|--------|
| Manifest + README fixes | Shipped on `integrate/phase-K` in PR #172 (workspace **0.7.1**) |
| GitHub issue state | **OPEN** until **v0.7.1** tag + `crates-publish.yml` completes |
| Close policy | Close with crates.io spot-check URLs for `wyvern-cli` and `wyvern-schema` after publish |
| Tag note | Tag **v0.7.1** at merge commit `3b9f498` (k.3), not at k.4 **0.8.0** tip |
