#!/usr/bin/env python3
"""Enforce boundaries/*.toml dependency allow/forbid lists against Cargo.toml.

Validates each boundary that names an existing owner package:
  - every direct Cargo dependency must appear in allowed_dependencies
  - no direct Cargo dependency may appear in forbidden_dependencies
  - io_forbidden tokens receive minimal source-grep enforcement (c.15+)

Ownership note:
  - io_owns remains an advisory ownership declaration (documents which crate
    is responsible for a concern). Mechanical ownership of every io_owns
    token is not enforced here.
  - io_forbidden is enforced with lightweight path+content greps scoped to
    the owner crate sources so regressions like host spawning wyvern-viewer
    or viewer gaining an HTTP server are caught in the boundaries CI job.

Aspirational lint_rules:
  - [enforcement].lint_rules entries (e.g. LINT-BOUNDARY-*) are aspirational
    metadata until those rules are implemented in sc-lint / .sc-lint.toml.
  - This script does not enforce lint_rules; the current mechanical gate for
    ownership forbids is the io_forbidden greps above (plus Cargo dep checks).

Exits 0 on success, 1 on violation or parse error.
"""

from __future__ import annotations

import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BOUNDARIES = ROOT / "boundaries"
CRATES = ROOT / "crates"

# Minimal grep patterns for io_forbidden tokens used by active c.15 boundaries.
# Patterns are applied only to the owner package that declares the forbid.
# Doc comments / line comments are skipped before matching.
IO_FORBIDDEN_GREPS: dict[str, list[re.Pattern[str]]] = {
    "http_server": [
        re.compile(r"\baxum\b"),
        re.compile(r"\bhyper\b"),
        re.compile(r"TcpListener\s*::\s*bind"),
        re.compile(r"warp::"),
    ],
    "dialog_ipc": [
        re.compile(r"with_ipc_handler"),
        re.compile(r"\bIpcMessage\b"),
    ],
    "embedded_viewer_spawn": [
        re.compile(r"\bspawn_embedded_viewer\b"),
        re.compile(r'Command::new\([^;\n]*wyvern-viewer'),
        re.compile(r'CARGO_BIN_EXE_wyvern-viewer'),
        re.compile(r'WYVERN_VIEWER_BIN'),
    ],
    "webview_creation": [
        re.compile(r"\bWebViewBuilder\b"),
        re.compile(r"\bwry\s*::"),
        re.compile(r"\bwinit\s*::"),
    ],
    "inline_html": [
        re.compile(r"with_html\s*\("),
        re.compile(r"include_str!\s*\([^\n]*\.html"),
    ],
    "wizard_history_internals": [
        re.compile(r"wyvern_wizard::history"),
        re.compile(r"use\s+wyvern_wizard::history\b"),
    ],
    "wizard_domain_logic": [
        re.compile(r"\bHistory::"),
        re.compile(r"\bHistoryEntry::"),
    ],
    "browser_history_internals": [
        re.compile(r"pub\s+mod\s+history\b"),
        re.compile(r"pub\s+use\s+history::"),
    ],
    "stdin_reading": [
        re.compile(r"\bstd::io::stdin\b"),
        re.compile(r"\bio::stdin\b"),
    ],
    "stdout_writing": [
        re.compile(r"\bprintln!\s*\("),
        re.compile(r"\bprint!\s*\("),
        re.compile(r"\bstd::io::stdout\b"),
    ],
}


def code_lines_without_comments(text: str) -> str:
    """Strip // and /// / //! line comments for lightweight matching."""
    out: list[str] = []
    for line in text.splitlines():
        stripped = line.lstrip()
        if stripped.startswith("//"):
            continue
        # Drop trailing // comments (naive; good enough for boundary greps).
        if "//" in line:
            line = line.split("//", 1)[0]
        out.append(line)
    return "\n".join(out)


