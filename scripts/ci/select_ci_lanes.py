#!/usr/bin/env python3
"""Choose which candidate CI lanes a change needs.

The candidate pipeline is proportional to the change. A documentation edit
runs the documentation checks; a script edit runs the script tests; a crate
edit tests that crate and compiles everything that depends on it, on Linux;
a change to storage, the installer, the native harness, the workflows, the
workspace manifest or lock file, or anything this module does not recognize
runs the full matrix. The engine lane never links the whole workspace in one
target graph, which the hosted runners cannot hold. Unknown paths
fail closed into the full lane.

The classifier is pure: it maps a list of repository-relative paths to lanes.
Dependents are resolved from `cargo metadata`, which the caller supplies or
this module runs. The workflow reads the printed `key=value` lines into job
outputs; `ci-gate` then requires every selected lane to succeed and every
unselected lane to be skipped.
"""

from __future__ import annotations

import argparse
import fnmatch
import json
import subprocess
import sys
from pathlib import Path, PurePosixPath

from workspace_packages import member_directories, package_of

ROOT = Path(__file__).resolve().parents[2]
LANES = ("full", "docs", "scripts", "engine", "sdk")

# Any match sends the change through the complete matrix.
FULL_PATTERNS = (
    ".github/workflows/*",
    ".github/workflows/**",
    "scripts/ci/*",
    "scripts/ci/**",
    ".cargo/**",
    ".devcontainer/**",
    ".gitmodules",
    "connectome",
    "rust-toolchain.toml",
    "Cargo.toml",
    "Cargo.lock",
    "VERSION",
    "deploy/**",
    "crates/kernel/**",
    "crates/persistence/**",
    # Transactional semantics and durable mutations require native recovery
    # qualification even when their source sits in the engine crate.
    "crates/authority/rrd-engine/src/engine/transaction*",
    "crates/authority/rrd-engine/src/engine/session_transactions.rs",
    "crates/authority/rrd-engine/src/engine/query_transaction.rs",
    "crates/authority/rrd-engine/src/engine/*mutation*",
    # Known engine modules that publish durable runtime or control effects.
    "crates/authority/rrd-engine/src/engine/attunement.rs",
    "crates/authority/rrd-engine/src/engine/backup.rs",
    "crates/authority/rrd-engine/src/engine/control.rs",
    "crates/authority/rrd-engine/src/engine/endpoint_configuration.rs",
    "crates/authority/rrd-engine/src/engine/estate_control.rs",
    "crates/authority/rrd-engine/src/engine/flight.rs",
    "crates/authority/rrd-engine/src/engine/function/**",
    "crates/authority/rrd-engine/src/engine/knowledge.rs",
    "crates/authority/rrd-engine/src/engine/observation_delivery/**",
    "crates/authority/rrd-engine/src/engine/observation_source.rs",
    "crates/authority/rrd-engine/src/engine/reasoning_budget/persistence.rs",
    "crates/authority/rrd-engine/src/engine/restore_publication.rs",
    "crates/authority/rrd-engine/src/engine/session.rs",
    "crates/authority/rrd-engine/src/engine/subscription.rs",
    "crates/authority/rrd-engine/src/engine/task.rs",
    "crates/authority/rrd-engine/src/engine/tail_repair/**",
    "crates/authority/rrd-engine/src/engine/tests/**",
    "crates/authority/rrd-engine/src/engine/query.rs",
    "crates/adapters/rrflow-cli/**",
    "crates/operations/rrd-maintenance/**",
    "crates/authority/rrd-engine/src/engine/installation.rs",
    "crates/authority/rrd-engine/src/engine/installation/**",
    "crates/authority/rrd-engine/src/engine/estate_layout.rs",
    "crates/authority/rrd-engine/src/engine/token_key.rs",
    "crates/authority/rrd-engine/src/engine/token_key/**",
    "crates/authority/rrd-engine/src/engine/maintenance.rs",
    "crates/authority/rrd-engine/src/engine/maintenance/**",
    "crates/authority/rrd-engine/src/engine/attunement/*supervisor*",
    "crates/authority/rrd-engine/src/engine/attunement/*supervisor*/**",
)
SDK_PATTERNS = ("sdks/**", "fixtures/**", "crates/transport/**")
ENGINE_PATTERNS = ("crates/**",)
SCRIPTS_PATTERNS = ("scripts/**", "eval/**")
DOCS_PATTERNS = (
    "docs/**",
    "*.md",
    "**/*.md",
    ".github/ISSUE_TEMPLATE/**",
    ".github/PULL_REQUEST_TEMPLATE.md",
    "LICENSE",
)


def matches(path: str, patterns: tuple[str, ...]) -> bool:
    posix = PurePosixPath(path).as_posix()
    for pattern in patterns:
        if fnmatch.fnmatchcase(posix, pattern):
            return True
        # fnmatch's `*` crosses `/`, so `a/**` also needs the directory prefix.
        if pattern.endswith("/**") and posix.startswith(pattern[:-2]):
            return True
    return False


