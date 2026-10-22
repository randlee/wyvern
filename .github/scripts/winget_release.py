#!/usr/bin/env python3
"""WinGet manifest helpers: derive nested zip paths and patch komac output."""
from __future__ import annotations

import argparse
import hashlib
import json
import zipfile
from io import BytesIO
from pathlib import Path
from urllib.request import Request, urlopen

import yaml


def archive_stem(archive_prefix: str, version: str, target: str) -> str:
    return f"{archive_prefix}_{version}_{target}"


def nested_relative_file_path(
    archive_prefix: str, version: str, target: str, nested_binary: str
) -> str:
    binary = nested_binary.removeprefix("/").replace("\\", "/")
    return f"{archive_stem(archive_prefix, version, target)}/{binary}"


def release_asset_filename(archive_prefix: str, version: str, target: str, archive: str) -> str:
    return f"{archive_stem(archive_prefix, version, target)}.{archive}"


def download_release_asset(url: str) -> bytes:
    request = Request(url, headers={"User-Agent": "wyvern-winget-publish/1.0"})
    with urlopen(request, timeout=120) as response:
        return response.read()


def verify_zip_layout(payload: bytes, relative_path: str) -> None:
    normalized = relative_path.replace("\\", "/")
    with zipfile.ZipFile(BytesIO(payload)) as archive:
        if normalized not in archive.namelist():
            sample = "\n".join(archive.namelist()[:8])
            raise SystemExit(
                f"release zip missing nested path {normalized!r}; sample entries:\n{sample}"
            )


def patch_installer_manifest(path: Path, relative_path: str, portable_alias: str) -> None:
    data = yaml.safe_load(path.read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        raise SystemExit(f"{path}: installer manifest must be a mapping")
    nested_files = [
        {
            "RelativeFilePath": relative_path.replace("\\", "/"),
            "PortableCommandAlias": portable_alias,
        }
    ]
    data["NestedInstallerType"] = data.get("NestedInstallerType") or "portable"
    data["NestedInstallerFiles"] = nested_files
    path.write_text(yaml.safe_dump(data, sort_keys=False, allow_unicode=True), encoding="utf-8")


def installer_manifest_path(output_dir: Path, identifier: str, version: str) -> Path:
    first = identifier[0].lower()
    segments = identifier.split(".")
    return (
        output_dir
        / "manifests"
        / first
        / Path(*segments)
        / version
        / f"{identifier}.installer.yaml"
    )


def cmd_prepare_vars(args: argparse.Namespace) -> int:
    config = json.loads(Path(args.channel_config).read_text(encoding="utf-8"))
    project = config["project"]
    channel = config["channel"]
    nested_binary = channel["nested_binary"]
    portable_alias = channel["portable_command_alias"]
    target_name = channel["installer_target"]
    target = config["release_targets"][target_name]
    version = args.version
    filename = release_asset_filename(
        project["archive_prefix"], version, target_name, target["archive"]
    )
    url = (
        f"https://github.com/{args.repository}/releases/download/{args.tag}/{filename}"
    )
    payload = download_release_asset(url)
    relative_path = nested_relative_file_path(
        project["archive_prefix"], version, target_name, nested_binary
    )
    verify_zip_layout(payload, relative_path)
    sha256 = hashlib.file_digest(BytesIO(payload), "sha256").hexdigest().upper()
    result = {
        "identifier": channel["identifier"],
        "version": version,
        "installer_url": url,
        "installer_sha256": sha256,
        "relative_file_path": relative_path,
        "portable_command_alias": portable_alias,
    }
    Path(args.output).write_text(json.dumps(result, separators=(",", ":")), encoding="utf-8")
    return 0


def cmd_patch_installer(args: argparse.Namespace) -> int:
    vars_path = Path(args.vars)
    plan = json.loads(vars_path.read_text(encoding="utf-8"))
    path = installer_manifest_path(
        Path(args.output_dir), plan["identifier"], plan["version"]
    )
    if not path.is_file():
        raise SystemExit(f"komac did not emit installer manifest at {path}")
    patch_installer_manifest(
        path, plan["relative_file_path"], plan["portable_command_alias"]
    )
    text = path.read_text(encoding="utf-8")
    expected = plan["relative_file_path"]
    if expected not in text:
        raise SystemExit(f"patched manifest at {path} does not contain {expected!r}")
    if "PortableCommandAlias" not in text:
        raise SystemExit(f"patched manifest at {path} missing PortableCommandAlias")
    return 0


def cmd_verify_installer(args: argparse.Namespace) -> int:
    plan = json.loads(Path(args.vars).read_text(encoding="utf-8"))
    path = Path(args.installer)
    text = path.read_text(encoding="utf-8")
    if plan["relative_file_path"] not in text:
        raise SystemExit("installer RelativeFilePath does not match release layout")
    if f"PortableCommandAlias: {plan['portable_command_alias']}" not in text.replace(
        '"', ""
    ):
        if plan["portable_command_alias"] not in text:
            raise SystemExit("installer manifest missing PortableCommandAlias")
    if plan["installer_sha256"].lower() not in text.lower():
        raise SystemExit("installer manifest SHA256 does not match release asset")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)

    prepare = sub.add_parser("prepare-vars")
    prepare.add_argument("--channel-config", required=True)
    prepare.add_argument("--repository", required=True)
    prepare.add_argument("--tag", required=True)
    prepare.add_argument("--version", required=True)
    prepare.add_argument("--output", required=True)
    prepare.set_defaults(func=cmd_prepare_vars)

    patch = sub.add_parser("patch-installer")
    patch.add_argument("--output-dir", required=True)
    patch.add_argument("--vars", required=True)
    patch.set_defaults(func=cmd_patch_installer)

    verify = sub.add_parser("verify-installer")
    verify.add_argument("--installer", required=True)
    verify.add_argument("--vars", required=True)
    verify.set_defaults(func=cmd_verify_installer)

    args = parser.parse_args()
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
