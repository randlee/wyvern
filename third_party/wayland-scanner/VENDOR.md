# Vendored wayland-scanner 0.31.10 (patched)

Source: crates.io `wayland-scanner` 0.31.10 (vcs `a3d7927d`).

## Why

crates.io `wayland-scanner` 0.31.10 pins `quick-xml ^0.39`, which is
flagged by RUSTSEC-2026-0194 / RUSTSEC-2026-0195. Upstream master bumps
to `quick-xml` 0.41 but also changes generated `Proxy` APIs, so a
`[patch.crates-io]` git rev breaks `wayland-client` on Linux (E0277).

## Patch vs crates.io 0.31.10

1. `Cargo.toml`: `quick-xml = "0.41"`
2. `src/parse.rs`: `xml_content()` → `xml10_content()` (0.41 API)

## Remove when

crates.io publishes a `wayland-scanner` that depends on `quick-xml >= 0.41`
compatible with the `wayland-client` version in `Cargo.lock`. Then delete
this directory and the `[patch.crates-io]` entry in the workspace
`Cargo.toml`.
