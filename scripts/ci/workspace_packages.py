#!/usr/bin/env python3
"""Which workspace package owns a repository path.

The root `Cargo.toml` lists every member's directory, the same list Cargo
builds from, and each member's own manifest names its package. A crate can
therefore sit at any depth under `crates/`, such as
`crates/evaluation/lab/rrflow-lab-core`. A path belongs to the member whose
directory is its longest prefix, so a nested package that is not a member,
such as a fuzz target, belongs to the member around it. Every script that
maps a path to a package asks here instead of counting path parts.
"""

from __future__ import annotations

import tomllib
from functools import lru_cache
from pathlib import Path, PurePosixPath

ROOT = Path(__file__).resolve().parents[2]


@lru_cache(maxsize=None)
def member_directories(root: Path = ROOT) -> dict[str, str]:
    """Each workspace member's directory, relative and in POSIX form, mapped to its package name."""
    with (root / "Cargo.toml").open("rb") as source:
        members = tomllib.load(source)["workspace"]["members"]
    directories = {}
    for member in members:
        if any(character in member for character in "*?["):
            raise ValueError(f"workspace member {member!r} is a glob; list members explicitly")
        with (root / member / "Cargo.toml").open("rb") as source:
            name = tomllib.load(source)["package"]["name"]
        directories[PurePosixPath(member).as_posix().rstrip("/")] = name
    return directories


def package_of(path: str, directories: dict[str, str]) -> str | None:
    """The package whose member directory is the longest prefix of `path`, if any."""
    posix = PurePosixPath(path.replace("\\", "/")).as_posix()
    owners = [
        directory
        for directory in directories
        if posix == directory or posix.startswith(directory + "/")
    ]
    return directories[max(owners, key=len)] if owners else None
