#!/usr/bin/env python3

import contextlib
import hashlib
import json
import os
import platform
import shutil
import subprocess
import sys
from collections.abc import Sequence
from pathlib import Path

is_windows = platform.system() == "Windows"

root_path = Path(__file__).resolve().parent
dist_path = root_path.joinpath("dist")
staging_root_path = root_path.joinpath("staging")

app_name = "fersk"
bin_name = "fersk"


def get_version() -> str:
    with open(root_path.joinpath("version.json"), "r", encoding="utf-8") as f:
        vobj = json.load(f)

    return vobj["FullVersion"]


version = get_version()


@contextlib.contextmanager
def cd(path: Path | str):
    old_cwd = Path.cwd()

    os.chdir(path)

    try:
        yield
    finally:
        os.chdir(old_cwd)


def rmdir(path: Path | str):
    path = Path(path)

    with contextlib.suppress(FileNotFoundError):
        shutil.rmtree(path)


def rmfile(path: Path | str):
    path = Path(path)

    path.unlink(missing_ok=True)


def cmd(description: str, args: Sequence[str | Path]):
    print(f"--- {description} ---")
    subprocess.run(args, check=True)


def create_tarball(archive_name: str, path: Path | str):
    path = Path(path)

    archive_file = f"{archive_name}.tar.zst"
    archive_path = dist_path.joinpath(archive_file)

    # Delete old archive
    rmfile(archive_path)

    with cd(path):
        cmd("Compressing archive", ["tar", "acf", archive_path, "."])

    with open(archive_path, "rb") as f:
        checksum = hashlib.file_digest(f, "sha256")

    with open(f"{archive_path}.sha256.txt", "w", encoding="utf-8") as f:
        f.write(checksum.hexdigest())


def create_zip(archive_name: str, path: Path | str):
    path = Path(path)

    archive_file = f"{archive_name}.zip"
    archive_path = dist_path.joinpath(dist_path, archive_file)

    # Delete old archive
    rmfile(archive_path)

    with cd(path):
        cmd("Compressing archive", ["7z", "a", "-mx9", archive_path, "*"])

    with open(archive_path, "rb") as f:
        checksum = hashlib.file_digest(f, "sha256")

    with open(f"{archive_path}.sha256.txt", "w", encoding="utf-8") as f:
        f.write(checksum.hexdigest())


def build(name: str, target: str):
    staging_path = staging_root_path.joinpath(name)

    # Delte old staging directory
    rmdir(staging_path)

    # Create staging directory
    staging_path.mkdir(parents=True, exist_ok=True)

    # Build
    print("--- BUILDING ---")
    cmd("Cargo build", ["cargo", "build", "--release", "--target", target])

    return staging_path


def copy_bin(staging_path: Path | str, target: str, bin_name: str):
    staging_path = Path(staging_path)

    print("--- COPYING BIN ---")
    shutil.copy2(
        root_path.joinpath("target", target, "release", bin_name), staging_path
    )


def build_linux(name: str, target: str):
    staging_path = build(name, target)

    # Copy files
    copy_bin(staging_path, target, bin_name)

    # Create tarball
    create_tarball(f"{app_name}-{version}-{name}", staging_path)


def build_windows(name: str, target: str):
    staging_path = build(name, target)

    # Copy files
    copy_bin(staging_path, target, f"{bin_name}.exe")

    # Create archive
    create_zip(f"{app_name}-{version}-{name}", staging_path)


def build_linux64():
    build_linux("linux-x86_64", "x86_64-unknown-linux-gnu")


def build_linux32():
    build_linux("linux-i686", "i686-unknown-linux-gnu")


def build_win64():
    if is_windows:
        target = "x86_64-pc-windows-msvc"
    else:
        target = "x86_64-pc-windows-gnu"

    build_windows("windows-x86_64", target)


def build_win32():
    if is_windows:
        target = "i686-pc-windows-msvc"
    else:
        target = "i686-pc-windows-gnu"

    build_windows("windows-i686", target)


def build_chocolatey():
    version_txt_path = dist_path.joinpath("VERSION.txt")

    with open(version_txt_path, "w", encoding="utf-8") as f:
        f.write(version)

    cmd(
        "Build Chocolatey package",
        [
            "choco",
            "pack",
            "chocolatey/package.nuspec",
            "--outputdirectory",
            dist_path,
            f"--version={version}",
        ],
    )

    rmfile(version_txt_path)


CONFIGURATIONS = {
    "linux64": {
        "build": build_linux64,
    },
    "linux32": {
        "build": build_linux32,
    },
    "win64": {
        "build": build_win64,
    },
    "win32": {
        "build": build_win32,
    },
    "chocolatey": {
        "build": build_chocolatey,
    },
}


def main() -> int:
    to_build = []
    for cfg_name in sys.argv[1:]:
        cfg = CONFIGURATIONS.get(cfg_name)
        if cfg is None:
            print(f"Configuration '{cfg_name}' does not exist.")
            continue

        to_build.append(cfg)

    if to_build:
        dist_path.mkdir(parents=True, exist_ok=True)

        for cfg in to_build:
            cfg["build"]()

    return 0


if __name__ == "__main__":
    sys.exit(main())
