#!/usr/bin/env python3
"""Apply or invoke the Claude Code AskUserQuestion hook (REQ-0125).

Post mode (wizard finish on stdin): merge or strip Wyvern-managed hook entries.
``--dry-run`` prints the plan and writes nothing. ``--remove`` is script/test-only.
``--invoke`` maps PreToolUse stdin to a Wyvern ``question`` command.
"""

from __future__ import annotations

import argparse
import json
import os
import shlex
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any

MANAGED_BY = "wyvern:askuserquestion-hook"
MANAGED_VERSION = 1
MANAGED_COMMENT = "# wyvern:askuserquestion-hook v1"
SIDECAR_NAME = "wyvern-askuserquestion-bin"
MATCHER = "AskUserQuestion"


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


def load_settings(path: str) -> dict[str, Any]:
    """Parse settings JSON/JSONC; missing file → empty object."""
    if not os.path.isfile(path):
        return {}
    with open(path, encoding="utf-8") as handle:
        raw = handle.read()
    if not raw.strip():
        return {}
    value = json.loads(strip_jsonc(raw))
    if not isinstance(value, dict):
        raise ValueError(f"settings file is not a JSON object: {path}")
    return value


def is_managed_hook(hook: Any) -> bool:
    return isinstance(hook, dict) and hook.get("managed_by") == MANAGED_BY


def strip_managed(settings: dict[str, Any]) -> dict[str, Any]:
    """Remove only Wyvern-managed PreToolUse hooks; keep unrelated entries."""
    hooks = settings.get("hooks")
    if not isinstance(hooks, dict):
        return settings
    entries = hooks.get("PreToolUse")
    if not isinstance(entries, list):
        return settings
    kept: list[Any] = []
    for entry in entries:
        if not isinstance(entry, dict):
            kept.append(entry)
            continue
        hook_list = entry.get("hooks")
        if not isinstance(hook_list, list):
            kept.append(entry)
            continue
        remaining = [hook for hook in hook_list if not is_managed_hook(hook)]
        if not remaining:
            continue
        new_entry = dict(entry)
        new_entry["hooks"] = remaining
        kept.append(new_entry)
    hooks = dict(hooks)
    hooks["PreToolUse"] = kept
    settings = dict(settings)
    settings["hooks"] = hooks
    return settings


def resolve_python() -> str:
    for name in ("python3", "py", "python"):
        found = shutil.which(name)
        if found:
            return os.path.abspath(found)
    return "python3"


def quote_hook_token(token: str) -> str:
    """Quote one argv token for Claude Code's hook ``command`` string."""
    if os.name == "nt":
        return subprocess.list2cmdline([token])
    return shlex.quote(token)


def resolve_apply_script() -> str:
    return os.path.realpath(os.path.abspath(__file__))


def hook_command() -> str:
    python = resolve_python()
    script = resolve_apply_script()
    return f"{quote_hook_token(python)} {quote_hook_token(script)} --invoke"


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


def is_python_interpreter(path: str) -> bool:
    base = os.path.basename(path).lower()
    stem, ext = os.path.splitext(base)
    if ext in {".exe", ".bat", ".cmd"}:
        base = stem
    if base in {"python", "python3", "py"}:
        return True
    try:
        return os.path.realpath(path) == os.path.realpath(sys.executable)
    except OSError:
        return False


def managed_entry() -> dict[str, Any]:
    return {
        "matcher": MATCHER,
        "hooks": [
            {
                "type": "command",
                "command": hook_command(),
                "managed_by": MANAGED_BY,
                "version": MANAGED_VERSION,
            }
        ],
    }


def upsert_managed(settings: dict[str, Any]) -> dict[str, Any]:
    settings = strip_managed(settings)
    hooks = settings.get("hooks")
    if not isinstance(hooks, dict):
        hooks = {}
    else:
        hooks = dict(hooks)
    entries = hooks.get("PreToolUse")
    if not isinstance(entries, list):
        entries = []
    else:
        entries = list(entries)
    entries.append(managed_entry())
    hooks["PreToolUse"] = entries
    settings = dict(settings)
    settings["hooks"] = hooks
    return settings


