# wyvern v0.7.0 — sc-lint boundaries, observability v2, atm-core Send-To picker

**Released:** October 8, 2026 · **Install:** `cargo install wyvern-cli --version 0.7.0` (Rust), or download native binaries for macOS, Windows, Linux from [releases](https://github.com/randlee/wyvern/releases)

[Changelog](https://github.com/randlee/wyvern/blob/main/CHANGELOG.md) · [Release notes](https://github.com/randlee/wyvern/releases/tag/v0.7.0)

---

## Agent Orchestrator (Send-To / atm-core)

**As an agent orchestrator wiring atm-core Send-To, I want a documented Wyvern picker contract with CI that catches wire drift, so optional native pickers stay aligned with atm-core releases.**

v0.7.0 ships the **`atm-pick-member`** wizard example — the canonical HTML/JS page atm-core vendors for optional Wyvern pickers — plus CI that pins **atm-core v1.6.1** fixtures, hash-checks contract bytes, runs atm-core Send-To surface tests, and probes a real `wyvern` build. The CLI surface is unchanged: PickerInput still travels as wizard `config`, and callers read `WizardResult.data` as PickerOutput.

---

## Maintainer / release engineer

**As a maintainer, I want sc-lint boundary enforcement and kit-managed releases to stay green, so quality gates match the rest of the org.**

This minor release lands **sc-lint 0.5.0** across Wyvern boundaries and CI (including consumer bootstrap lanes and extended analyzers) and migrates the binary logger to **sc-observability v2**. No breaking changes to dialog JSON schemas or host IPC for integrators on 0.6.x.

---

## What's Next

Follow kit release flow on `main` after merge: `release-candidate-v0.7.0` → preflight → production dispatch. Advance `release/atm-core-send-to-pin.toml` together with any PickerInput/PickerOutput schema change.
