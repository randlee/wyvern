# Waiver and upstream disposition (j.2 closeout)

Each blocker is exactly one state:

| Blocker | State | PR link | Signed waiver | Effect |
|---------|-------|---------|---------------|--------|
| CR-001 Linux webview deps | open | | | j.3/j.4 **not startable** until resolved |
| CR-002 Homebrew/Scoop renderer | open | | | j.3/j.4 **not startable** until resolved |

**Rules:**

- `resolved` ⇒ merged sc-publish (or wyvern) PR link recorded; j.3 may proceed.
- `waived` ⇒ **blocks j.3 and j.4 entirely** (no re-sign escape). Phase J pauses until resolved.
- CR-002 resolved does **not** unblock Homebrew while CR-001 is still `open`.

Updated in j.2 sprint closeout.
