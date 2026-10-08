# j.3 RC runbook (post PR #148)

**Blocked on:** [PR #148](https://github.com/randlee/wyvern/pull/148) merge (1 approving review; CI green).

Wyvern default branch is `main`; GitHub only registers `workflow_dispatch` workflows on the default branch. PR #148 copies `release-candidate.yml` to `main` so the RC can be dispatched against `develop`.

## After #148 merges

```bash
# 1. RC cut (tags origin/develop)
gh workflow run "Release Candidate" --repo randlee/wyvern --ref develop -f version=0.6.0
gh run list --repo randlee/wyvern --workflow "Release Candidate" --limit 1

# 2. Release branch from candidate tag
git fetch origin release-candidate-v0.6.0
git checkout -b release/v0.6.0 release-candidate-v0.6.0
# … version lockstep commits if needed …
git push origin release/v0.6.0
gh pr create --base main --head release/v0.6.0 --title "release: v0.6.0"

# 3. After merge to main
gh workflow run release-preflight.yml --repo randlee/wyvern --ref main -f tag=v0.6.0
gh workflow run release.yml --repo randlee/wyvern --ref main -f tag=v0.6.0 -f target=production

# 4. Post-release legs (per channel-dispatch-plan)
gh workflow run homebrew-publish.yml --repo randlee/wyvern --ref main -f tag=v0.6.0
gh workflow run scoop-publish.yml --repo randlee/wyvern --ref main -f tag=v0.6.0
gh workflow run winget-publish.yml --repo randlee/wyvern --ref main -f tag=v0.6.0
gh workflow run crates-publish.yml --repo randlee/wyvern --ref main -f tag=v0.6.0
```

Record run IDs in [first-release-record.md](first-release-record.md).

## Alternative (org policy)

Set repo default branch to `develop` (atm-core model) — requires admin on `randlee/wyvern`. Then #148 is unnecessary.
