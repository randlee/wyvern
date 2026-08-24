#!/usr/bin/env python3
"""Wrap XHTML fragments or documents in a wyvern report frame.

Usage:
    python3 xhtml_report.py --mode single --input FILE --title TITLE --out PATH
    python3 xhtml_report.py --mode array ...   # h.2 — exits non-zero
"""

from __future__ import annotations

import argparse
import html
import os
import re
import sys

BODY_OPEN = re.compile(r"<body\b[^>]*>", re.IGNORECASE)
BODY_CLOSE = re.compile(r"</body\s*>", re.IGNORECASE)


def extract_fragment(text: str) -> str:
    """Return body inner HTML, or the original fragment when no body is present."""
    stripped = text.lstrip("\ufeff").strip()
    open_match = BODY_OPEN.search(stripped)
    if open_match is None:
        return stripped
    close_match = BODY_CLOSE.search(stripped)
    if close_match is None or close_match.start() <= open_match.end():
        return stripped
    return stripped[open_match.end() : close_match.start()].strip()


def wrap_single(title: str, fragment: str) -> str:
    safe_title = html.escape(title, quote=True)
    return (
        "<!DOCTYPE html>\n"
        '<html lang="en">\n'
        "<head>\n"
        '  <meta charset="utf-8" />\n'
        '  <meta name="viewport" content="width=device-width, initial-scale=1" />\n'
        f"  <title>{safe_title}</title>\n"
        '  <link rel="stylesheet" href="/shared/report-base.css" />\n'
        "</head>\n"
        '<body class="report report--single">\n'
        f'  <main class="report-body">{fragment}</main>\n'
        "</body>\n"
        "</html>\n"
    )


def write_out(path: str, content: str) -> None:
    parent = os.path.dirname(os.path.abspath(path))
    if parent:
        os.makedirs(parent, exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(content)


def run_single(input_path: str, title: str, out_path: str) -> int:
    try:
        with open(input_path, encoding="utf-8") as handle:
            source = handle.read()
    except OSError as exc:
        print(f"xhtml_report: could not read input '{input_path}': {exc}", file=sys.stderr)
        return 1
    fragment = extract_fragment(source)
    write_out(out_path, wrap_single(title, fragment))
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Wrap XHTML panels in a report frame.")
    parser.add_argument("--mode", choices=("single", "array"), required=True)
    parser.add_argument("--input", help="XHTML fragment or document (single mode)")
    parser.add_argument("--title", default="", help="Report title")
    parser.add_argument("--out", required=True, help="Output HTML/XHTML path")
    parser.add_argument("--manifest", help="Reserved for --mode array (h.2)")
    parser.add_argument("--command-out", help="Reserved for --mode array (h.2)")
    args = parser.parse_args(argv)

    if args.mode == "array":
        print(
            "xhtml_report: --mode array is not implemented until sprint h.2",
            file=sys.stderr,
        )
        return 2
    if not args.input:
        print("xhtml_report: --input is required for --mode single", file=sys.stderr)
        return 1
    return run_single(args.input, args.title or "Report", args.out)


if __name__ == "__main__":
    sys.exit(main())
