#!/usr/bin/env python3
"""Export reviewed Markdown into one deterministic RRFlow knowledge package."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
import tempfile
import unicodedata
from pathlib import Path
from urllib.parse import unquote

CONTRACT_VERSION = 1
EXPORTER_ID = "rrflow-knowledge-exporter"
EXPORTER_VERSION = 1
DEFAULT_PACKAGE_ID = "rrflow-bootstrap-knowledge"
MAX_ID_BYTES = 128
MAX_SOURCE_PATH_BYTES = 4_096
MAX_COORDINATE_BYTES = 1_024
MAX_BODY_BYTES = 4 * 1024 * 1024
MAX_PACKAGE_BODY_BYTES = 512 * 1024 * 1024
MAX_RECORDS = 100_000
MAX_EXCLUSIONS = 100_000
MAX_REVISION_BYTES = 256

CLASSIFICATIONS = {
    "architecture": "architecture",
    "decisions": "decision",
    "evidence": "evidence",
    "guides": "guide",
    "history": "history",
    "objectives": "objective",
    "operations": "operations",
    "poam": "poam",
    "reference": "reference",
    "research": "research",
    "roadmap": "roadmap",
}
INLINE_LINK = re.compile(r"!?\[[^\]\n]*\]\(([^)\n]+)\)")
REFERENCE_LINK = re.compile(r"(?m)^\[[^\]\n]+\]:\s*(\S+)")
URI_SCHEME = re.compile(r"^[a-z][a-z0-9+.-]*:", re.IGNORECASE)
CANONICAL_ID = re.compile(r"^[a-z0-9][a-z0-9._-]*$")


class ExportError(ValueError):
    """The source tree cannot produce a valid deterministic package."""


def _utf8_length(value: str) -> int:
    return len(value.encode("utf-8"))


def _sha256(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def _frame(value: bytes) -> bytes:
    return len(value).to_bytes(8, "big") + value


def _domain(value: bytes) -> bytes:
    return value + b"\0"


def _canonical_id(value: str, field: str) -> str:
    if (
        not value
        or _utf8_length(value) > MAX_ID_BYTES
        or CANONICAL_ID.fullmatch(value) is None
    ):
        raise ExportError(f"{field} is not a bounded canonical identifier: {value!r}")
    return value


def _ascii_token(value: str, field: str, maximum: int) -> str:
    encoded = value.encode("utf-8")
    if (
        not encoded
        or len(encoded) > maximum
        or value.strip() != value
        or any(byte < 0x21 or byte > 0x7E for byte in encoded)
    ):
        raise ExportError(f"{field} is not a bounded printable ASCII token")
    return value


def _source_path(value: str) -> str:
    if (
        not value
        or _utf8_length(value) > MAX_SOURCE_PATH_BYTES
        or value.startswith("/")
        or value.endswith("/")
        or "\\" in value
        or "\0" in value
        or any(unicodedata.category(character) == "Cc" for character in value)
    ):
        raise ExportError(f"unsafe or non-normalized source path: {value!r}")
    if any(segment in {"", ".", ".."} for segment in value.split("/")):
        raise ExportError(f"unsafe or non-normalized source path: {value!r}")
    return value


def _coordinate(value: str, field: str) -> str:
    if (
        _utf8_length(value) > MAX_COORDINATE_BYTES
        or value.strip() != value
        or any(character in value for character in ("\0", "\\", "?", "#"))
        or not value.startswith("rrflow://")
    ):
        raise ExportError(f"{field} is not a canonical RRFlow coordinate")
    segments = value.removeprefix("rrflow://").split("/")
    if len(segments) < 4 or segments[1] != "data":
        raise ExportError(
            f"{field} must be rrflow://<instance>/data/<canonical-path>/<record-id>"
        )
    for segment in (segments[0], *segments[2:]):
        _canonical_id(segment, field)
    return value


def _provenance_bytes(provenance: dict[str, object]) -> bytes:
    return b"".join(
        (
            _frame(str(provenance["repository"]).encode()),
            _frame(str(provenance["revision"]).encode()),
            _frame(str(provenance["exporter"]).encode()),
            _frame(int(provenance["exporter_version"]).to_bytes(2, "big")),
        )
    )


def record_digest(record: dict[str, object]) -> str:
    """Calculate the exact KB-02 record digest."""
    encoded = b"".join(
        (
            _domain(b"rrflow-knowledge-record-v1"),
            _frame(int(record["contract_version"]).to_bytes(2, "big")),
            _frame(str(record["coordinate"]).encode()),
            _frame(str(record["source_path"]).encode()),
            _frame(str(record["classification"]).encode()),
            _frame(str(record["owner_coordinate"]).encode()),
            _frame(str(record["body"]).encode()),
            _frame(str(record["body_sha256"]).encode()),
            _frame(_provenance_bytes(record["provenance"])),
        )
    )
    return _sha256(encoded)


def _manifest_bytes(entry: dict[str, object]) -> bytes:
    disposition = entry["disposition"]
    values = [
        _frame(str(entry["source_path"]).encode()),
        _frame(str(entry["source_sha256"]).encode()),
        _frame(str(disposition["disposition"]).encode()),
    ]
    if disposition["disposition"] == "included":
        values.extend(
            (
                _frame(str(disposition["coordinate"]).encode()),
                _frame(str(disposition["classification"]).encode()),
                _frame(str(disposition["record_sha256"]).encode()),
            )
        )
    else:
        values.append(_frame(str(disposition["reason_code"]).encode()))
    return b"".join(values)


def _record_bytes(record: dict[str, object]) -> bytes:
    return b"".join(
        (
            _frame(int(record["contract_version"]).to_bytes(2, "big")),
            _frame(str(record["coordinate"]).encode()),
            _frame(str(record["source_path"]).encode()),
            _frame(str(record["classification"]).encode()),
            _frame(str(record["owner_coordinate"]).encode()),
            _frame(str(record["body"]).encode()),
            _frame(str(record["body_sha256"]).encode()),
            _frame(_provenance_bytes(record["provenance"])),
            _frame(str(record["record_sha256"]).encode()),
        )
    )


def _exclusion_bytes(exclusion: dict[str, object]) -> bytes:
    return b"".join(
        _frame(str(exclusion[field]).encode())
        for field in ("source_path", "source_sha256", "reason_code", "reason")
    )


def _collection(values: list[dict[str, object]], encode: object) -> bytes:
    return _frame(len(values).to_bytes(8, "big")) + b"".join(
        _frame(encode(value)) for value in values
    )


def package_digest(package: dict[str, object]) -> str:
    """Calculate the exact KB-02 package digest."""
    encoded = b"".join(
        (
            _domain(b"rrflow-knowledge-package-v1"),
            _frame(int(package["contract_version"]).to_bytes(2, "big")),
            _frame(str(package["id"]).encode()),
            _frame(_provenance_bytes(package["provenance"])),
            _collection(package["manifest"], _manifest_bytes),
            _collection(package["records"], _record_bytes),
            _collection(package["exclusions"], _exclusion_bytes),
        )
    )
    return _sha256(encoded)


def _normalize(raw: bytes, source_path: str) -> str:
    try:
        body = raw.decode("utf-8")
    except UnicodeDecodeError as error:
        raise ExportError(f"{source_path}: source is not UTF-8") from error
    body = body.replace("\r\n", "\n").replace("\r", "\n")
    if "\0" in body:
        raise ExportError(f"{source_path}: source contains NUL")
    if not body.endswith("\n"):
        body += "\n"
    if not body.strip():
        raise ExportError(f"{source_path}: source is empty")
    if _utf8_length(body) > MAX_BODY_BYTES:
        raise ExportError(f"{source_path}: normalized body exceeds the record bound")
    return body


def _header_field(body: str, name: str) -> str | None:
    prefix = f"{name}:"
    for line in body.splitlines()[:12]:
        normalized = line.replace("**", "").strip()
        if normalized.casefold().startswith(prefix.casefold()):
            value = normalized[len(prefix) :].strip()
            return value or None
    return None


def _header_coordinate(body: str) -> str | None:
    value = _header_field(body, "Coordinate")
    return None if value is None else value.strip("`")


def _local_target(raw: str) -> str | None:
    target = raw.strip()
    if target.startswith("<") and ">" in target:
        target = target[1 : target.index(">")]
    else:
        target = target.split(maxsplit=1)[0]
    target = unquote(target.split("#", 1)[0].split("?", 1)[0])
    if not target or target.startswith("#") or URI_SCHEME.match(target):
        return None
    return target


def _owner_index(relative: Path) -> Path:
    if relative == Path("docs/README.md"):
        return relative
    if relative.name == "README.md":
        return relative.parent.parent / "README.md"
    return relative.parent / "README.md"


def _index_links(index: Path, record: Path, bodies: dict[str, str]) -> bool:
    index_key = index.as_posix()
    body = bodies.get(index_key)
    if body is None:
        return False
    accepted = {record}
    if record.name == "README.md":
        accepted.add(record.parent)
    for raw in (*INLINE_LINK.findall(body), *REFERENCE_LINK.findall(body)):
        target = _local_target(raw)
        if target is None:
            continue
        resolved = Path(os.path.normpath(index.parent / target))
        if resolved in accepted:
            return True
    return False


def _discover(root: Path) -> list[Path]:
    candidates = [
        path for path in (root / "README.md", root / "SPEC.md") if path.is_file()
    ]
    docs = root / "docs"
    if docs.is_dir():
        candidates.extend(path for path in docs.rglob("*.md") if path.is_file())
    resolved_root = root.resolve()
    for path in candidates:
        if not path.resolve().is_relative_to(resolved_root):
            raise ExportError(f"source symlink escapes repository: {path}")
    return sorted(
        candidates, key=lambda path: path.relative_to(root).as_posix().encode()
    )


def _exclusion(
    source_path: str, source_sha256: str, reason_code: str, reason: str
) -> dict[str, object]:
    _canonical_id(reason_code, "knowledge exclusion reason_code")
    return {
        "source_path": source_path,
        "source_sha256": source_sha256,
        "reason_code": reason_code,
        "reason": reason,
    }


def build_package(
    root: Path,
    *,
    repository: str,
    revision: str,
    package_id: str = DEFAULT_PACKAGE_ID,
) -> dict[str, object]:
    """Build a package in memory without modifying its Markdown sources."""
    root = root.resolve()
    _canonical_id(repository, "knowledge provenance repository")
    _canonical_id(package_id, "knowledge package id")
    _ascii_token(revision, "knowledge provenance revision", MAX_REVISION_BYTES)
    candidates = _discover(root)
    if not candidates:
        raise ExportError("no bootstrap Markdown sources were discovered")

    raw_before = {
        path.relative_to(root).as_posix(): path.read_bytes() for path in candidates
    }
    bodies = {
        source_path: _normalize(raw, source_path)
        for source_path, raw in raw_before.items()
    }
    inventory = {
        source_path: _sha256(body.encode()) for source_path, body in bodies.items()
    }
    provenance = {
        "repository": repository,
        "revision": revision,
        "exporter": EXPORTER_ID,
        "exporter_version": EXPORTER_VERSION,
    }
    records: list[dict[str, object]] = []
    exclusions: list[dict[str, object]] = []
    coordinates: dict[str, str] = {}

    for source_path in sorted(inventory, key=lambda value: value.encode()):
        _source_path(source_path)
        relative = Path(source_path)
        body = bodies[source_path]
        source_sha256 = inventory[source_path]
        coordinate = _header_coordinate(body)
        status = _header_field(body, "Status")

        if coordinate is None:
            if source_path == "README.md":
                exclusions.append(
                    _exclusion(
                        source_path,
                        source_sha256,
                        "bootstrap-product-entry",
                        "The root README is the bootstrap product portal, not a classified knowledge record.",
                    )
                )
            else:
                exclusions.append(
                    _exclusion(
                        source_path,
                        source_sha256,
                        "unclassified-record",
                        "The source has no accepted stable coordinate and remains outside the knowledge package.",
                    )
                )
            continue

        if status is None or not status.casefold().startswith(("active", "historical")):
            raise ExportError(
                f"{source_path}: a coordinated record must declare active or historical status"
            )
        _coordinate(coordinate, f"{source_path} Coordinate")
        prior = coordinates.get(coordinate)
        if prior is not None:
            raise ExportError(
                f"{source_path}: duplicate Coordinate also used by {prior}: {coordinate}"
            )
        coordinates[coordinate] = source_path

        if relative == Path("docs/README.md") or relative.name == "README.md":
            classification = "portal"
        elif len(relative.parts) >= 3 and relative.parts[0] == "docs":
            classification = CLASSIFICATIONS.get(relative.parts[1])
            if classification is None:
                raise ExportError(f"{source_path}: no frozen knowledge classification")
        else:
            raise ExportError(
                f"{source_path}: coordinated record is outside docs taxonomy"
            )

        owner_index = _owner_index(relative)
        owner_body = bodies.get(owner_index.as_posix())
        if owner_body is None:
            raise ExportError(
                f"{source_path}: owner index {owner_index.as_posix()} is absent"
            )
        owner_coordinate = _header_coordinate(owner_body)
        if owner_coordinate is None:
            raise ExportError(
                f"{source_path}: owner index {owner_index.as_posix()} has no Coordinate"
            )
        _coordinate(owner_coordinate, f"{source_path} owner Coordinate")
        if relative != Path("docs/README.md") and not _index_links(
            owner_index, relative, bodies
        ):
            raise ExportError(
                f"{source_path}: owner index {owner_index.as_posix()} does not link the record"
            )

        record = {
            "contract_version": CONTRACT_VERSION,
            "coordinate": coordinate,
            "source_path": source_path,
            "classification": classification,
            "owner_coordinate": owner_coordinate,
            "body": body,
            "body_sha256": source_sha256,
            "provenance": provenance.copy(),
            "record_sha256": "",
        }
        record["record_sha256"] = record_digest(record)
        records.append(record)

    records.sort(
        key=lambda value: (
            str(value["coordinate"]).encode(),
            str(value["source_path"]).encode(),
        )
    )
    exclusions.sort(key=lambda value: str(value["source_path"]).encode())
    if not records or len(records) > MAX_RECORDS:
        raise ExportError("knowledge record count is outside the contract bound")
    if len(exclusions) > MAX_EXCLUSIONS:
        raise ExportError("knowledge exclusion count exceeds the contract bound")
    if (
        sum(_utf8_length(str(record["body"])) for record in records)
        > MAX_PACKAGE_BODY_BYTES
    ):
        raise ExportError("knowledge package bodies exceed the package byte bound")

    included = {str(record["source_path"]): record for record in records}
    excluded = {str(exclusion["source_path"]): exclusion for exclusion in exclusions}
    manifest: list[dict[str, object]] = []
    for source_path in sorted(inventory, key=lambda value: value.encode()):
        if source_path in included:
            record = included[source_path]
            disposition = {
                "disposition": "included",
                "coordinate": record["coordinate"],
                "classification": record["classification"],
                "record_sha256": record["record_sha256"],
            }
        elif source_path in excluded:
            disposition = {
                "disposition": "excluded",
                "reason_code": excluded[source_path]["reason_code"],
            }
        else:
            raise ExportError(
                f"eligible source disappeared from classification: {source_path}"
            )
        manifest.append(
            {
                "source_path": source_path,
                "source_sha256": inventory[source_path],
                "disposition": disposition,
            }
        )

    if len(manifest) != len(inventory):
        raise ExportError(
            "manifest does not cover the independently discovered inventory"
        )
    package = {
        "contract_version": CONTRACT_VERSION,
        "id": package_id,
        "provenance": provenance,
        "manifest": manifest,
        "records": records,
        "exclusions": exclusions,
        "package_sha256": "",
    }
    package["package_sha256"] = package_digest(package)

    after = _discover(root)
    raw_after = {path.relative_to(root).as_posix(): path.read_bytes() for path in after}
    if raw_after != raw_before:
        raise ExportError(
            "bootstrap Markdown changed while the package was being built"
        )
    return package


def encode_package(package: dict[str, object]) -> bytes:
    """Encode one package as one canonical UTF-8 JSONL record."""
    return (
        json.dumps(package, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
        + "\n"
    ).encode("utf-8")


def _output_is_safe(root: Path, output: Path) -> bool:
    resolved_root = root.resolve()
    resolved_output = output.resolve()
    if not resolved_output.is_relative_to(resolved_root):
        return True
    relative = resolved_output.relative_to(resolved_root)
    result = subprocess.run(
        ["git", "check-ignore", "--quiet", "--", relative.as_posix()],
        cwd=resolved_root,
        check=False,
        capture_output=True,
    )
    return result.returncode == 0


def write_package(root: Path, output: Path, package: dict[str, object]) -> None:
    """Atomically write a generated package outside tracked documentation."""
    if output.suffix != ".jsonl":
        raise ExportError("knowledge package output must use the .jsonl suffix")
    if not _output_is_safe(root, output):
        raise ExportError(
            "knowledge package output inside the repository must be ignored by Git"
        )
    output.parent.mkdir(parents=True, exist_ok=True)
    temporary_name: str | None = None
    try:
        with tempfile.NamedTemporaryFile(
            mode="wb", dir=output.parent, prefix=f".{output.name}.", delete=False
        ) as temporary:
            temporary_name = temporary.name
            temporary.write(encode_package(package))
            temporary.flush()
            os.fsync(temporary.fileno())
        os.replace(temporary_name, output)
    finally:
        if temporary_name is not None and os.path.exists(temporary_name):
            os.unlink(temporary_name)


def _arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root",
        type=Path,
        default=Path(__file__).resolve().parents[2],
        help="repository root containing README.md and docs/",
    )
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--repository", default="rrflow")
    parser.add_argument("--revision", required=True)
    parser.add_argument("--package-id", default=DEFAULT_PACKAGE_ID)
    return parser.parse_args()


def main() -> int:
    arguments = _arguments()
    try:
        package = build_package(
            arguments.root,
            repository=arguments.repository,
            revision=arguments.revision,
            package_id=arguments.package_id,
        )
        write_package(arguments.root, arguments.output, package)
    except (ExportError, OSError) as error:
        print(f"knowledge-export: {error}", file=sys.stderr)
        return 1
    print(
        "knowledge-export: wrote "
        f"{len(package['records'])} records and {len(package['exclusions'])} exclusions "
        f"to {arguments.output} ({package['package_sha256']})"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
