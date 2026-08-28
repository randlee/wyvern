#!/usr/bin/env python3
"""Write finish ``data.dag`` to ``wyvern-dag-export.json`` (REQ-0125).

Default path is ``$WYVERN_REPO_ROOT/wyvern-dag-export.json`` when that env
var is set, otherwise ``./wyvern-dag-export.json``. ``--dry-run`` (appended
by ``--workflow-dry-run``) writes nothing. ``-o`` / ``--output`` is
script/test-only; ``WorkflowRunner`` does not pass it.

This script only serializes the finish DAG. It does not spawn agents or
run the graph.
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path
from typing import Any


EXPORT_NAME = "wyvern-dag-export.json"


def eprint(message: str) -> None:
    print(message, file=sys.stderr)


def resolve_output_root() -> Path:
    raw = os.environ.get("WYVERN_REPO_ROOT")
    if raw:
        return Path(raw).resolve()
    return Path.cwd().resolve()


def default_export_path() -> Path:
    return resolve_output_root() / EXPORT_NAME


def finish_data(finish: dict[str, Any]) -> dict[str, Any]:
    data = finish.get("data")
    if data is None:
        return {}
    if not isinstance(data, dict):
        raise ValueError("finish JSON data must be an object")
    return data


def require_string(value: Any, field: str) -> str:
    if not isinstance(value, str) or not value:
        raise ValueError(f"data.dag.{field} must be a non-empty string")
    return value


def validate_dag(dag: Any) -> dict[str, Any]:
    if not isinstance(dag, dict):
        raise ValueError("finish JSON data.dag is required and must be an object")
    layout_id = require_string(dag.get("layout_id"), "layout_id")
    nodes_raw = dag.get("nodes")
    if not isinstance(nodes_raw, list):
        raise ValueError("data.dag.nodes must be an array")
    nodes: list[dict[str, str]] = []
    for index, node in enumerate(nodes_raw):
        if not isinstance(node, dict):
            raise ValueError(f"data.dag.nodes[{index}] must be an object")
        nodes.append(
            {
                "id": require_string(node.get("id"), f"nodes[{index}].id"),
                "name": require_string(node.get("name"), f"nodes[{index}].name"),
                "role": require_string(node.get("role"), f"nodes[{index}].role"),
            }
        )
    edges_raw = dag.get("edges")
    if not isinstance(edges_raw, list):
        raise ValueError("data.dag.edges must be an array")
    edges: list[list[str]] = []
    for index, edge in enumerate(edges_raw):
        if not isinstance(edge, list) or len(edge) != 2:
            raise ValueError(f"data.dag.edges[{index}] must be a [from, to] pair")
        start = edge[0]
        end = edge[1]
        if not isinstance(start, str) or not start or not isinstance(end, str) or not end:
            raise ValueError(f"data.dag.edges[{index}] must be two non-empty strings")
        edges.append([start, end])
    return {"layout_id": layout_id, "nodes": nodes, "edges": edges}


def resolve_output_path(raw: str | None) -> Path:
    if not raw:
        return default_export_path()
    path = Path(raw)
    if path.is_absolute():
        return path.resolve()
    return (resolve_output_root() / path).resolve()


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


def write_export(path: Path, dag: dict[str, Any], dry_run: bool) -> None:
    body = json.dumps(dag, indent=2, sort_keys=False) + "\n"
    if dry_run:
        print(f"would write {path}")
        return
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(body, encoding="utf-8")
    print(f"wrote {path}")


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Export Agent DAG finish JSON.")
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print the export path; write no files (WorkflowRunner dry-run).",
    )
    parser.add_argument(
        "-o",
        "--output",
        metavar="PATH",
        help="Script/test-only: write this path instead of the default export file.",
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
        dag = validate_dag(finish_data(finish).get("dag"))
        write_export(resolve_output_path(args.output), dag, args.dry_run)
        return 0
    except (OSError, ValueError, json.JSONDecodeError) as exc:
        eprint(f"export-agent-dag: {exc}")
        return 1


if __name__ == "__main__":
    sys.exit(main())
