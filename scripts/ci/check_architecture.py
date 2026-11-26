#!/usr/bin/env python3
"""Enforce the workspace architecture recorded in `crates/architecture.toml`.

Owner: docs/architecture/code-structure-and-conventions.md, sections 6.2, 6.11 and 6.12.

The manifest lists every crate's tier, folder, name and standing. This checker reads it, reads
each workspace member's own `Cargo.toml`, and fails on any difference from the rules the manifest
states at its top. It needs no build: it runs in seconds on a clean checkout.

A crate that has not reached its target name or folder carries `target_name` or `target_folder`.
Those records are the ratchet: a crate without one must already conform, and a record that no
longer applies is an error until it is removed, so the list of pending work can only shrink.
"""

from __future__ import annotations

import subprocess
import sys
import tomllib
from dataclasses import dataclass, field
from pathlib import Path, PurePosixPath

ROOT = Path(__file__).resolve().parents[2]
MANIFEST = "crates/architecture.toml"
EVALUATION = "evaluation"
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


def conforming_name(name: str, rules: dict) -> bool:
    return name.startswith(rules["component_prefix"]) or name in rules["daemon_crates"]


def check_crates(manifest: dict, members: dict[str, Member]) -> list[str]:
    """Every rule about crates; pure, so the rules are tested without a workspace."""
    problems: list[str] = []
    rules = manifest["rules"]
    tier_folders = {tier["id"]: tier["folders"] for tier in manifest["tier"]}
    crates: dict[str, dict] = {}
    order: dict[str, int] = {}
    for position, crate in enumerate(manifest["crate"]):
        if crate["name"] in crates:
            problems.append(f"{crate['name']}: listed twice in {MANIFEST}")
        crates[crate["name"]] = crate
        order[crate["name"]] = position

    for name in sorted(set(members) - set(crates)):
        problems.append(f"{name}: a workspace member that {MANIFEST} does not list; add it with its tier and folder")
    for name in sorted(set(crates) - set(members)):
        problems.append(f"{name}: listed in {MANIFEST} but not a workspace member; remove the entry")

    for name, crate in crates.items():
        member = members.get(name)
        if member is None:
            continue
        tier = crate["tier"]
        retiring = crate.get("retire", False)
        expected = f"{crate['folder']}/{name}"
        if member.directory != expected:
            problems.append(f"{name}: is at {member.directory} but {MANIFEST} places it at {expected}")

        if tier == EVALUATION:
            if not (crate["folder"] == "crates/evaluation" or crate["folder"].startswith("crates/evaluation/")):
                problems.append(f"{name}: an evaluation crate must sit under crates/evaluation")
            allowed_folders = [crate["folder"]]
        elif tier not in tier_folders:
            problems.append(f"{name}: tier {tier!r} is not declared in {MANIFEST}")
            continue
        else:
            allowed_folders = tier_folders[tier]

        target_name, target_folder = crate.get("target_name"), crate.get("target_folder")
        if retiring:
            if target_name or target_folder:
                problems.append(f"{name}: is marked retire, so it has no target name or folder")
        else:
            if conforming_name(name, rules) and target_name is None:
                pass
            elif target_name is None:
                problems.append(f"{name}: is not `{rules['component_prefix']}<name>` or a daemon crate; rename it or record `target_name`")
            elif target_name == name:
                problems.append(f"{name}: has reached its target name; remove `target_name`")
            elif not conforming_name(target_name, rules):
                problems.append(f"{name}: target name {target_name} is not `{rules['component_prefix']}<name>` or a daemon crate")
            elif target_name in members:
                problems.append(f"{name}: target name {target_name} is already a workspace member")

            if crate["folder"] in allowed_folders:
                if target_folder is not None:
                    problems.append(f"{name}: already sits in a folder of its tier; remove `target_folder`")
            elif target_folder is None:
                problems.append(f"{name}: {crate['folder']} is not a folder of tier {tier} ({', '.join(allowed_folders)}); move it or record `target_folder`")
            elif target_folder not in allowed_folders:
                problems.append(f"{name}: target folder {target_folder} is not a folder of tier {tier} ({', '.join(allowed_folders)})")

        for kind, dependencies in sorted(member.dependencies.items()):
            for dependency in sorted(dependencies):
                used = crates.get(dependency)
                if used is None:
                    continue
                if used.get("retire", False):
                    problems.append(f"{name}: depends on {dependency}, which is marked retire")
                if kind == "dev":
                    continue
                used_tier = used["tier"]
                if used_tier == EVALUATION:
                    if tier != EVALUATION:
                        problems.append(f"{name}: may use evaluation crate {dependency} only in its tests")
                    continue
                if tier == EVALUATION:
                    continue
                if used_tier > tier:
                    problems.append(f"{name} (tier {tier}): depends on {dependency} in the higher tier {used_tier}")
                elif used_tier == tier and order[dependency] > order[name]:
                    problems.append(f"{name}: depends on {dependency} of its own tier, which {MANIFEST} lists below it")

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


def summary(manifest: dict) -> str:
    crates = manifest["crate"]
    count = lambda key: sum(1 for crate in crates if crate.get(key))
    exceptions = sum(len(crate.get("standalone_exceptions", [])) for crate in crates)
    return (f"{len(crates)} crates; pending: {count('target_name')} renames, {count('target_folder')} moves, "
            f"{count('retire')} removals, {exceptions} standalone exceptions")


def tracked_rust(root: Path) -> list[str]:
    listed = subprocess.run(["git", "ls-files", "-z", "--cached", "--others", "--exclude-standard", "--", "crates/*.rs"], cwd=root,
                            check=True, capture_output=True).stdout.decode("utf-8").split("\0")
    return [path for path in listed if path]


def main() -> int:
    manifest = load_manifest()
    problems = check_crates(manifest, read_members())
    problems += check_module_names(tracked_rust(ROOT), manifest["rules"]["banned_module_names"])
    for problem in problems:
        print(f"::error::architecture: {problem}")
    if problems:
        return 1
    print(f"architecture: OK: {summary(manifest)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
