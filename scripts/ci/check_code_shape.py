#!/usr/bin/env python3
"""Enforce the engine's code-shape rules with a ratchet that only tightens.

Owner: docs/architecture/code-structure-and-conventions.md (rules R1, R2, R3, R5, R6, R7).

Every tracked Rust file under `crates/` is classified and measured:

- `entry`   `src/lib.rs` and `src/main.rs`: at most ENTRY_LIMIT lines (R2).
- `harness` files under `tests/`, `benches/`, `examples/`, or named `tests.rs` or `*_tests.rs`:
            at most HARNESS_LIMIT lines.
- `source`  everything else: at most SOURCE_LIMIT lines (R1).

Counted patterns, in entry and source files only, totalled per crate so that moving code between
files of a crate is neutral:

- `mod_rs`                    a module file named `mod.rs` (R3)
- `path_attribute`            `#[path = ...]` (R3)
- `string_error`              `Result<..., String>` (R5)
- `boxed_error`               `Box<dyn Error>` (R5)
- `too_many_arguments_allow`  an allowed `clippy::too_many_arguments` (R7)
- `sha2_crate`                any `sha2::` path outside the digest owner (R6); the owner's own
                              `rrd_core::digest::Sha256` hasher is the intended API and is not counted

The recorded baseline (`code_shape_baseline.json`) lists every file that broke a rule when the
rules landed, with its size or count at that time. The check fails when:

- a file not in the baseline breaks a rule;
- a listed file grows, or a listed count rises;
- a listed file shrank, a count fell, or an entry no longer applies. Run `--tighten` to record the
  improvement, so that it cannot be undone later.

`--tighten` only lowers numbers and removes entries; it never admits a new violation.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path

ENTRY_LIMIT = 200
SOURCE_LIMIT = 800
HARNESS_LIMIT = 1500
BASELINE = Path(__file__).with_name("code_shape_baseline.json")
DIGEST_OWNER = "crates/kernel/rrd-core/src/digest.rs"

PATTERNS: dict[str, re.Pattern[str]] = {
    "path_attribute": re.compile(r"#\[path\s*="),
    "string_error": re.compile(r"Result<[^;{}]*,\s*String\s*>"),
    "boxed_error": re.compile(r"Box<dyn\s+(?:std::error::)?Error"),
    "too_many_arguments_allow": re.compile(r"allow\([^)]*clippy::too_many_arguments"),
    "sha2_crate": re.compile(r"(?<![A-Za-z0-9_])sha2::"),
}


def crate_of(path: str) -> str:
    """The crate directory a file belongs to: pattern counts are kept per crate, so moving code
    between files of one crate is neutral while a new site anywhere in the crate is refused."""
    parts = path.split("/")
    if "src" in parts:
        return "/".join(parts[:parts.index("src")])
    for marker in ("tests", "benches", "examples"):
        if marker in parts:
            return "/".join(parts[:parts.index(marker)])
    return "/".join(parts[:-1])


def classify(path: str) -> str:
    parts = path.split("/")
    name = parts[-1]
    if any(part in ("tests", "benches", "examples") for part in parts[:-1]) or name == "tests.rs" or name.endswith("_tests.rs"):
        return "harness"
    if len(parts) >= 2 and parts[-2] == "src" and name in ("lib.rs", "main.rs"):
        return "entry"
    return "source"


def limit_for(kind: str) -> int:
    return {"entry": ENTRY_LIMIT, "harness": HARNESS_LIMIT, "source": SOURCE_LIMIT}[kind]


@dataclass
class Shape:
    oversize: dict[str, int] = field(default_factory=dict)
    patterns: dict[str, dict[str, int]] = field(default_factory=dict)

    def to_json(self) -> dict:
        return {
            "version": 1,
            "limits": {"entry": ENTRY_LIMIT, "source": SOURCE_LIMIT, "harness": HARNESS_LIMIT},
            "oversize": dict(sorted(self.oversize.items())),
            "patterns": {name: dict(sorted(counts.items())) for name, counts in sorted(self.patterns.items()) if counts},
        }

    @classmethod
    def from_json(cls, data: dict) -> "Shape":
        return cls(oversize=dict(data.get("oversize", {})),
                   patterns={name: dict(counts) for name, counts in data.get("patterns", {}).items()})


def measure(files: dict[str, str]) -> Shape:
    """Measure `{repository-relative path: text}`; pure, so the rules are tested without Git."""
    shape = Shape()
    for path, text in files.items():
        kind = classify(path)
        lines = text.count("\n") + (0 if text.endswith("\n") or not text else 1)
        if lines > limit_for(kind):
            shape.oversize[path] = lines
        if kind == "harness":
            continue
        counts: dict[str, int] = {}
        if path.endswith("/mod.rs"):
            counts["mod_rs"] = 1
        for name, pattern in PATTERNS.items():
            if name == "sha2_crate" and path == DIGEST_OWNER:
                continue
            found = len(pattern.findall(text))
            if found:
                counts[name] = found
        for name, count in counts.items():
            owner = crate_of(path)
            shape.patterns.setdefault(name, {})[owner] = shape.patterns.get(name, {}).get(owner, 0) + count
    return shape


def compare(baseline: Shape, current: Shape) -> list[str]:
    """Every difference that fails the check, in a stable order."""
    problems: list[str] = []
    for path, lines in sorted(current.oversize.items()):
        limit = limit_for(classify(path))
        recorded = baseline.oversize.get(path)
        if recorded is None:
            problems.append(f"{path}: {lines} lines exceeds the {classify(path)} limit of {limit}; split it")
        elif lines > recorded:
            problems.append(f"{path}: grew from {recorded} to {lines} lines; the limit is {limit} and a listed file may only shrink")
        elif lines < recorded:
            problems.append(f"{path}: shrank from {recorded} to {lines} lines; run `check_code_shape.py --tighten` to lock it in")
    for path in sorted(set(baseline.oversize) - set(current.oversize)):
        problems.append(f"{path}: no longer over its limit (or removed); run `check_code_shape.py --tighten`")
    names = sorted(set(baseline.patterns) | set(current.patterns))
    for name in names:
        before, now = baseline.patterns.get(name, {}), current.patterns.get(name, {})
        for path in sorted(set(before) | set(now)):
            recorded, count = before.get(path, 0), now.get(path, 0)
            if count > recorded:
                problems.append(f"{path} (crate): {name} rose from {recorded} to {count}; this pattern may only be removed")
            elif count < recorded:
                problems.append(f"{path} (crate): {name} fell from {recorded} to {count}; run `check_code_shape.py --tighten`")
    return problems


def tighten(baseline: Shape, current: Shape) -> Shape:
    """Lower every recorded number to the current one; never raise or add."""
    tightened = Shape()
    for path, recorded in baseline.oversize.items():
        if path in current.oversize:
            tightened.oversize[path] = min(recorded, current.oversize[path])
    for name, counts in baseline.patterns.items():
        for path, recorded in counts.items():
            count = current.patterns.get(name, {}).get(path, 0)
            if count:
                tightened.patterns.setdefault(name, {})[path] = min(recorded, count)
    return tightened


def tracked_rust(root: Path) -> dict[str, str]:
    listed = subprocess.run(["git", "ls-files", "-z", "--", "crates/*.rs"], cwd=root, check=True,
                            capture_output=True).stdout.decode("utf-8").split("\0")
    return {path: (root / path).read_text(encoding="utf-8") for path in listed if path}


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--tighten", action="store_true", help="record improvements; never admits a violation")
    mode.add_argument("--init", action="store_true", help="write the first baseline; refused when one exists")
    args = parser.parse_args(argv)
    root = Path(__file__).resolve().parents[2]
    current = measure(tracked_rust(root))
    if args.init:
        if BASELINE.exists():
            print("code-shape: a baseline exists; use --tighten", file=sys.stderr)
            return 2
        BASELINE.write_text(json.dumps(current.to_json(), indent=1) + "\n", encoding="utf-8", newline="\n")
        print(f"code-shape: baseline written with {len(current.oversize)} oversize files")
        return 0
    baseline = Shape.from_json(json.loads(BASELINE.read_text(encoding="utf-8")))
    if args.tighten:
        tightened = tighten(baseline, current)
        BASELINE.write_text(json.dumps(tightened.to_json(), indent=1) + "\n", encoding="utf-8", newline="\n")
        remaining = compare(tightened, current)
        for problem in remaining:
            print(f"code-shape: {problem}", file=sys.stderr)
        return 1 if remaining else 0
    problems = compare(baseline, current)
    for problem in problems:
        print(f"::error::code-shape: {problem}")
    if problems:
        return 1
    total = sum(len(counts) for counts in baseline.patterns.values())
    print(f"code-shape: OK: {len(baseline.oversize)} listed oversize files and {total} listed pattern sites, none grown")
    return 0


if __name__ == "__main__":
    sys.exit(main())
