---
id: c.1
title: Production icon asset bundle
status: pending
branch: feature/phase-C-c1-icon-set
target: integrate/phase-C
---

# Sprint c.1 — Production icon asset bundle

## Goal

- Ship the curated built-in icon set (REQ-0030): six semantic roles, minimum two variants each, bundled via `include_bytes!`.
- Replace Phase B placeholder SVGs as the default source for `level` rendering and named icons that map to level roles.

## Hard Dependencies

- Phase B complete (`integrate/phase-B` merged): b.2 placeholder icon pipeline in `message/media.rs`

## Exact Targets

- `crates/wyvern-window/assets/icons/` — new production tree (see layout below)
- `crates/wyvern-window/assets/icons/placeholder/` — retained for b.2 regression tests only; **not** used at runtime after c.1
- `crates/wyvern-schema/src/icons.rs` — new module: role catalog (`ROLES`), `variant_count`, `parse_icon_spec` (ADR-0011: schema stays pure — no window dep)
- `crates/wyvern-schema/src/lib.rs` — export `icons` module
- `crates/wyvern-window/src/icons/mod.rs` — new module: embed helpers (`variant_bytes`, `svg_markup`) consuming `wyvern_schema::icons` catalog
- `crates/wyvern-window/src/message/media.rs` — switch `level` + named level-role resolution to production assets
- `crates/wyvern-window/src/lib.rs` — export `icons` module if needed for tests
- `docs/wyvern-window/architecture.md` — ADR-0015 icon asset layout (see cross-cutting doc)

## Deliverables

- Six roles: `info`, `warning`, `error`, `question`, `success`, `loading`
- Minimum **2 variants** per role (SVG preferred; PNG/WebP acceptable per REQ-0030)
- Assets embedded at compile time — no runtime filesystem reads for built-in icons
- `level: "info"` (etc.) renders production variant **1** for that role
- Named icon `"warning"` resolves to `warning/1.svg` (variant 1) — variant selection logic deferred to c.2
- Distinct visual identity per role (REQ-0012 for level values; success/loading are icon-only roles)
- Binary size impact documented in sprint notes if approaching NFR-0003 (10MB macOS)

## Required Work — asset layout (authoritative)

```
crates/wyvern-window/assets/icons/
  info/
    1.svg
    2.svg
  warning/
    1.svg
    2.svg
  error/
    1.svg
    2.svg
  question/
    1.svg
    2.svg
  success/
    1.svg
    2.svg
  loading/
    1.svg
    2.svg
  placeholder/          # b.2 legacy — tests only after c.1
    info.svg
    ...
```

### Runtime rules (c.1 scope)

- `MessageLevel` → production role variant 1 (replaces `placeholder_svg_for_level`)
- Named icon base name matching a level role (`"error"`) → same as `level: "error"` variant 1
- `:variant` suffix still accepted syntactically; c.1 may treat `:2` same as variant 1 until c.2 lands (document in non-closure if so)
- Path and base64 icon specs unchanged from b.2 (handled in existing `media.rs` paths)

## Explicit Code Samples

```rust
// crates/wyvern-schema/src/icons.rs — role catalog (validation + render share this)
pub const ROLES: &[&str] = &["info", "warning", "error", "question", "success", "loading"];

/// Max 1-based variant index per role (c.1 ships 2 variants each).
pub fn variant_count(role: &str) -> u32 {
    match role {
        "info" | "warning" | "error" | "question" | "success" | "loading" => 2,
        _ => 0,
    }
}

pub fn parse_icon_spec(spec: &str) -> (String, u32) {
    let (base, variant) = spec
        .split_once(':')
        .map(|(b, v)| (b, v.parse().unwrap_or(1)))
        .unwrap_or((spec, 1));
    (base.to_string(), variant)
}
```

```rust
// crates/wyvern-window/src/icons/mod.rs — embed helpers consume schema catalog
use wyvern_schema::icons;

pub fn variant_bytes(role: &str, index: u32) -> Option<&'static [u8]> {
    if index < 1 || index > icons::variant_count(role) {
        return None;
    }
    match (role, index) {
        ("info", 1) => Some(include_bytes!("../../assets/icons/info/1.svg")),
        ("info", 2) => Some(include_bytes!("../../assets/icons/info/2.svg")),
        // ... all roles × variants
        _ => None,
    }
}

pub fn svg_markup(role: &str, index: u32) -> Option<&'static str> {
    variant_bytes(role, index).and_then(|b| std::str::from_utf8(b).ok())
}
```

```rust
// media.rs — level resolution after c.1
pub fn icon_html_for_level(level: MessageLevel) -> IconHtml {
    let role = level.as_str(); // "info", "warning", ...
    icons::svg_markup(role, 1)
        .expect("c.1 bundles variant 1 for every level role")
        .to_string()
}
```

## This Sprint Does Not Close

- Variant index selection (`"warning:2"`) — c.2
- Unknown named icon validation error — c.2 (b.2 run-time fallback may remain until c.2)
- Win/Linux `decorations: false` — c.3
- NFR benchmarking — c.4
- Release workflow — c.5

## Acceptance Criteria

- All six roles present with ≥ 2 variants each under `assets/icons/{role}/`
- Each role's variants are visually distinct from each other and from other roles
- `level` on message renders production SVG (no `data-placeholder-level` attribute in shipped assets)
- Built-in icons require no filesystem access at runtime
- `cargo test -p wyvern-window` icon/level render tests updated to assert production markers
- Phase B placeholder directory retained for explicit regression tests only

## Required Validation

- `cargo test --workspace -- --test-threads=1`
- `cargo test -p wyvern-window` — level render tests, icon embed tests
- `sc-lint check native --config .sc-lint.toml`
- `cargo clippy --workspace -- -D warnings`
- Optional: `ls -lh target/release/wyvern` after release build — note size for c.4 NFR-0003
