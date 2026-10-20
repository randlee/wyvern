# Version policy

## Phase K decision (sprint k.3)

**Ship patch 0.7.1.** Do not defer the crates.io README and documentation-link fix to **0.8.0**.

k.4 moves the workspace to **0.8.0**. This section is the patch decision; tag **v0.7.1** from the k.3 commit. The minor is recorded under sprint k.4 below.

#62 and #63 are packaging metadata only. A patch republish puts a non-empty README and a docs.rs Documentation link on crates.io without waiting for the Phase K minor (docli CLI HTML docs in sprint k.4).

| Item | Decision |
|------|----------|
| Workspace version at k.3 | **0.7.1** (tag **v0.7.1** from the k.3 commit). k.4 sets root `Cargo.toml` to **0.8.0** |
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

## Phase K decision (sprint k.4)

**Ship minor 0.8.0** for docli-generated CLI HTML, on top of the **0.7.1** crates.io metadata patch. This is not a patch-only release, so `site/announcements/wyvern-v0.8.0.md` is in scope.

The k.3 workspace version (**0.7.1**) is the parent of this minor. Tag **v0.7.1** from the k.3 commit on `integrate/phase-K` (before this version bump) and dispatch `crates-publish.yml` with `tag=v0.7.1`. This branch's workspace version is **0.8.0**.

| Item | Decision |
|------|----------|
| Workspace version | **0.8.0** in root `Cargo.toml` (`[workspace.package]` and internal path-dependency versions) |
| 0.7.1 tag | k.3 commit. Publishes README and docs.rs links only. #62 and #63 stay open until that publish is visible on crates.io |
| 0.8.0 contents | `gen-cli-model`, pinned `sc-docli` 0.2.0, committed `site/cli/`, CI drift gate (ADR-0027, REQ-0150) |
| Not in 0.8.0 | Dialog protocol changes, kit channel formula changes, docli Markdown manual |
| Announcement | `site/announcements/wyvern-v0.8.0.md` |
| Integration merge | `integrate/phase-K` → `develop` after phase-ending QA (coordinator; not a separate sprint) |
