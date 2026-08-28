#!/usr/bin/env python3
"""Convert a CSV file into a wyvern table-viewer tmpdir or markdown stdout.

Usage:
    python3 csv_to_view.py <csv_file> --out <tmpdir> --format <html|markdown>
"""

from __future__ import annotations

import argparse
import csv
import json
import os
import shutil
import sys

# Interactive tables stay usable above this cap; extra rows set truncated=true.
ROW_CAP = 10_000


def find_csv_assets(script_path: str) -> str | None:
    """Locate packaged ``ext/csv`` assets from runtime or source-tree layout.

    Runtime (materialized ``{wyvern_share}``)::

        {wyvern_share}/scripts/ext/csv_to_view.py
        {wyvern_share}/ext/csv/...

    Source tree (pytest / direct invocation)::

        <repo>/scripts/ext/csv_to_view.py
        <repo>/share/wyvern/ext/csv/...
    """
    script_dir = os.path.dirname(os.path.abspath(script_path))
    # scripts/ext -> {wyvern_share} (runtime) or <repo> (source tree)
    root = os.path.dirname(os.path.dirname(script_dir))
    runtime = os.path.join(root, "ext", "csv")
    if os.path.isdir(runtime):
        return runtime
    source = os.path.join(root, "share", "wyvern", "ext", "csv")
    if os.path.isdir(source):
        return source
    return None


def parse_csv(path: str, row_cap: int = ROW_CAP) -> tuple[list[str], list[list[str]], bool]:
    """Read ``path`` and return ``(columns, rows, truncated)``.

    Extra cells beyond the header are dropped; short rows are padded with
    empty strings so every row matches the column count.
    """
    try:
        with open(path, newline="", encoding="utf-8-sig") as handle:
            reader = csv.reader(handle)
            try:
                header = next(reader)
            except StopIteration as exc:
                raise ValueError("CSV file is empty") from exc
            columns = [cell.strip() if isinstance(cell, str) else str(cell) for cell in header]
            if not columns or all(col == "" for col in columns):
                raise ValueError("CSV file has no header columns")
            rows: list[list[str]] = []
            truncated = False
            for raw in reader:
                if len(rows) >= row_cap:
                    truncated = True
                    break
                normalized = [raw[i] if i < len(raw) else "" for i in range(len(columns))]
                rows.append(normalized)
            return columns, rows, truncated
    except OSError as exc:
        raise ValueError(f"could not read CSV '{path}': {exc}") from exc
    except csv.Error as exc:
        raise ValueError(f"malformed CSV '{path}': {exc}") from exc


def write_rows_json(out_dir: str, columns: list[str], rows: list[list[str]], truncated: bool) -> None:
    data_dir = os.path.join(out_dir, "data")
    os.makedirs(data_dir, exist_ok=True)
    payload = {
        "columns": columns,
        "rows": rows,
        "meta": {"truncated": truncated, "row_count": len(rows)},
    }
    dest = os.path.join(data_dir, "rows.json")
    with open(dest, "w", encoding="utf-8") as handle:
        json.dump(payload, handle, ensure_ascii=False)


def copy_static_assets(out_dir: str, assets_root: str) -> None:
    mapping = (
        ("pages/view.html", os.path.join("pages", "view.html")),
        ("shared/table.js", os.path.join("shared", "table.js")),
        ("shared/table.css", os.path.join("shared", "table.css")),
    )
    for rel, dest_rel in mapping:
        src = os.path.join(assets_root, rel)
        if not os.path.isfile(src):
            raise ValueError(f"missing packaged CSV asset: {src}")
        dest = os.path.join(out_dir, dest_rel)
        os.makedirs(os.path.dirname(dest), exist_ok=True)
        shutil.copy2(src, dest)


def _md_cell(value: str) -> str:
    return str(value).replace("|", "\\|").replace("\n", " ").replace("\r", "")


def to_markdown(columns: list[str], rows: list[list[str]]) -> str:
    header = "| " + " | ".join(_md_cell(col) for col in columns) + " |"
    sep = "| " + " | ".join("---" for _ in columns) + " |"
    body = ["| " + " | ".join(_md_cell(cell) for cell in row) + " |" for row in rows]
    return "\n".join([header, sep, *body]) + "\n"


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Stage a CSV as a wyvern table view or markdown.")
    parser.add_argument("csv_file", help="Path to the CSV file")
    parser.add_argument("--out", required=True, help="Output tmpdir")
    parser.add_argument(
        "--format",
        required=True,
        choices=("html", "markdown"),
        help="html stages files; markdown writes a pipe table to stdout",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    try:
        columns, rows, truncated = parse_csv(args.csv_file)
        os.makedirs(args.out, exist_ok=True)
        if args.format == "html":
            assets = find_csv_assets(__file__)
            if assets is None:
                raise ValueError(
                    "could not locate share/wyvern/ext/csv assets relative to this script"
                )
            write_rows_json(args.out, columns, rows, truncated)
            copy_static_assets(args.out, assets)
        else:
            sys.stdout.write(to_markdown(columns, rows))
        return 0
    except ValueError as exc:
        print(f"csv_to_view: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
