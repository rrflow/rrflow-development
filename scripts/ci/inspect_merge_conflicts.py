#!/usr/bin/env python3
"""Inspect two Git commits without changing the index, worktree or refs.

Git's merge-tree --write-tree may write unreachable objects. This report does
not resolve conflicts, authorize a merge, or replace candidate/queue checks.
"""
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

from build_execution_inventory import GENERATED_PROJECTIONS

OBJECT_ID = re.compile(r"(?:[0-9a-f]{40}|[0-9a-f]{64})\Z")


def commit_id(root: Path, revision: str) -> str:
    result = subprocess.run(
        ["git", "rev-parse", "--verify", "--end-of-options", f"{revision}^{{commit}}"],
        cwd=root, capture_output=True, text=True,
    )
    value = result.stdout.strip()
    if result.returncode != 0 or not OBJECT_ID.fullmatch(value):
        raise ValueError("revision does not resolve to exactly one commit")
    return value


def parse_merge_tree(status: int, output: bytes) -> tuple[str, list[str]]:
    if status not in (0, 1) or not output.endswith(b"\0"):
        raise ValueError("Git merge-tree failed or returned malformed output")
    try:
        fields = output.decode("utf-8", errors="strict").split("\0")
    except UnicodeError as error:
        raise ValueError("Git returned a non-UTF-8 conflict path") from error
    if not OBJECT_ID.fullmatch(fields[0]) or fields[-1] != "":
        raise ValueError("Git returned an invalid merge-tree identity")
    paths = fields[1:-1]
    if status == 0 and paths:
        raise ValueError("Git merge status and conflict paths disagree")
    if any(not path for path in paths) or len(paths) != len(set(paths)):
        raise ValueError("Git returned empty or duplicate conflict paths")
    return fields[0], paths


def inspect(root: Path, base: str, head: str) -> dict:
    base_id = commit_id(root, base)
    head_id = commit_id(root, head)
    drivers = subprocess.run(
        ["git", "config", "--get-regexp", r"^merge\..*\.driver$"],
        cwd=root, capture_output=True,
    )
    if drivers.returncode == 0:
        raise ValueError("configured external merge drivers require manual review")
    if drivers.returncode != 1:
        raise ValueError("cannot inspect configured merge drivers")
    result = subprocess.run(
        ["git", "merge-tree", "--write-tree", "--name-only", "--no-messages", "-z", base_id, head_id],
        cwd=root, capture_output=True,
    )
    tree, paths = parse_merge_tree(result.returncode, result.stdout)
    return {
        "version": 1,
        "base": base_id,
        "head": head_id,
        "merge_tree": tree,
        "status": "clean" if result.returncode == 0 else "conflicted",
        "conflict_paths_may_be_incomplete": result.returncode == 1,
        "conflicts": [
            {"path": path, "generator": GENERATED_PROJECTIONS.get(path)}
            for path in paths
        ],
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("base", help="base revision to resolve to a commit")
    parser.add_argument("head", help="candidate revision to resolve to a commit")
    parser.add_argument("--repo", type=Path, default=Path(__file__).resolve().parents[2])
    args = parser.parse_args()
    try:
        print(json.dumps(inspect(args.repo, args.base, args.head), indent=2))
    except (OSError, ValueError) as error:
        print(f"merge inspection failed: {error}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
