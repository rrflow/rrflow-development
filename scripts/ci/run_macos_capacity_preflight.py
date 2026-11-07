#!/usr/bin/env python3
"""Run selected cases through the existing non-gating native capacity runner."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

from run_native_capacity import POINTS


def case_arguments(point: str, mode: str, repetitions: str) -> list[str]:
    if point != "all" and point not in POINTS:
        raise ValueError("unknown native capacity point")
    if mode not in ("both", "fresh", "existing"):
        raise ValueError("unknown native capacity mode")
    if repetitions not in ("1", "2", "3"):
        raise ValueError("repetitions must be 1, 2, or 3")
    arguments = [] if point == "all" else ["--point", point]
    modes = ("fresh", "existing") if mode == "both" else (mode,)
    for _ in range(int(repetitions)):
        for selected in modes:
            arguments.extend(("--mode", selected))
    return arguments


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--point", default="all")
    parser.add_argument("--mode", default="both")
    parser.add_argument("--repetitions", default="3")
    args = parser.parse_args()
    try:
        selected = case_arguments(args.point, args.mode, args.repetitions)
    except ValueError as error:
        parser.error(str(error))
    print(
        "native_capacity_preflight_selection="
        + json.dumps(
            {
                "point": args.point,
                "mode": args.mode,
                "repetitions": int(args.repetitions),
                "scope": "manual_preflight_not_candidate_ci",
            }
        ),
        flush=True,
    )
    owner = Path(__file__).with_name("run_native_capacity.py")
    return subprocess.run(
        [sys.executable, str(owner), *selected], check=False
    ).returncode


if __name__ == "__main__":
    raise SystemExit(main())