def dumps_with_marker(settings: dict[str, Any]) -> str:
    """Pretty-print JSONC with the Wyvern marker above the managed entry."""
    text = json.dumps(settings, indent=2, ensure_ascii=False)
    needle = f'"managed_by": "{MANAGED_BY}"'
    idx = text.find(needle)
    if idx < 0:
        return text + "\n"
    # Walk back to the matcher-object opening brace that contains this hook.
    brace = text.rfind("{", 0, idx)
    # The inner hook object starts at `brace`; the matcher object is the previous `{`.
    outer = text.rfind("{", 0, brace)
    if outer < 0:
        outer = brace
    line_start = text.rfind("\n", 0, outer) + 1
    indent = text[line_start:outer]
    marked = text[:line_start] + indent + MANAGED_COMMENT + "\n" + text[line_start:]
    if not marked.endswith("\n"):
        marked += "\n"
    return marked


def write_settings(path: str, settings: dict[str, Any], dry_run: bool) -> None:
    if dry_run:
        print(f"dry-run: would write {path}", file=sys.stderr)
        return
    parent = os.path.dirname(path)
    os.makedirs(parent, exist_ok=True)
    payload = dumps_with_marker(settings)
    tmp = path + ".wyvern-tmp"
    with open(tmp, "w", encoding="utf-8") as handle:
        handle.write(payload)
    os.replace(tmp, path)


def resolve_wyvern_bin() -> str | None:
    env_bin = os.environ.get("WYVERN_BIN")
    if env_bin:
        if os.path.isfile(env_bin):
            return os.path.abspath(env_bin)
        found = shutil.which(env_bin)
        if found:
            return os.path.abspath(found)
    which = shutil.which("wyvern")
    if which:
        return os.path.abspath(which)
    return None


def sidecar_path(settings_path: str) -> str:
    return os.path.join(os.path.dirname(settings_path), SIDECAR_NAME)


def write_sidecar(settings_path: str, wyvern_bin: str | None, dry_run: bool) -> None:
    if not wyvern_bin:
        return
    path = sidecar_path(settings_path)
    if dry_run:
        print(f"dry-run: would write sidecar {path}", file=sys.stderr)
        return
    os.makedirs(os.path.dirname(path), exist_ok=True)
    tmp = path + ".wyvern-tmp"
    with open(tmp, "w", encoding="utf-8") as handle:
        handle.write(wyvern_bin + "\n")
    os.replace(tmp, path)


def read_sidecar_bin() -> str | None:
    home = resolve_home()
    repo = os.environ.get("WYVERN_REPO_ROOT") or os.getcwd()
    candidates = []
    if home:
        candidates.append(os.path.join(home, ".claude", SIDECAR_NAME))
    candidates.append(os.path.join(repo, ".claude", SIDECAR_NAME))
    for path in candidates:
        try:
            with open(path, encoding="utf-8") as handle:
                value = handle.read().strip()
        except OSError:
            continue
        if value and os.path.isfile(value):
            return value
    return None


def scope_paths() -> tuple[str | None, str]:
    home = resolve_home()
    repo = os.environ.get("WYVERN_REPO_ROOT") or os.getcwd()
    global_path = os.path.join(home, ".claude", "settings.json") if home else None
    repo_path = os.path.join(repo, ".claude", "settings.local.json")
    return global_path, repo_path


def enabled_from_finish(finish: dict[str, Any], scope: str) -> bool:
    data = finish.get("data")
    if not isinstance(data, dict):
        raise ValueError("finish JSON data must be an object")
    hook_config = data.get("hook_config")
    if not isinstance(hook_config, dict):
        raise ValueError("finish JSON data.hook_config is required")
    scope_obj = hook_config.get(scope)
    if not isinstance(scope_obj, dict):
        raise ValueError(f"finish JSON data.hook_config.{scope} is required")
    return bool(scope_obj.get("enabled"))


