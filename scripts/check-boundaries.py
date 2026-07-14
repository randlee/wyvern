#!/usr/bin/env python3
"""Enforce boundaries/*.toml dependency allow/forbid lists against Cargo.toml.

Validates each boundary that names an existing owner package:
  - every direct Cargo dependency must appear in allowed_dependencies
  - no direct Cargo dependency may appear in forbidden_dependencies

Exits 0 on success, 1 on violation or parse error.
"""

from __future__ import annotations

import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BOUNDARIES = ROOT / "boundaries"
CRATES = ROOT / "crates"


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


def check_one(boundary_path: Path) -> list[str]:
    errors: list[str] = []
    data = load_boundary(boundary_path)
    owner = data.get("owner_package")
    if not owner:
        errors.append(f"{boundary_path}: missing owner_package")
        return errors

    pkg = package_dir(owner)
    if pkg is None:
        # Planned packages (e.g. wyvern-viewer) are inventory-only until present.
        return errors

    deps = data.get("dependencies") or {}
    allowed = set(deps.get("allowed_dependencies") or [])
    forbidden = set(deps.get("forbidden_dependencies") or [])
    if not allowed and not forbidden:
        return errors

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
