#!/usr/bin/env python3
"""Pre-query Claude Code AskUserQuestion hook state (REQ-0124).

Prints one JSON object: ``{"config_patch": {"hook_state": ...}}``.
Does not write hook files. ``--dry-run`` is accepted and remains read-only.
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path
from typing import Any

MANAGED_BY = "wyvern:askuserquestion-hook"


def strip_jsonc(text: str) -> str:
    """Remove ``#``, ``//``, and ``/* */`` comments outside of strings."""
    out: list[str] = []
    i = 0
    in_str = False
    escape = False
    while i < len(text):
        ch = text[i]
        nxt = text[i + 1] if i + 1 < len(text) else ""
        if in_str:
            out.append(ch)
            if escape:
                escape = False
            elif ch == "\\":
                escape = True
            elif ch == '"':
                in_str = False
            i += 1
            continue
        if ch == '"':
            in_str = True
            out.append(ch)
            i += 1
            continue
        if ch == "#" or (ch == "/" and nxt == "/"):
            while i < len(text) and text[i] not in "\n\r":
                i += 1
            continue
        if ch == "/" and nxt == "*":
            i += 2
            while i < len(text) and not (text[i] == "*" and i + 1 < len(text) and text[i + 1] == "/"):
                i += 1
            i = min(i + 2, len(text))
            continue
        out.append(ch)
        i += 1
    return "".join(out)


def load_settings(path: str) -> dict[str, Any] | None:
    """Parse a settings JSON/JSONC file. Missing or invalid → ``None``."""
    if not os.path.isfile(path):
        return None
    try:
        with open(path, encoding="utf-8") as handle:
            raw = handle.read()
        value = json.loads(strip_jsonc(raw))
    except (OSError, json.JSONDecodeError):
        return None
    return value if isinstance(value, dict) else None


def pretool_entries(settings: dict[str, Any] | None) -> list[Any]:
    if not settings:
        return []
    hooks = settings.get("hooks")
    if not isinstance(hooks, dict):
        return []
    entries = hooks.get("PreToolUse")
    return entries if isinstance(entries, list) else []


def is_managed_entry(entry: Any) -> bool:
    if not isinstance(entry, dict):
        return False
    hooks = entry.get("hooks")
    if not isinstance(hooks, list):
        return False
    return any(
        isinstance(hook, dict) and hook.get("managed_by") == MANAGED_BY for hook in hooks
    )


def resolved_settings_path(root: str | None, filename: str) -> str:
    """Absolute settings file path. Same roots as apply ``scope_paths``."""
    if not root:
        return ""
    return os.path.abspath(os.path.join(root, ".claude", filename))


def scope_state(settings_path: str) -> dict[str, Any]:
    settings = load_settings(settings_path) if settings_path else None
    present = any(is_managed_entry(entry) for entry in pretool_entries(settings))
    return {
        "enabled": present,
        "installed": present,
        "settings_path": settings_path,
    }


def resolve_home() -> str | None:
    """WYVERN_HOME, then HOME, then USERPROFILE, then Path.home()."""
    for key in ("WYVERN_HOME", "HOME", "USERPROFILE"):
        value = os.environ.get(key)
        if value:
            return value
    try:
        return str(Path.home())
    except (RuntimeError, OSError):
        return None


def hook_state() -> dict[str, Any]:
    home = resolve_home()
    repo = os.environ.get("WYVERN_REPO_ROOT") or os.getcwd()
    # Same roots as apply-askuserquestion-hook.scope_paths; abspath for UI.
    global_path = resolved_settings_path(home, "settings.json")
    repo_path = resolved_settings_path(repo, "settings.local.json")
    return {
        "global": scope_state(global_path),
        "repo": scope_state(repo_path),
    }


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Query AskUserQuestion hook disk state.")
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Accepted from WorkflowRunner; this script never writes.",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    parse_args(argv)
    payload = {"config_patch": {"hook_state": hook_state()}}
    json.dump(payload, sys.stdout, separators=(",", ":"))
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
