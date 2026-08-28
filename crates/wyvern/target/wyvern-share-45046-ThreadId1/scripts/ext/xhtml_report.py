#!/usr/bin/env python3
"""Wrap XHTML fragments or documents in a wyvern report frame.

Usage:
    python3 xhtml_report.py --mode single --input FILE --title TITLE --out PATH
    python3 xhtml_report.py --mode array --manifest FILE --out PATH [--command-out PATH]
    python3 xhtml_report.py --mode review --manifest FILE --out PATH [--command-out PATH]
    python3 xhtml_report.py --manifest FILE --out PATH [--command-out PATH] [--force-mode review]
    python3 xhtml_report.py --validate-manifest FILE
"""

from __future__ import annotations

import argparse
import html
import json
import os
import re
import sys

BODY_OPEN = re.compile(r"<body\b[^>]*>", re.IGNORECASE)
BODY_CLOSE = re.compile(r"</body\s*>", re.IGNORECASE)

ALLOWED_MODES = frozenset({"view", "review"})
ALLOWED_ROLES = frozenset({"failure", "proposal", "info"})
ALLOWED_KEYS = frozenset({"title", "mode", "panels"})
PANEL_KEYS = frozenset({"path", "label", "role"})
MAX_PANELS = 32
MAX_HTML_BYTES = 4 * 1024 * 1024
COMMAND_PAGE = "pages/view.xhtml"


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


def pane_sections(panes: list[tuple[dict, str]]) -> str:
    sections: list[str] = []
    for panel, fragment in panes:
        path = str(panel["path"])
        role = panel.get("role")
        label = panel.get("label") or os.path.basename(path)
        classes = "pane pane--proposal" if role == "proposal" else "pane"
        role_attr = f' data-role="{html.escape(str(role), quote=True)}"' if role else ""
        sections.append(
            f'<section class="{classes}"{role_attr} '
            f'data-path="{html.escape(path, quote=True)}">\n'
            f'  <header class="pane-label">{html.escape(str(label))}</header>\n'
            f"  {fragment}\n"
            "</section>"
        )
    return "\n".join(sections)


def wrap_array(title: str, panes: list[tuple[dict, str]]) -> str:
    safe_title = html.escape(title, quote=True)
    body = pane_sections(panes)
    return (
        "<!DOCTYPE html>\n"
        '<html lang="en">\n'
        "<head>\n"
        '  <meta charset="utf-8" />\n'
        '  <meta name="viewport" content="width=device-width, initial-scale=1" />\n'
        f"  <title>{safe_title}</title>\n"
        '  <link rel="stylesheet" href="/shared/report-base.css" />\n'
        "</head>\n"
        '<body class="report report--array">\n'
        f'  <main class="report-body">\n{body}\n  </main>\n'
        "</body>\n"
        "</html>\n"
    )


def embed_manifest_json(manifest: dict, mode: str) -> str:
    payload = {
        "title": manifest["title"],
        "mode": mode,
        "panels": manifest["panels"],
    }
    # Escape '<' so a panel label cannot break out of the script element.
    encoded = json.dumps(payload, ensure_ascii=True).replace("<", "\\u003c")
    return encoded


