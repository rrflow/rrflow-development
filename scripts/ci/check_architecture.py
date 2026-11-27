#!/usr/bin/env python3
"""Enforce the workspace architecture recorded in `architecture.toml`.

The manifest lists every crate once, in layering order, with the workspace crates it may use.
This checker reads it, reads each workspace member's own `Cargo.toml`, and fails on any
difference from the rules the manifest states at its top. It needs no build: it runs in seconds
on a clean checkout.

`pending` records are the ratchet: they name dependencies the target layering removes, and one
that is no longer used is an error until it is deleted, so the list of pending work only shrinks.
`--tighten` lowers the recorded size of `rrflow-core` to its current size.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
import tomllib
from dataclasses import dataclass, field
from pathlib import Path, PurePosixPath

ROOT = Path(__file__).resolve().parents[2]
MANIFEST = "architecture.toml"
DEPENDENCY_TABLES = {"normal": "dependencies", "build": "build-dependencies", "dev": "dev-dependencies"}
HARNESS_FOLDERS = ("tests", "benches", "examples")


def load_manifest(root: Path = ROOT) -> dict:
    with (root / MANIFEST).open("rb") as source:
        return tomllib.load(source)


@dataclass
class Member:
    """One workspace member as its own `Cargo.toml` describes it."""

    directory: str
    dependencies: dict[str, set[str]] = field(default_factory=dict)


def read_members(root: Path = ROOT) -> dict[str, Member]:
    """Every workspace member by package name, with its first-party dependencies by kind."""
    with (root / "Cargo.toml").open("rb") as source:
        directories = tomllib.load(source)["workspace"]["members"]
    documents = {}
    for directory in directories:
        with (root / directory / "Cargo.toml").open("rb") as source:
            document = tomllib.load(source)
        documents[document["package"]["name"]] = (PurePosixPath(directory).as_posix(), document)
    members = {}
    for name, (directory, document) in documents.items():
        tables = [document, *document.get("target", {}).values()]
        members[name] = Member(directory, {
            kind: {dependency for table in tables for dependency in table.get(key, {}) if dependency in documents}
            for kind, key in DEPENDENCY_TABLES.items()
        })
    return members


def group_of(directory: str, groups: list[dict]) -> dict | None:
    """The group whose folder holds `directory` directly: `<group>/<folder>` and nothing deeper."""
    parts = directory.split("/")
    if len(parts) != 2:
        return None
    return next((group for group in groups if group["folder"] == parts[0]), None)


def check_crates(manifest: dict, members: dict[str, Member]) -> list[str]:
    """Every rule about crates; pure, so the rules are tested without a workspace."""
    problems: list[str] = []
    groups = manifest["group"]
    banned = manifest["rules"]["banned_names"]
    crates: dict[str, dict] = {}
    order: dict[str, int] = {}
    for position, crate in enumerate(manifest["crate"]):
        if crate["name"] in crates:
            problems.append(f"{crate['name']}: listed twice in {MANIFEST}")
        crates[crate["name"]] = crate
        order[crate["name"]] = position

    for name in sorted(set(members) - set(crates)):
        problems.append(f"{name}: a workspace member that {MANIFEST} does not list; add it with what it uses")
    for name in sorted(set(crates) - set(members)):
        problems.append(f"{name}: listed in {MANIFEST} but not a workspace member; remove the entry")

    outside = {name for name, member in members.items()
               if (group_of(member.directory, groups) or {}).get("outside", False)}

    for name, crate in crates.items():
        member = members.get(name)
        if member is None:
            continue
        group = group_of(member.directory, groups)
        if group is None:
            folders = ", ".join(f"{group['folder']}/" for group in groups)
            problems.append(f"{name}: is at {member.directory}; a crate sits one folder below one of {folders}")
        else:
            folder = member.directory.split("/")[1]
            expected = group["prefix"] + folder
            if name != expected:
                problems.append(f"{name}: is at {member.directory}, so its package name is {expected}")
            if folder in banned:
                problems.append(f"{name}: a folder named `{folder}` says nothing about what it owns; name it for its subject")

        uses = set(crate.get("uses", []))
        actual = member.dependencies.get("normal", set()) | member.dependencies.get("build", set())
        for dependency in sorted(actual - uses):
            problems.append(f"{name}: depends on {dependency}, which its `uses` does not allow")
        for dependency in sorted(uses - actual):
            problems.append(f"{name}: `uses` lists {dependency}, which it does not depend on; remove it")
        for dependency in sorted(uses):
            if dependency in order and order[dependency] > order[name]:
                problems.append(f"{name}: uses {dependency}, which {MANIFEST} lists below it")
            if dependency in outside and name not in outside:
                problems.append(f"{name}: may use lab crate {dependency} only in its tests")
        for dependency in sorted(set(crate.get("pending", [])) - actual):
            problems.append(f"{name}: pending dependency {dependency} is gone; remove it from `pending`")

        exceptions = set(crate.get("standalone_exceptions", []))
        if not crate.get("standalone", False):
            if exceptions:
                problems.append(f"{name}: has standalone exceptions but is not standalone")
            continue
        found = set()
        for kind, dependencies in member.dependencies.items():
            for dependency in dependencies:
                if not crates.get(dependency, {}).get("standalone", False):
                    found.add(f"{kind}:{dependency}")
        for edge in sorted(found - exceptions):
            kind, dependency = edge.split(":", 1)
            problems.append(f"{name}: is standalone but has a {kind} dependency on {dependency}, which is not")
        for edge in sorted(exceptions - found):
            problems.append(f"{name}: standalone exception {edge} no longer applies; remove it")
    return problems


def check_module_names(paths: list[str], banned: list[str]) -> list[str]:
    """A source module, file or folder, named a banned word. Test, bench and example trees are not modules."""
    problems = []
    for path in sorted(paths):
        parts = path.split("/")
        if "src" not in parts or any(part in HARNESS_FOLDERS for part in parts):
            continue
        inside = parts[parts.index("src") + 1:]
        modules = [*inside[:-1], inside[-1].removesuffix(".rs")] if inside else []
        for module in modules:
            if module in banned:
                problems.append(f"{path}: a module named `{module}` says nothing about what it owns; name it for its subject")
                break
    return problems


def core_shape(directory: str, sources: dict[str, int]) -> tuple[set[str], int]:
    """The top-level modules of the crate at `directory` and the lines of source under its `src`."""
    prefix = f"{directory}/src/"
    modules, lines = set(), 0
    for path, count in sources.items():
        if not path.startswith(prefix):
            continue
        lines += count
        top = path[len(prefix):].split("/")[0].removesuffix(".rs")
        if top != "lib":
            modules.add(top)
    return modules, lines


def check_core(core: dict, directory: str, sources: dict[str, int]) -> list[str]:
    """`rrflow-core` holds exactly its recorded modules and never grows past its recorded size."""
    problems = []
    name = core["crate"]
    modules, lines = core_shape(directory, sources)
    for module in sorted(modules - set(core["modules"])):
        problems.append(f"{name}: has a module `{module}` that `[core]` does not list; "
                        "a type with one owner belongs in that owner's crate")
    for module in sorted(set(core["modules"]) - modules):
        problems.append(f"{name}: `[core]` lists module `{module}`, which is gone; remove it")
    if lines > core["max_lines"]:
        problems.append(f"{name}: has {lines} lines of source, above its recorded {core['max_lines']}; core does not grow")
    return problems


def summary(manifest: dict) -> str:
    crates = manifest["crate"]
    pending = sum(len(crate.get("pending", [])) for crate in crates)
    exceptions = sum(len(crate.get("standalone_exceptions", [])) for crate in crates)
    return (f"{len(crates)} crates; pending: {pending} dependencies to remove, "
            f"{exceptions} standalone exceptions; core limit {manifest['core']['max_lines']} lines")


def tracked_rust(root: Path, groups: list[dict]) -> list[str]:
    patterns = [f"{group['folder']}/*.rs" for group in groups]
    listed = subprocess.run(["git", "ls-files", "-z", "--cached", "--others", "--exclude-standard", "--", *patterns], cwd=root,
                            check=True, capture_output=True).stdout.decode("utf-8").split("\0")
    return [path for path in listed if path]


def line_counts(root: Path, paths: list[str]) -> dict[str, int]:
    return {path: len((root / path).read_text(encoding="utf-8").splitlines()) for path in paths}


def tighten(root: Path, lines: int) -> None:
    path = root / MANIFEST
    text = path.read_text(encoding="utf-8")
    path.write_text(re.sub(r"(?m)^max_lines = \d+$", f"max_lines = {lines}", text, count=1), encoding="utf-8", newline="\n")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--tighten", action="store_true", help="lower the recorded size of rrflow-core to its current size")
    arguments = parser.parse_args(argv)
    manifest = load_manifest()
    members = read_members()
    paths = tracked_rust(ROOT, manifest["group"])
    core = manifest["core"]
    core_directory = members[core["crate"]].directory
    core_sources = line_counts(ROOT, [path for path in paths if path.startswith(core_directory + "/src/")])
    if arguments.tighten:
        lines = core_shape(core_directory, core_sources)[1]
        if lines < core["max_lines"]:
            tighten(ROOT, lines)
            manifest = load_manifest()
            core = manifest["core"]
    problems = check_crates(manifest, members)
    problems += check_module_names(paths, manifest["rules"]["banned_names"])
    problems += check_core(core, core_directory, core_sources)
    for problem in problems:
        print(f"::error::architecture: {problem}")
    if problems:
        return 1
    print(f"architecture: OK: {summary(manifest)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
