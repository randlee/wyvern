#!/usr/bin/env python3
"""Enforce Wyvern io_forbidden grep policy from scripts/io-forbidden.toml.

ADR-004 boundary inventory and package dependency policy are enforced by
`sc-lint lint sc-boundary`. This script covers Wyvern-specific ownership greps
that are not yet modeled in sc-lint-boundary.

Exits 0 on success, 1 on violation or parse error.
"""

from __future__ import annotations

import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
IO_POLICY = ROOT / "scripts" / "io-forbidden.toml"
CRATES = ROOT / "crates"

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
        re.compile(r"CARGO_BIN_EXE_wyvern-viewer"),
        re.compile(r"WYVERN_VIEWER_BIN"),
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
    out: list[str] = []
    for line in text.splitlines():
        stripped = line.lstrip()
        if stripped.startswith("//"):
            continue
        if "//" in line:
            line = line.split("//", 1)[0]
        out.append(line)
    return "\n".join(out)


def package_dir(owner: str) -> Path | None:
    candidate = CRATES / owner
    if (candidate / "Cargo.toml").is_file():
        return candidate
    for crate_dir in sorted(CRATES.iterdir()):
        manifest = crate_dir / "Cargo.toml"
        if not manifest.is_file():
            continue
        data = tomllib.loads(manifest.read_text(encoding="utf-8"))
        if data.get("package", {}).get("name") == owner:
            return crate_dir
    return None


def iter_rs_sources(pkg: Path) -> list[Path]:
    src = pkg / "src"
    if not src.is_dir():
        return []
    return sorted(src.rglob("*.rs"))


def check_io_forbidden(owner: str, forbidden: list[str]) -> list[str]:
    errors: list[str] = []
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
            continue
        for src in sources:
            text = code_lines_without_comments(src.read_text(encoding="utf-8"))
            for pat in patterns:
                if pat.search(text):
                    rel = src.relative_to(ROOT)
                    errors.append(
                        f"{owner}: io_forbidden '{token}' matched {pat.pattern!r} in {rel}"
                    )
                    break
            else:
                continue
            break
    return errors


def main() -> int:
    if not IO_POLICY.is_file():
        print(f"error: missing io policy file: {IO_POLICY}", file=sys.stderr)
        return 1

    policy = tomllib.loads(IO_POLICY.read_text(encoding="utf-8"))
    errors: list[str] = []
    checked = 0
    for owner, section in policy.items():
        if not isinstance(section, dict):
            continue
        forbidden = list(section.get("io_forbidden") or [])
        if package_dir(owner) is not None:
            checked += 1
        errors.extend(check_io_forbidden(owner, forbidden))

    if errors:
        print("io-forbidden check FAILED:", file=sys.stderr)
        for err in errors:
            print(f"  - {err}", file=sys.stderr)
        return 1

    print(f"io-forbidden check OK ({checked} package(s))")
    return 0


if __name__ == "__main__":
    sys.exit(main())
