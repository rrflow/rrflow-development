"""Build immutable offline review packets from explicitly exported paired outcomes.

This exchange tool has no engine, training, review-approval or activation authority.
Inputs must already be authorized and redacted; byte identity is not authenticity.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import tempfile

MAX_FILE_BYTES = 10 * 1024 * 1024
MAX_TOTAL_BYTES = 32 * 1024 * 1024
MAX_ROW_BYTES = 64 * 1024
MAX_TASKS = 10000
MAX_CANDIDATES = 16
SHA256 = re.compile(r"[0-9a-f]{64}\Z")


def canonical(value: object) -> bytes:
    return (json.dumps(value, sort_keys=True, separators=(",", ":"),
                       ensure_ascii=True, allow_nan=False) + "\n").encode("utf-8")


def digest(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def strict_json(value: bytes) -> object:
    def pairs(items: list) -> dict:
        result = {}
        for key, item in items:
            if key in result:
                raise ValueError("duplicate JSON key")
            result[key] = item
        return result

    def invalid(_: str) -> None:
        raise ValueError("non-finite JSON number")

    try:
        parsed = json.loads(value.decode("utf-8"), object_pairs_hook=pairs,
                            parse_constant=invalid)
        canonical(parsed)  # Also reject exponent overflow and excessive nesting.
        return parsed
    except (UnicodeError, RecursionError) as error:
        raise ValueError("invalid UTF-8 or excessive JSON nesting") from error


def fields(value: object, required: set[str], optional: set[str] | None = None) -> dict:
    if not isinstance(value, dict) or not required <= value.keys() or value.keys() - required - (optional or set()):
        raise ValueError("missing or unsupported exchange fields")
    return value


def text(value: object) -> str:
    if not isinstance(value, str) or not value.strip() or len(value) > 2048:
        raise ValueError("require a nonempty bounded string")
    return value


def identity(value: object) -> str:
    if not isinstance(value, str) or not SHA256.fullmatch(value):
        raise ValueError("require a lowercase SHA-256 identity")
    return value


def read_bounded(path: Path, limit: int = MAX_FILE_BYTES) -> bytes:
    # Exports are operator-owned regular files, not adversarial live directories.
    if not path.is_file():
        raise ValueError("input must be a regular file")
    with path.open("rb") as stream:
        content = stream.read(limit + 1)
    if len(content) > limit:
        raise ValueError("input byte limit exceeded")
    return content


def read_reference(root: Path, reference: object) -> bytes:
    ref = fields(reference, {"path", "sha256"})
    name = text(ref["path"])
    relative = PurePosixPath(name)
    if "\\" in name or ":" in name or relative.is_absolute() or any(part in {"", ".", ".."} for part in name.split("/")):
        raise ValueError("require a contained relative POSIX path")
    path = root.joinpath(*relative.parts)
    if not path.resolve().is_relative_to(root.resolve()):
        raise ValueError("reference escapes export directory")
    content = read_bounded(path)
    if digest(content) != identity(ref["sha256"]):
        raise ValueError("referenced bytes changed")
    return content


def rows(content: bytes) -> list[dict]:
    lines = content.splitlines()
    if not lines or len(lines) > MAX_TASKS or not content.endswith(b"\n"):
        raise ValueError("require bounded, nonempty, newline-terminated JSONL")
    result = []
    seen = set()
    for line in lines:
        if not line or len(line) > MAX_ROW_BYTES:
            raise ValueError("empty or oversized JSONL row")
        row = strict_json(line)
        if not isinstance(row, dict):
            raise ValueError("require JSONL objects")
        task_id = text(row.get("taskId"))
        if task_id in seen:
            raise ValueError("duplicate task identity")
        seen.add(task_id)
        result.append(row)
    return result


def build_packet(manifest_path: Path) -> dict:
    manifest_bytes = read_bounded(manifest_path)
    manifest = fields(strict_json(manifest_bytes), {"schemaVersion", "purpose", "tasks", "candidates"})
    if type(manifest["schemaVersion"]) is not int or manifest["schemaVersion"] != 1:
        raise ValueError("unsupported exchange version")
    purpose = text(manifest["purpose"])
    candidates = manifest["candidates"]
    if not isinstance(candidates, list) or not 2 <= len(candidates) <= MAX_CANDIDATES:
        raise ValueError("require between two and sixteen candidates")
    task_bytes = read_reference(manifest_path.parent, manifest["tasks"])
    tasks = rows(task_bytes)
    for task in tasks:
        fields(task, {"taskId", "project", "sourceSha256", "input", "labelOrigin", "provisionalLabel"})
        text(task["project"])
        identity(task["sourceSha256"])
        text(task["input"])
        text(task["labelOrigin"])
        if task["provisionalLabel"] is not None:
            text(task["provisionalLabel"])
    task_ids = {task["taskId"] for task in tasks}
    total = len(manifest_bytes) + len(task_bytes)
    outputs: dict[str, dict[str, dict]] = {task_id: {} for task_id in task_ids}
    candidate_ids = set()
    for candidate in candidates:
        fields(candidate, {"candidateId", "artifactSha256", "outcomes"})
        candidate_id = text(candidate["candidateId"])
        identity(candidate["artifactSha256"])
        if candidate_id in candidate_ids:
            raise ValueError("duplicate candidate identity")
        candidate_ids.add(candidate_id)
        outcome_bytes = read_reference(manifest_path.parent, candidate["outcomes"])
        total += len(outcome_bytes)
        if total > MAX_TOTAL_BYTES:
            raise ValueError("aggregate input byte limit exceeded")
        outcomes = rows(outcome_bytes)
        if {row["taskId"] for row in outcomes} != task_ids:
            raise ValueError("every candidate must retain exactly every task outcome")
        for outcome in outcomes:
            fields(outcome, {"taskId", "status", "output", "verification"})
            if outcome["status"] not in ("success", "error", "abstained"):
                raise ValueError("unsupported outcome status")
            verification = fields(outcome["verification"], {"state", "basis"})
            if verification["state"] not in ("unknown", "pass", "fail", "disputed"):
                raise ValueError("unsupported verification state")
            text(verification["basis"])
            outputs[outcome["taskId"]][candidate_id] = outcome
    return {
        "schemaVersion": 1,
        "purpose": purpose,
        "manifestSha256": digest(manifest_bytes),
        "toolSha256": digest(Path(__file__).read_bytes()),
        "sourceManifest": manifest,
        "trainingAdmitted": False,
        "deploymentQualified": False,
        "reviewState": "pending_independent_review",
        "tasks": [{"task": task, "outcomes": outputs[task["taskId"]],
                   "reviewState": "pending_independent_review"} for task in tasks],
    }


def publish(packet: dict, output_dir: Path) -> Path:
    content = canonical(packet)
    output_dir.mkdir(parents=True, exist_ok=True)
    destination = output_dir / (digest(content) + ".json")
    # Publish a complete file without replacing an earlier packet. A crash before
    # linking can leave an unreferenced temporary file, never a partial packet.
    temporary = None
    try:
        with tempfile.NamedTemporaryFile(dir=output_dir, prefix=".pending-", delete=False) as stream:
            temporary = Path(stream.name)
            stream.write(content)
            stream.flush()
            os.fsync(stream.fileno())
        try:
            os.link(temporary, destination)
        except FileExistsError:
            if destination.is_symlink() or read_bounded(destination, len(content)) != content:
                raise ValueError("existing packet identity collision or corruption")
        return destination
    finally:
        if temporary is not None:
            temporary.unlink(missing_ok=True)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args()
    try:
        packet = build_packet(args.manifest)
        output = publish(packet, args.output_dir)
    except (ValueError, OSError) as error:
        parser.exit(1, f"outcome review rejected: {error}\n")
    print(json.dumps({"packetSha256": output.stem, "tasks": len(packet["tasks"]),
                      "candidates": len(packet["sourceManifest"]["candidates"]),
                      "trainingAdmitted": False, "deploymentQualified": False}))


if __name__ == "__main__":
    main()