def crate_of(path: str, root: Path = ROOT) -> str | None:
    """The workspace package that owns `path`, from the workspace member list at any depth."""
    if not path.startswith("crates/"):
        return None
    return package_of(path, member_directories(root))


def workspace_graph(metadata: dict) -> dict[str, set[str]]:
    """Reverse dependency edges between workspace members: package -> dependents."""
    members = {package["id"]: package for package in metadata["packages"]}
    member_ids = set(metadata.get("workspace_members", members))
    names = {package["name"] for pid, package in members.items() if pid in member_ids}
    dependents: dict[str, set[str]] = {name: set() for name in names}
    for pid, package in members.items():
        if pid not in member_ids:
            continue
        for dependency in package.get("dependencies", []):
            if dependency["name"] in names:
                dependents[dependency["name"]].add(package["name"])
    return dependents


def with_dependents(packages: set[str], dependents: dict[str, set[str]]) -> set[str]:
    selected = set(packages)
    frontier = list(packages)
    while frontier:
        for dependent in dependents.get(frontier.pop(), ()):
            if dependent not in selected:
                selected.add(dependent)
                frontier.append(dependent)
    return selected


def cargo_metadata(root: Path = ROOT) -> dict:
    completed = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1", "--locked"],
        cwd=root, check=True, capture_output=True, text=True,
    )
    return json.loads(completed.stdout)


def select(paths: list[str], *, metadata: dict | None = None, root: Path = ROOT,
           force_full: bool = False) -> dict[str, object]:
    """Return lane flags and the cargo package selection for the engine lane."""
    lanes = {lane: False for lane in LANES}
    packages: set[str] = set()
    unknown: list[str] = []
    for raw in paths:
        path = raw.strip().replace("\\", "/")
        if not path:
            continue
        if matches(path, FULL_PATTERNS):
            lanes["full"] = True
            continue
        known = False
        if matches(path, SDK_PATTERNS):
            lanes["sdk"] = True
            known = True
        if matches(path, ENGINE_PATTERNS):
            lanes["engine"] = True
            known = True
            crate = crate_of(path, root)
            if crate is None:
                # A crate directory this checkout does not know is unknown.
                lanes["full"] = True
                unknown.append(path)
            else:
                packages.add(crate)
        if matches(path, SCRIPTS_PATTERNS):
            lanes["scripts"] = True
            known = True
        if matches(path, DOCS_PATTERNS):
            lanes["docs"] = True
            known = True
        if not known:
            lanes["full"] = True
            unknown.append(path)
    if force_full or not paths:
        lanes["full"] = True
    selection, dependents = "", ""
    if lanes["full"]:
        # The full matrix already covers every light lane.
        for lane in ("docs", "scripts", "engine", "sdk"):
            lanes[lane] = False
    elif lanes["engine"] and packages:
        graph = workspace_graph(metadata if metadata is not None else cargo_metadata(root))
        closure = with_dependents(packages, graph) - packages
        selection = " ".join(f"-p {name}" for name in sorted(packages))
        dependents = " ".join(f"-p {name}" for name in sorted(closure))
    return {**lanes, "packages": selection, "dependents": dependents, "unknown": sorted(unknown)}


def changed_paths(base: str, head: str, root: Path = ROOT) -> list[str]:
    merge_base = subprocess.run(["git", "merge-base", base, head], cwd=root, check=True,
                                capture_output=True, text=True).stdout.strip()
    completed = subprocess.run(["git", "diff", "--name-only", merge_base, head], cwd=root,
                               check=True, capture_output=True, text=True)
    return [line for line in completed.stdout.splitlines() if line]


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--base", help="base revision; with --head, the diff decides the lanes")
    parser.add_argument("--head", default="HEAD")
    parser.add_argument("--full", action="store_true", help="select the full matrix regardless of paths")
    parser.add_argument("--paths-from", type=Path, help="read changed paths from a file, one per line")
    parser.add_argument("--github-output", type=Path, help="append key=value lines to this file")
    args = parser.parse_args(argv)
    if args.full:
        paths: list[str] = []
    elif args.paths_from is not None:
        paths = [line for line in args.paths_from.read_text(encoding="utf-8").splitlines() if line.strip()]
    elif args.base:
        paths = changed_paths(args.base, args.head)
    else:
        parser.error("one of --full, --paths-from or --base is required")
    result = select(paths, force_full=args.full)
    lines = [f"{lane}={'true' if result[lane] else 'false'}" for lane in LANES]
    lines.append(f"packages={result['packages']}")
    lines.append(f"dependents={result['dependents']}")
    for line in lines:
        print(line)
    if result["unknown"]:
        print("unknown paths routed to the full matrix:", *result["unknown"], sep="\n  ", file=sys.stderr)
    if args.github_output is not None:
        with args.github_output.open("a", encoding="utf-8") as handle:
            handle.write("\n".join(lines) + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
