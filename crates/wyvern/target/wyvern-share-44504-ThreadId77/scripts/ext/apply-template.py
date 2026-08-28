#!/usr/bin/env python3
"""Copy a bundled template and substitute ``{var}`` placeholders (REQ-0125).

Post mode (wizard finish on stdin): copy files under ``$WYVERN_SHARE/templates``.
``--dry-run`` prints the copy plan and writes nothing. ``--force`` is
script/test-only and is not passed by the shipped ``WorkflowRunner`` post.
``--finish-file`` is test-only.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
from pathlib import Path
from typing import Any

MANAGED_BY = "wyvern:template"
MANAGED_VERSION = 1
ALLOWED_TEMPLATE_IDS = (
    "pytest",
    "github-workflow",
    "nunit",
    "xunit",
    "benchmark-dotnet",
    "wizard/minimal",
    "wizard/two-step",
)
SLASH_ALLOWED_IDS = frozenset({"wizard/minimal", "wizard/two-step"})
VAR_PATTERN = re.compile(r"\{([A-Za-z_][A-Za-z0-9_]*)\}")


def eprint(message: str) -> None:
    print(message, file=sys.stderr)


def is_under(path: Path, root: Path) -> bool:
    try:
        path.relative_to(root)
        return True
    except ValueError:
        return False


def has_parent_component(raw: str) -> bool:
    return ".." in Path(raw).parts


def resolve_share() -> Path:
    raw = os.environ.get("WYVERN_SHARE")
    if not raw:
        raise ValueError("WYVERN_SHARE is required to locate bundled templates")
    share = Path(raw).resolve()
    if not share.is_dir():
        raise ValueError(f"WYVERN_SHARE is not a directory: {share}")
    return share


def resolve_output_root() -> Path:
    raw = os.environ.get("WYVERN_REPO_ROOT")
    if raw:
        return Path(raw).resolve()
    return Path.cwd().resolve()


def validate_template_id(template_id: str) -> str:
    if not template_id:
        raise ValueError("finish JSON data.template_id is required")
    if template_id not in ALLOWED_TEMPLATE_IDS:
        raise ValueError(
            f"template_id must be one of {', '.join(ALLOWED_TEMPLATE_IDS)}: {template_id}"
        )
    if "/" in template_id and template_id not in SLASH_ALLOWED_IDS:
        raise ValueError(f"slash is not allowed in template_id: {template_id}")
    if has_parent_component(template_id):
        raise ValueError(f"template_id must not contain '..': {template_id}")
    return template_id


def resolve_template_dir(share: Path, template_id: str) -> Path:
    templates_root = (share / "templates").resolve()
    if not templates_root.is_dir():
        raise ValueError(f"templates root does not exist: {templates_root}")
    candidate = (templates_root / template_id).resolve()
    if not is_under(candidate, templates_root):
        raise ValueError(f"template path escaped the templates root: {template_id}")
    if not candidate.is_dir():
        raise ValueError(f"template directory is missing: {candidate}")
    return candidate


def validate_output_path(raw: str) -> str:
    if not raw or not raw.strip():
        raise ValueError("finish JSON data.output_path is required")
    path = raw.strip()
    as_path = Path(path)
    if as_path.is_absolute():
        raise ValueError("output_path must be relative to the output root")
    if has_parent_component(path):
        raise ValueError(f"output_path must not contain '..': {path}")
    return path


def load_manifest(template_dir: Path) -> dict[str, Any]:
    path = template_dir / "template.manifest.json"
    if not path.is_file():
        raise ValueError(f"template.manifest.json is missing: {path}")
    with path.open(encoding="utf-8") as handle:
        manifest = json.load(handle)
    if not isinstance(manifest, dict):
        raise ValueError("template.manifest.json must be a JSON object")
    files = manifest.get("files")
    if not isinstance(files, list) or not files:
        raise ValueError("template.manifest.json files must be a non-empty array")
    for item in files:
        if not isinstance(item, str) or not item or has_parent_component(item):
            raise ValueError(f"invalid template file entry: {item}")
        if Path(item).is_absolute():
            raise ValueError(f"template file entry must be relative: {item}")
    return manifest


def variables_from_finish(data: dict[str, Any]) -> dict[str, str]:
    raw = data.get("variables")
    if raw is None:
        return {}
    if not isinstance(raw, dict):
        raise ValueError("finish JSON data.variables must be an object")
    out: dict[str, str] = {}
    for key, value in raw.items():
        if not isinstance(key, str) or not key:
            raise ValueError("variable names must be non-empty strings")
        if value is None:
            out[key] = ""
        elif isinstance(value, (str, int, float, bool)):
            out[key] = str(value)
        else:
            raise ValueError(f"variable {key} must be a scalar")
    return out


def substitute(text: str, variables: dict[str, str]) -> str:
    def repl(match: re.Match[str]) -> str:
        key = match.group(1)
        return variables[key] if key in variables else match.group(0)

    return VAR_PATTERN.sub(repl, text)


def sidecar_path(dest: Path) -> Path:
    return dest.with_name(dest.name + ".wyvern.json")


def is_tagged(dest: Path) -> bool:
    path = sidecar_path(dest)
    if not path.is_file():
        return False
    try:
        with path.open(encoding="utf-8") as handle:
            payload = json.load(handle)
    except (OSError, json.JSONDecodeError):
        return False
    return isinstance(payload, dict) and payload.get("managed_by") == MANAGED_BY


def is_directory_output(output_path: str, files: list[str]) -> bool:
    if output_path.endswith(("/", "\\")):
        return True
    return len(files) != 1


def destination_for(output_root: Path, output_path: str, rel_file: str, files: list[str]) -> Path:
    if is_directory_output(output_path, files):
        dest = (output_root / output_path / rel_file).resolve()
    else:
        dest = (output_root / output_path).resolve()
    if not is_under(dest, output_root):
        raise ValueError(f"output path escaped the output root: {dest}")
    return dest


def source_for(template_dir: Path, rel_file: str) -> Path:
    src = (template_dir / rel_file).resolve()
    if not is_under(src, template_dir):
        raise ValueError(f"template file escaped the template root: {rel_file}")
    if not src.is_file():
        raise ValueError(f"template source file is missing: {src}")
    return src


class CopyOp:
    def __init__(self, src: Path, dest: Path, sidecar: Path) -> None:
        self.src = src
        self.dest = dest
        self.sidecar = sidecar


def plan_copies(
    template_dir: Path,
    manifest: dict[str, Any],
    output_root: Path,
    output_path: str,
) -> list[CopyOp]:
    files = [str(item) for item in manifest["files"]]
    ops: list[CopyOp] = []
    for rel_file in files:
        src = source_for(template_dir, rel_file)
        dest = destination_for(output_root, output_path, rel_file, files)
        ops.append(CopyOp(src, dest, sidecar_path(dest)))
    return ops


def overwrite_allowed(dest: Path, force: bool) -> tuple[bool, str]:
    if not dest.exists():
        return True, "create"
    if is_tagged(dest):
        return True, "overwrite-tagged"
    if force:
        return True, "overwrite-force"
    return False, "untagged-exists"


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_name(path.name + ".wyvern-tmp")
    tmp.write_text(text, encoding="utf-8")
    tmp.replace(path)


def sidecar_bytes(template_id: str) -> str:
    return json.dumps(
        {"managed_by": MANAGED_BY, "template_id": template_id, "version": MANAGED_VERSION},
        indent=2,
    ) + "\n"


def print_plan(ops: list[CopyOp], actions: list[str]) -> None:
    for op, action in zip(ops, actions, strict=True):
        print(f"copy {op.src} -> {op.dest} ({action})")
        print(f"sidecar {op.sidecar}")


def apply_ops(
    ops: list[CopyOp],
    variables: dict[str, str],
    template_id: str,
    force: bool,
    dry_run: bool,
) -> int:
    actions: list[str] = []
    blocked: list[str] = []
    for op in ops:
        allowed, action = overwrite_allowed(op.dest, force)
        actions.append(action)
        if not allowed:
            blocked.append(str(op.dest))
    if dry_run:
        print_plan(ops, actions)
        return 0
    if blocked:
        for dest in blocked:
            eprint(f"apply-template: destination exists and is untagged: {dest}")
        return 1
    for op in ops:
        text = substitute(op.src.read_text(encoding="utf-8"), variables)
        write_text(op.dest, text)
        write_text(op.sidecar, sidecar_bytes(template_id))
    print_plan(ops, actions)
    return 0


def finish_data(finish: dict[str, Any]) -> dict[str, Any]:
    data = finish.get("data")
    if not isinstance(data, dict):
        raise ValueError("finish JSON data must be an object")
    return data


def apply_finish(finish: dict[str, Any], dry_run: bool, force: bool) -> int:
    data = finish_data(finish)
    template_id = validate_template_id(str(data.get("template_id") or ""))
    output_path = validate_output_path(str(data.get("output_path") or ""))
    variables = variables_from_finish(data)
    share = resolve_share()
    template_dir = resolve_template_dir(share, template_id)
    manifest = load_manifest(template_dir)
    output_root = resolve_output_root()
    ops = plan_copies(template_dir, manifest, output_root, output_path)
    return apply_ops(ops, variables, template_id, force, dry_run)


def load_finish(args: argparse.Namespace) -> dict[str, Any]:
    if args.finish_file:
        raw = Path(args.finish_file).read_text(encoding="utf-8")
    else:
        raw = sys.stdin.read()
    if not raw.strip():
        raise ValueError("post stdin must be the wizard finish JSON object")
    finish = json.loads(raw)
    if not isinstance(finish, dict):
        raise ValueError("finish JSON must be an object")
    return finish


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Apply a bundled Wyvern template.")
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print the copy plan; write no files (WorkflowRunner dry-run).",
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="Script/test-only: overwrite untagged destination files.",
    )
    parser.add_argument(
        "--finish-file",
        metavar="PATH",
        help="Test-only: read finish JSON from PATH instead of stdin.",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    try:
        finish = load_finish(args)
        return apply_finish(finish, args.dry_run, args.force)
    except (OSError, ValueError, json.JSONDecodeError) as exc:
        eprint(f"apply-template: {exc}")
        return 1


if __name__ == "__main__":
    sys.exit(main())
