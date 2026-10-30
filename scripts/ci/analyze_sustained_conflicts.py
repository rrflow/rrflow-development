#!/usr/bin/env python3
"""Analyze verified C-07 client intervals without claiming commit causality.

The campaign verifier remains responsible for integrity and qualification.
This tool reads already retained segment files and synthetic conflict traces;
its output does not prove which operation advanced an engine cursor.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
from pathlib import Path
from typing import Any

MAX_FILE_BYTES = 8 * 1024 * 1024
MAX_TOTAL_BYTES = 128 * 1024 * 1024
MAX_SEGMENTS = 4096
MAX_ATTEMPTS = 100_000
MAX_TRACES = 32
MAX_OVERLAPS_PER_CONFLICT = 64
TRACE_PREFIX = "c07_runtime_conflict_trace "
TRACE = re.compile(
    r"c07_runtime_conflict_trace request_sha256=([0-9a-f]{64}) "
    r"read_cursor=(0|[1-9][0-9]{0,19}) expected_cursor=(0|[1-9][0-9]{0,19}) "
    r"actual_cursor=(0|[1-9][0-9]{0,19})"
)
SEGMENT = re.compile(r"segment-([0-9]{20})\.json")
U64_MAX = (1 << 64) - 1


class AnalysisError(ValueError):
    """Incomplete, malformed or oversized analysis input."""


def read_bounded(path: Path, maximum: int) -> bytes:
    if maximum <= 0:
        raise AnalysisError("file bound must be positive")
    try:
        with path.open("rb") as source:
            data = source.read(maximum + 1)
    except OSError as error:
        raise AnalysisError(
            f"cannot read {path.name}: {error.__class__.__name__}"
        ) from error
    if len(data) > maximum:
        raise AnalysisError(f"input {path.name} exceeds the file bound")
    return data


def unsigned(value: Any, label: str) -> int:
    if type(value) is not int or value < 0 or value > U64_MAX:
        raise AnalysisError(f"{label} must be an unsigned 64-bit integer")
    return value


def analyze(
    evidence: Path,
    stderr: Path,
    *,
    max_file_bytes: int = MAX_FILE_BYTES,
    max_attempts: int = MAX_ATTEMPTS,
) -> dict[str, Any]:
    if max_attempts <= 0 or max_attempts > MAX_ATTEMPTS:
        raise AnalysisError("attempt bound is invalid")
    paths = []
    try:
        for path in evidence.iterdir():
            if SEGMENT.fullmatch(path.name):
                paths.append(path)
                if len(paths) > MAX_SEGMENTS:
                    raise AnalysisError("segment count exceeds the bound")
        paths.sort()
    except OSError as error:
        raise AnalysisError(
            f"cannot enumerate evidence: {error.__class__.__name__}"
        ) from error
    if not paths or len(paths) > MAX_SEGMENTS:
        raise AnalysisError("segment count is empty or exceeds the bound")
    attempts: list[dict[str, Any]] = []
    input_segments = []
    campaign_identity: str | None = None
    total_bytes = 0
    seen: set[str] = set()
    for index, path in enumerate(paths):
        number = int(SEGMENT.fullmatch(path.name).group(1))
        if number != index:
            raise AnalysisError("segment numbers must be contiguous from zero")
        raw = read_bounded(path, max_file_bytes)
        total_bytes += len(raw)
        if total_bytes > MAX_TOTAL_BYTES:
            raise AnalysisError("total evidence bytes exceed the bound")
        try:
            segment = json.loads(raw)
        except (UnicodeDecodeError, json.JSONDecodeError) as error:
            raise AnalysisError(f"segment {number} is not valid JSON") from error
        if (
            not isinstance(segment, dict)
            or type(segment.get("index")) is not int
            or segment.get("index") != number
        ):
            raise AnalysisError(f"segment {number} has the wrong index")
        if (
            type(segment.get("format_version")) is not int
            or segment["format_version"] != 1
        ):
            raise AnalysisError(f"segment {number} has an unsupported format")
        campaign = segment.get("campaign_sha256")
        point = segment.get("point")
        if (
            not isinstance(campaign, str)
            or re.fullmatch(r"[0-9a-f]{64}", campaign) is None
        ):
            raise AnalysisError(f"segment {number} has an invalid campaign digest")
        if campaign_identity is None:
            campaign_identity = campaign
        elif campaign_identity != campaign:
            raise AnalysisError("segments bind different campaigns")
        if not isinstance(point, str) or not point or len(point.encode("utf-8")) > 4096:
            raise AnalysisError(f"segment {number} has an invalid operating point")
        point_identity = hashlib.sha256(point.encode("utf-8")).hexdigest()
        seed = unsigned(segment.get("seed"), "segment seed")
        phase = segment.get("phase")
        rows = segment.get("attempts")
        if (
            not isinstance(phase, str)
            or phase not in {"warmup", "measured", "burst", "drain", "soak"}
            or not isinstance(rows, list)
        ):
            raise AnalysisError(f"segment {number} has invalid phase or attempts")
        if len(attempts) + len(rows) > max_attempts:
            raise AnalysisError("attempt count exceeds the bound")
        input_segments.append(
            {"index": number, "sha256": hashlib.sha256(raw).hexdigest()}
        )
        for row in rows:
            if not isinstance(row, dict):
                raise AnalysisError("attempt must be an object")
            logical_id = row.get("logical_id")
            kind, outcome = row.get("kind"), row.get("outcome")
            if (
                not isinstance(logical_id, str)
                or not logical_id
                or len(logical_id.encode("utf-8")) > 4096
            ):
                raise AnalysisError("attempt identity is missing or too long")
            if not isinstance(kind, str) or not isinstance(outcome, str):
                raise AnalysisError("attempt kind and outcome must be strings")
            if kind not in {
                "mixed_write",
                "point_read",
                "range_read",
                "maintenance",
            } or outcome not in {
                "acknowledged",
                "rejected",
                "cancelled",
                "failed",
                "uncertain",
            }:
                raise AnalysisError("attempt kind or outcome is unsupported")
            identity = hashlib.sha256(logical_id.encode("utf-8")).hexdigest()
            if identity in seen:
                raise AnalysisError("attempt identity occurs more than once")
            seen.add(identity)
            start = unsigned(row.get("started_ns"), "attempt start")
            end = unsigned(row.get("finished_ns"), "attempt finish")
            if end < start:
                raise AnalysisError("attempt finishes before it starts")
            attempts.append(
                {
                    "logical_id_sha256": identity,
                    "segment": number,
                    "phase": phase,
                    "point_sha256": point_identity,
                    "seed": seed,
                    "sequence": unsigned(row.get("sequence"), "attempt sequence"),
                    "kind": kind,
                    "outcome": outcome,
                    "started_ns": start,
                    "finished_ns": end,
                }
            )

    raw_log = read_bounded(stderr, max_file_bytes)
    if total_bytes + len(raw_log) > MAX_TOTAL_BYTES:
        raise AnalysisError("total evidence bytes exceed the bound")
    try:
        lines = raw_log.decode("utf-8").splitlines()
    except UnicodeDecodeError as error:
        raise AnalysisError("trace log is not UTF-8") from error
    traces: dict[str, tuple[int, int, int]] = {}
    for line in lines:
        if not line.startswith(TRACE_PREFIX):
            continue
        match = TRACE.fullmatch(line)
        if match is None:
            raise AnalysisError("malformed conflict trace")
        identity = match.group(1)
        if identity in traces or len(traces) >= MAX_TRACES:
            raise AnalysisError("duplicate conflict trace or trace bound exceeded")
        numbers = tuple(
            unsigned(int(value), "trace cursor") for value in match.groups()[1:]
        )
        traces[identity] = numbers

    by_id = {row["logical_id_sha256"]: row for row in attempts}
    conflicts = []
    unmatched = []
    for identity, cursors in sorted(traces.items()):
        row = by_id.get(identity)
        if row is None:
            unmatched.append(identity)
            continue
        if row["outcome"] != "rejected":
            raise AnalysisError("conflict trace matches an attempt not marked rejected")
        overlap = [
            other
            for other in attempts
            if other is not row
            and other["point_sha256"] == row["point_sha256"]
            and other["seed"] == row["seed"]
            and other["started_ns"] <= row["finished_ns"]
            and row["started_ns"] <= other["finished_ns"]
        ]
        overlap.sort(
            key=lambda other: (other["started_ns"], other["logical_id_sha256"])
        )
        conflicts.append(
            {
                **row,
                "read_cursor": cursors[0],
                "expected_cursor": cursors[1],
                "actual_cursor": cursors[2],
                "overlap_count": len(overlap),
                "overlaps_truncated": len(overlap) > MAX_OVERLAPS_PER_CONFLICT,
                "overlaps": overlap[:MAX_OVERLAPS_PER_CONFLICT],
            }
        )
    conflicts.sort(key=lambda row: (row["started_ns"], row["logical_id_sha256"]))
    return {
        "format_version": 1,
        "analysis_only": True,
        "limitation": "Client response intervals do not establish commit order or cause; independently verify campaign evidence.",
        "campaign_sha256": campaign_identity,
        "segments": input_segments,
        "stderr_sha256": hashlib.sha256(raw_log).hexdigest(),
        "attempts": len(attempts),
        "conflicts": conflicts,
        "unmatched_trace_digests": unmatched,
        "rejected_without_trace": sum(
            row["outcome"] == "rejected" and row["logical_id_sha256"] not in traces
            for row in attempts
        ),
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "evidence", type=Path, help="verified campaign evidence directory"
    )
    parser.add_argument(
        "stderr", type=Path, help="retained bounded synthetic conflict trace log"
    )
    args = parser.parse_args()
    try:
        result = analyze(args.evidence, args.stderr)
    except AnalysisError as error:
        parser.exit(1, f"analysis failed: {error}\n")
    print(json.dumps(result, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