def wrap_review(title: str, panes: list[tuple[dict, str]], manifest: dict, mode: str) -> str:
    safe_title = html.escape(title, quote=True)
    body = pane_sections(panes)
    manifest_json = embed_manifest_json(manifest, mode)
    return (
        "<!DOCTYPE html>\n"
        '<html lang="en">\n'
        "<head>\n"
        '  <meta charset="utf-8" />\n'
        '  <meta name="viewport" content="width=device-width, initial-scale=1" />\n'
        f"  <title>{safe_title}</title>\n"
        '  <link rel="stylesheet" href="/shared/report-base.css" />\n'
        "</head>\n"
        '<body class="report report--array report--review">\n'
        f'  <main class="report-body">\n{body}\n  </main>\n'
        '  <footer class="report-review" data-testid="report-review">\n'
        '    <label for="review-comments">Comments</label>\n'
        '    <textarea id="review-comments" data-testid="review-comments"></textarea>\n'
        "    <nav>\n"
        '      <button type="button" data-report-cancel data-testid="report-cancel">Cancel</button>\n'
        '      <button type="button" data-report-approve data-testid="report-approve">Approve</button>\n'
        "    </nav>\n"
        "  </footer>\n"
        f'  <script id="manifest-data" type="application/json">{manifest_json}</script>\n'
        '  <script src="/shared/report-review.js"></script>\n'
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


def is_safe_relative(rel: str) -> bool:
    if not rel or os.path.isabs(rel) or rel.startswith("/") or rel.startswith("~"):
        return False
    normalized = rel.replace("\\", "/")
    return all(part != ".." for part in normalized.split("/"))


def resolve_panel_path(manifest_dir: str, rel: str) -> str | None:
    if not is_safe_relative(rel):
        return None
    full = os.path.abspath(os.path.normpath(os.path.join(manifest_dir, rel)))
    root = os.path.abspath(manifest_dir)
    if full == root or not full.startswith(root + os.sep):
        return None
    return full


def validate_manifest_data(data: object) -> list[str]:
    if not isinstance(data, dict):
        return ["manifest must be a JSON object"]
    errors: list[str] = []
    unknown = sorted(set(data) - ALLOWED_KEYS)
    if unknown:
        errors.append(f"unknown fields: {', '.join(unknown)}")
    title = data.get("title")
    if not isinstance(title, str) or not title.strip():
        errors.append("title must be a non-empty string")
    mode = data.get("mode", "view")
    if mode not in ALLOWED_MODES:
        errors.append(f"mode must be 'view' or 'review' (got {mode!r})")
    panels = data.get("panels")
    if not isinstance(panels, list) or not panels:
        errors.append("panels must be a non-empty array")
        return errors
    if len(panels) > MAX_PANELS:
        errors.append(f"panels must have at most {MAX_PANELS} entries (got {len(panels)})")
    for index, panel in enumerate(panels):
        prefix = f"panels[{index}]"
        if not isinstance(panel, dict):
            errors.append(f"{prefix} must be an object")
            continue
        extra = sorted(set(panel) - PANEL_KEYS)
        if extra:
            errors.append(f"{prefix} unknown fields: {', '.join(extra)}")
        path = panel.get("path")
        if not isinstance(path, str) or not path:
            errors.append(f"{prefix}.path must be a non-empty string")
        elif not path.endswith(".xhtml"):
            errors.append(f"{prefix}.path must end with .xhtml (got {path!r})")
        elif not is_safe_relative(path):
            errors.append(f"{prefix}.path must be a relative .xhtml path (got {path!r})")
        if "label" in panel and not isinstance(panel.get("label"), str):
            errors.append(f"{prefix}.label must be a string")
        role = panel.get("role")
        if role is not None and role not in ALLOWED_ROLES:
            errors.append(f"{prefix}.role must be one of {sorted(ALLOWED_ROLES)} (got {role!r})")
    return errors


def load_manifest(path: str) -> tuple[dict | None, str | None]:
    try:
        with open(path, encoding="utf-8") as handle:
            data = json.load(handle)
    except FileNotFoundError:
        return None, f"xhtml_report: manifest not found: {path}"
    except OSError as exc:
        return None, f"xhtml_report: could not read manifest '{path}': {exc}"
    except json.JSONDecodeError as exc:
        return None, f"xhtml_report: invalid manifest JSON: {exc}"
    errors = validate_manifest_data(data)
    if errors:
        return None, "xhtml_report: invalid manifest: " + "; ".join(errors)
    if not isinstance(data, dict):
        return None, "xhtml_report: invalid manifest: manifest must be a JSON object"
    return data, None



def missing_panel_message(rel: str, resolved: str | None) -> str:
    if resolved is None:
        return f"xhtml_report: missing panel file '{rel}'"
    return f"xhtml_report: missing panel file '{rel}' ({resolved})"


def collect_panes(manifest: dict, manifest_dir: str) -> tuple[list[tuple[dict, str]] | None, str | None]:
    panes: list[tuple[dict, str]] = []
    for panel in manifest["panels"]:
        rel = panel["path"]
        resolved = resolve_panel_path(manifest_dir, rel)
        if resolved is None or not os.path.isfile(resolved):
            return None, missing_panel_message(rel, resolved)
        try:
            with open(resolved, encoding="utf-8") as handle:
                source = handle.read()
        except OSError as exc:
            return None, f"xhtml_report: could not read panel '{rel}': {exc}"
        panes.append((panel, extract_fragment(source)))
    return panes, None


def resolve_command_mode(manifest: dict, force_mode: str | None) -> str:
    if force_mode:
        return force_mode
    mode = manifest.get("mode", "view")
    return mode if mode in ALLOWED_MODES else "view"


def write_command_json(path: str, manifest: dict, mode: str) -> None:
    command = {
        "type": "report",
        "title": manifest["title"],
        "page": COMMAND_PAGE,
        "mode": mode,
        "panels": manifest["panels"],
    }
    parent = os.path.dirname(os.path.abspath(path))
    if parent:
        os.makedirs(parent, exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        json.dump(command, handle, indent=2)
        handle.write("\n")


def run_validate(manifest_path: str) -> int:
    manifest, err = load_manifest(manifest_path)
    if manifest is None:
        print(err or "xhtml_report: invalid manifest", file=sys.stderr)
        return 1
    manifest_dir = os.path.dirname(os.path.abspath(manifest_path)) or "."
    panes, pane_err = collect_panes(manifest, manifest_dir)
    if panes is None:
        print(pane_err or "xhtml_report: missing panel file", file=sys.stderr)
        return 1
    command_mode = resolve_command_mode(manifest, None)
    stitched = (
        wrap_review(manifest["title"], panes, manifest, command_mode)
        if command_mode == "review"
        else wrap_array(manifest["title"], panes)
    )
    if len(stitched.encode("utf-8")) > MAX_HTML_BYTES:
        print(
            f"xhtml_report: stitched HTML exceeds {MAX_HTML_BYTES} bytes",
            file=sys.stderr,
        )
        return 1
    return 0


def run_array(
    manifest_path: str,
    out_path: str,
    command_out: str | None,
    force_mode: str | None,
    frame_mode: str | None,
) -> int:
    manifest, err = load_manifest(manifest_path)
    if manifest is None:
        print(err or "xhtml_report: invalid manifest", file=sys.stderr)
        return 1
    manifest_dir = os.path.dirname(os.path.abspath(manifest_path)) or "."
    panes, pane_err = collect_panes(manifest, manifest_dir)
    if panes is None:
        print(pane_err or "xhtml_report: missing panel file", file=sys.stderr)
        return 1
    command_mode = resolve_command_mode(manifest, force_mode)
    use_review = frame_mode == "review" or command_mode == "review"
    html_out = (
        wrap_review(manifest["title"], panes, manifest, command_mode)
        if use_review
        else wrap_array(manifest["title"], panes)
    )
    if len(html_out.encode("utf-8")) > MAX_HTML_BYTES:
        print(
            f"xhtml_report: stitched HTML exceeds {MAX_HTML_BYTES} bytes",
            file=sys.stderr,
        )
        return 1
    write_out(out_path, html_out)
    if command_out:
        write_command_json(command_out, manifest, command_mode)
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Wrap XHTML panels in a report frame.")
    parser.add_argument("--mode", choices=("single", "array", "review"))
    parser.add_argument("--input", help="XHTML fragment or document (single mode)")
    parser.add_argument("--title", default="", help="Report title")
    parser.add_argument("--out", help="Output HTML/XHTML path")
    parser.add_argument("--manifest", help="Review manifest JSON (array or review mode)")
    parser.add_argument("--command-out", help="Write type=report command JSON")
    parser.add_argument(
        "--force-mode",
        choices=("view", "review"),
        help="Override manifest mode when writing report-command.json",
    )
    parser.add_argument(
        "--validate-manifest",
        metavar="PATH",
        help="Validate a review manifest (schema + panel files) and exit",
    )
    args = parser.parse_args(argv)

    if args.validate_manifest:
        return run_validate(args.validate_manifest)

    mode = args.mode
    if mode is None and args.manifest:
        mode = "array"
    if mode is None:
        print("xhtml_report: --mode or --manifest is required", file=sys.stderr)
        return 1
    if not args.out:
        print("xhtml_report: --out is required", file=sys.stderr)
        return 1
    if mode == "single":
        if not args.input:
            print("xhtml_report: --input is required for --mode single", file=sys.stderr)
            return 1
        return run_single(args.input, args.title or "Report", args.out)
    if not args.manifest:
        print("xhtml_report: --manifest is required for --mode array/review", file=sys.stderr)
        return 1
    return run_array(args.manifest, args.out, args.command_out, args.force_mode, mode)


if __name__ == "__main__":
    sys.exit(main())
