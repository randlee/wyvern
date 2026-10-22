"""WinGet nested portable path helpers."""
from __future__ import annotations

import sys
import zipfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from io import BytesIO

import pytest
import yaml

from winget_release import (
    nested_relative_file_path,
    patch_installer_manifest,
    verify_zip_layout,
)


def test_nested_relative_file_path_matches_release_layout() -> None:
    path = nested_relative_file_path(
        "wyvern", "0.8.0", "x86_64-pc-windows-msvc", "bin/wyvern.exe"
    )
    assert path == "wyvern_0.8.0_x86_64-pc-windows-msvc/bin/wyvern.exe"


def test_verify_zip_layout_accepts_matching_entry() -> None:
    payload = BytesIO()
    with zipfile.ZipFile(payload, "w") as archive:
        archive.writestr(
            "wyvern_0.8.0_x86_64-pc-windows-msvc/bin/wyvern.exe", b"stub"
        )
    verify_zip_layout(payload.getvalue(), "wyvern_0.8.0_x86_64-pc-windows-msvc/bin/wyvern.exe")


def test_verify_zip_layout_rejects_missing_entry() -> None:
    payload = BytesIO()
    with zipfile.ZipFile(payload, "w") as archive:
        archive.writestr(
            "wyvern_0.7.0_x86_64-pc-windows-msvc/bin/wyvern.exe", b"stub"
        )
    with pytest.raises(SystemExit, match="missing nested path"):
        verify_zip_layout(
            payload.getvalue(),
            "wyvern_0.8.0_x86_64-pc-windows-msvc/bin/wyvern.exe",
        )


def test_patch_installer_manifest_sets_alias_and_path(tmp_path: Path) -> None:
    installer = tmp_path / "randlee.wyvern.installer.yaml"
    installer.write_text(
        yaml.safe_dump(
            {
                "PackageIdentifier": "randlee.wyvern",
                "PackageVersion": "0.8.0",
                "InstallerType": "zip",
                "NestedInstallerType": "portable",
                "NestedInstallerFiles": [
                    {
                        "RelativeFilePath": "wyvern_0.7.0_x86_64-pc-windows-msvc/bin/wyvern.exe"
                    }
                ],
            },
            sort_keys=False,
        ),
        encoding="utf-8",
    )
    patch_installer_manifest(
        installer,
        "wyvern_0.8.0_x86_64-pc-windows-msvc/bin/wyvern.exe",
        "wyvern",
    )
    data = yaml.safe_load(installer.read_text(encoding="utf-8"))
    nested = data["NestedInstallerFiles"][0]
    assert nested["RelativeFilePath"] == "wyvern_0.8.0_x86_64-pc-windows-msvc/bin/wyvern.exe"
    assert nested["PortableCommandAlias"] == "wyvern"
