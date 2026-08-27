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

This is a real, minimal gap between the sprint doc's illustrative
`wyvern --picker page.html < input.json > output.json` sketch and Wyvern
v0.5.0's actual CLI surface; it does not block using Wyvern as an optional
picker, but the atm-core adapter script (`scripts/send-to/atm-send-to.sh`)
needs a small `wizard.json`-generation + `.data`-unwrap step (or an
equivalent `--picker` convenience flag added here) before this page can be
wired in as the real optional picker rather than only demonstrated by this
example. Tracked from the atm-core side in
`docs/plans/phase-aq/validation-evidence.md`.
