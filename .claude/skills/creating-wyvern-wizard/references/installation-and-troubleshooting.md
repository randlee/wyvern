# Installation and troubleshooting (wyvern CLI)

Load from the Layer 0 router when **G1** or Step 1 fails because `wyvern` is
missing, wrong version, or not the binary you expect. Do not skip verify — schema
and lint commands must come from a build that includes wizard lint support.

## Check first

```bash
which wyvern && wyvern --version
```

If `which` prints nothing or `--version` errors, install or fix PATH before
schema validate or `wyvern wizard lint`.

## Find an existing install / PATH

- `command -v wyvern` — first match on `PATH` (may be a system install, not your worktree build).
- `type -a wyvern` — all `wyvern` binaries visible in the current shell.
- Compare the path to your repo: a worktree build is usually under `target/release/wyvern` or `target/debug/wyvern` after `cargo build`.

**Common mistake:** the shell finds `/usr/local/bin/wyvern` or `~/.cargo/bin/wyvern` while you edited the crate in a git worktree. Prepend the worktree `target/release` directory to `PATH`, or use the absolute path to that binary for G1/G4 commands.

## Install (macOS primary)

From a clone of the Wyvern repo (this monorepo):

```bash
cargo install --path crates/wyvern
```

Or install from the repo root after pulling the branch that contains wizard lint:

```bash
cargo install --path crates/wyvern --locked
```

For a one-off local build without installing globally:

```bash
cargo build --release -p wyvern
export PATH="$(pwd)/target/release:$PATH"
```

## Minimum version

Wizard authoring gates assume a CLI that supports:

- G1: `wyvern path/to/wizard.json --viewer none`
- G4: `wyvern wizard lint path/to/wizard-dir`

**Wizard lint** ships on `feature/phase-G-wizard-lint` (or later on `integrate/phase-G`). If `wyvern wizard lint` is unknown or exits with usage errors, build from that branch or merge locally — do not substitute a different linter.

## Validation command

After install, re-run Step 1:

```bash
which wyvern && wyvern --version
wyvern wizard --help
wyvern wizard lint --help
```

Then proceed to G1/G4 per [validation-and-lint.md](core/validation-and-lint.md).

## Known issues

| Symptom | Likely cause | Fix |
|---------|----------------|-----|
| `wyvern: command not found` | Not on PATH | `cargo install --path crates/wyvern` or add `target/release` to PATH |
| Old behavior / missing `wizard lint` | System binary ahead of dev build | `type -a wyvern`; use worktree binary or reinstall |
| Schema passes in CI but fails locally | Different `wyvern` on PATH | Same `which wyvern` in both environments |
| Worktree vs main repo confusion | Built in one tree, run from another | Build in the tree you author from; export PATH to that `target/release` |

Do not patch around a missing CLI by skipping G1 or G4.
