#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///

"""Create a Typst Universe PR package tree for this repository."""

from __future__ import annotations

import argparse
import shutil
import tomllib
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_OUTPUT = REPO_ROOT / "package-pr"

ALWAYS_EXCLUDE = {
    ".git",
    ".gitattributes",
    ".gitignore",
    ".agents",
    ".codex",
    ".DS_Store",
    "package-pr",
}


def load_manifest() -> dict:
    with (REPO_ROOT / "typst.toml").open("rb") as manifest:
        return tomllib.load(manifest)


def excluded_paths(manifest: dict, output_dir: Path) -> set[Path]:
    excluded = {REPO_ROOT / path.lstrip("/") for path in ALWAYS_EXCLUDE}
    excluded.add(output_dir.resolve())

    for entry in manifest.get("package", {}).get("exclude", []):
        excluded.add((REPO_ROOT / entry.lstrip("/")).resolve())

    return excluded


def is_excluded(path: Path, excluded: set[Path]) -> bool:
    resolved = path.resolve()
    return any(resolved == item or item in resolved.parents for item in excluded)


def copy_package(output_dir: Path) -> Path:
    manifest = load_manifest()
    package = manifest["package"]
    package_dir = output_dir / "packages" / "preview" / package["name"] / package["version"]
    excluded = excluded_paths(manifest, output_dir)

    if output_dir.exists():
        shutil.rmtree(output_dir)
    package_dir.mkdir(parents=True)

    for source in sorted(REPO_ROOT.rglob("*")):
        if is_excluded(source, excluded) or source.is_dir():
            continue

        destination = package_dir / source.relative_to(REPO_ROOT)
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, destination)

    return package_dir


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Copy the package files into a Typst Universe PR tree."
    )
    parser.add_argument(
        "output",
        nargs="?",
        type=Path,
        default=DEFAULT_OUTPUT,
        help="Output folder to recreate. Defaults to ./package-pr.",
    )
    args = parser.parse_args()

    package_dir = copy_package(args.output.resolve())
    print(f"created {package_dir.relative_to(REPO_ROOT)}")


if __name__ == "__main__":
    main()
