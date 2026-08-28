# Winget bootstrap manifests (j.2 AC #9)

One-time submission to establish `randlee.wyvern` in `microsoft/winget-pkgs`
**before** j.3 automated `winget-publish.yml` legs can succeed.

Uses the **legacy** v0.5.0 asset `wyvern-windows.zip` (root-level `wyvern.exe`).
Kit releases from j.3 onward use `wyvern_<version>_x86_64-pc-windows-msvc.zip`
with `bin/wyvern.exe`; subsequent versions are handled by the kit workflow.

## Submit

1. Fork `microsoft/winget-pkgs` using `WINGET_GITHUB_TOKEN` (see
   [docs/WINGET_SETUP.md](../../docs/WINGET_SETUP.md)).
2. Copy `0.5.0/` to `manifests/r/randlee/wyvern/0.5.0/` in your fork.
3. Open PR against `microsoft/winget-pkgs` master.
4. After merge, record PR URL in
   [j2-closeout-audit.md](../../docs/plans/phase-J/.plan-hardening/j2-closeout-audit.md)
   and close j.2 AC #9.

## Verify

```bash
gh api "repos/microsoft/winget-pkgs/contents/manifests/r/randlee/wyvern/0.5.0"
```