def apply_scope(
    settings_path: str | None,
    enabled: bool,
    wyvern_bin: str | None,
    dry_run: bool,
    *,
    require_home: bool,
) -> None:
    if settings_path is None:
        if enabled and require_home:
            raise ValueError(
                "WYVERN_HOME, HOME, or USERPROFILE is required to apply the global AskUserQuestion hook"
            )
        return
    if not enabled:
        if not os.path.isfile(settings_path):
            return
        settings = strip_managed(load_settings(settings_path))
        write_settings(settings_path, settings, dry_run)
        return
    settings = upsert_managed(load_settings(settings_path))
    write_settings(settings_path, settings, dry_run)
    write_sidecar(settings_path, wyvern_bin, dry_run)


def apply_finish(finish: dict[str, Any], dry_run: bool) -> None:
    wyvern_bin = resolve_wyvern_bin()
    global_path, repo_path = scope_paths()
    apply_scope(
        global_path,
        enabled_from_finish(finish, "global"),
        wyvern_bin,
        dry_run,
        require_home=True,
    )
    apply_scope(
        repo_path,
        enabled_from_finish(finish, "repo"),
        wyvern_bin,
        dry_run,
        require_home=False,
    )


def remove_managed(dry_run: bool) -> None:
    global_path, repo_path = scope_paths()
    for path in (global_path, repo_path):
        if path is None or not os.path.isfile(path):
            continue
        settings = strip_managed(load_settings(path))
        write_settings(path, settings, dry_run)


def invoke_from_stdin(raw: str) -> int:
    try:
        event = json.loads(raw) if raw.strip() else {}
    except json.JSONDecodeError as exc:
        print(f"apply-askuserquestion-hook: invalid PreToolUse JSON: {exc}", file=sys.stderr)
        return 1
    if not isinstance(event, dict):
        print("apply-askuserquestion-hook: PreToolUse stdin must be an object", file=sys.stderr)
        return 1
    tool_input = event.get("tool_input")
    if not isinstance(tool_input, dict):
        print("apply-askuserquestion-hook: tool_input must be an object", file=sys.stderr)
        return 1
    questions = tool_input.get("questions")
    if not isinstance(questions, list):
        print("apply-askuserquestion-hook: tool_input.questions must be an array", file=sys.stderr)
        return 1
    envelope = {"type": "question", "questions": questions}
    wyvern_bin = resolve_wyvern_bin() or read_sidecar_bin()
    if not wyvern_bin:
        print(
            "apply-askuserquestion-hook: WYVERN_BIN is unset and wyvern was not found on PATH",
            file=sys.stderr,
        )
        return 1
    if is_python_interpreter(wyvern_bin):
        print(
            "apply-askuserquestion-hook: refusing to exec a Python interpreter as WYVERN_BIN",
            file=sys.stderr,
        )
        return 1
    completed = subprocess.run(
        [wyvern_bin, json.dumps(envelope, separators=(",", ":"))],
        check=False,
        capture_output=True,
        text=True,
    )
    if completed.stderr:
        sys.stderr.write(completed.stderr)
    sys.stdout.write(completed.stdout)
    return 0 if completed.returncode == 0 else completed.returncode


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Apply or invoke the AskUserQuestion hook.")
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print the plan; write no hook files (WorkflowRunner dry-run).",
    )
    parser.add_argument(
        "--remove",
        action="store_true",
        help="Script/test-only: strip wyvern:askuserquestion-hook entries.",
    )
    parser.add_argument(
        "--invoke",
        action="store_true",
        help="Installed hook: map PreToolUse stdin to a Wyvern question dialog.",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    try:
        raw = sys.stdin.read()
        if args.invoke:
            return invoke_from_stdin(raw)
        if args.remove:
            remove_managed(args.dry_run)
            return 0
        if not raw.strip():
            raise ValueError("post stdin must be the wizard finish JSON object")
        finish = json.loads(raw)
        if not isinstance(finish, dict):
            raise ValueError("finish JSON must be an object")
        apply_finish(finish, args.dry_run)
        return 0
    except (OSError, ValueError, json.JSONDecodeError) as exc:
        print(f"apply-askuserquestion-hook: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