def package_dir(owner: str) -> Path | None:
    candidate = CRATES / owner
    if (candidate / "Cargo.toml").is_file():
        return candidate
    return None


def cargo_dep_names(manifest: Path) -> set[str]:
    data = tomllib.loads(manifest.read_text(encoding="utf-8"))
    names: set[str] = set()
    for section in ("dependencies", "build-dependencies"):
        for name in data.get(section, {}):
            names.add(name)
    return names


def load_boundary(path: Path) -> dict:
    return tomllib.loads(path.read_text(encoding="utf-8"))


def iter_rs_sources(pkg: Path) -> list[Path]:
    src = pkg / "src"
    if not src.is_dir():
        return []
    return sorted(src.rglob("*.rs"))


def check_io_forbidden(owner: str, boundary_path: Path, data: dict) -> list[str]:
    errors: list[str] = []
    ownership = data.get("ownership") or {}
    forbidden = list(ownership.get("io_forbidden") or [])
    if not forbidden:
        return errors

    pkg = package_dir(owner)
    if pkg is None:
        return errors

    sources = iter_rs_sources(pkg)
    if not sources:
        return errors

    for token in forbidden:
        patterns = IO_FORBIDDEN_GREPS.get(token)
        if not patterns:
            # Unknown token: advisory only (documented in module docstring).
            continue
        for src in sources:
            text = code_lines_without_comments(src.read_text(encoding="utf-8"))
            for pat in patterns:
                if pat.search(text):
                    rel = src.relative_to(ROOT)
                    errors.append(
                        f"{owner}: io_forbidden '{token}' matched {pat.pattern!r} in "
                        f"{rel} ({boundary_path.relative_to(ROOT)})"
                    )
                    break
            else:
                continue
            break
    return errors


def check_one(boundary_path: Path) -> list[str]:
    errors: list[str] = []
    data = load_boundary(boundary_path)
    owner = data.get("owner_package")
    if not owner:
        errors.append(f"{boundary_path}: missing owner_package")
        return errors

    pkg = package_dir(owner)
    if pkg is None:
        # Planned packages are inventory-only until present.
        return errors

    deps = data.get("dependencies") or {}
    allowed = set(deps.get("allowed_dependencies") or [])
    forbidden = set(deps.get("forbidden_dependencies") or [])
    if allowed or forbidden:
        cargo_deps = cargo_dep_names(pkg / "Cargo.toml")

        if allowed:
            unknown = sorted(cargo_deps - allowed)
            if unknown:
                errors.append(
                    f"{owner}: Cargo.toml deps not in allowed_dependencies "
                    f"({boundary_path.relative_to(ROOT)}): {', '.join(unknown)}"
                )

        banned = sorted(cargo_deps & forbidden)
        if banned:
            errors.append(
                f"{owner}: Cargo.toml deps in forbidden_dependencies "
                f"({boundary_path.relative_to(ROOT)}): {', '.join(banned)}"
            )

    errors.extend(check_io_forbidden(owner, boundary_path, data))
    return errors


def main() -> int:
    if not BOUNDARIES.is_dir():
        print(f"error: boundaries directory missing: {BOUNDARIES}", file=sys.stderr)
        return 1

    tomls = sorted(BOUNDARIES.glob("*/*.toml"))
    if not tomls:
        print("error: no boundary TOML files found", file=sys.stderr)
        return 1

    errors: list[str] = []
    checked = 0
    for path in tomls:
        data = load_boundary(path)
        owner = data.get("owner_package")
        if owner and package_dir(owner) is not None:
            checked += 1
        errors.extend(check_one(path))

    if errors:
        print("boundary check FAILED:", file=sys.stderr)
        for err in errors:
            print(f"  - {err}", file=sys.stderr)
        return 1

    print(f"boundary check OK ({checked} package(s), {len(tomls)} boundary file(s))")
    return 0


if __name__ == "__main__":
    sys.exit(main())
