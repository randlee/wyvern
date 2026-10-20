# Version policy

## Phase K decision (sprint k.3)

**Ship patch 0.7.1.** Do not defer the crates.io README and documentation-link fix to **0.8.0**.

#62 and #63 are packaging metadata only. A patch republish puts a non-empty README and a docs.rs Documentation link on crates.io without waiting for the Phase K minor (docli CLI HTML docs in sprint k.4).

| Item | Decision |
|------|----------|
| Workspace version | **0.7.1** in root `Cargo.toml` (`[workspace.package]` and internal path-dependency versions) |
| Published crates | `wyvern-schema`, `wyvern-wizard`, `wyvern-host`, `wyvern-viewer`, `wyvern-cli` |
| Deferred to 0.8.0 | docli-generated `site/cli/` and the phase-ending minor (k.4) |
| Not in 0.7.1 | Dialog protocol, CLI behavior, or kit channel formula changes |

## Publish dispatch

`crates-publish.yml` republishes crates for an **already published** GitHub Release tag. It checks out that tag and fails closed unless the release exists and is complete.

After this change is on the release branch and tag **v0.7.1** is a published GitHub Release:

```bash
gh workflow run crates-publish.yml --repo randlee/wyvern --ref main -f tag=v0.7.1
```

## Issue closeout

#62 and #63 **stay open** until that publish succeeds and crates.io shows, for the published crates:

- a non-empty README
- a Documentation link (`https://docs.rs/<crate>`)

Close them in the publish PR, or with a follow-up comment that cites the crates.io pages. Do not close them from the metadata PR alone.
