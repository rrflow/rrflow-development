#!/usr/bin/env python3
"""Render marked Markdown indexes and derive the coordinated-record link graph."""

from __future__ import annotations

import argparse
import json
import os
import re
import tempfile
from pathlib import Path
from urllib.parse import unquote

INDEX_START = "<!-- rrflow:generated-index:start -->"
INDEX_END = "<!-- rrflow:generated-index:end -->"
INDEX_START_LINE = re.compile(rf"(?m)^{re.escape(INDEX_START)}[ \t]*$")
INDEX_END_LINE = re.compile(rf"(?m)^{re.escape(INDEX_END)}[ \t]*$")
HEADING = re.compile(r"(?m)^#\s+(.+?)\s*#*\s*$")
INLINE_LINK = re.compile(r"!?\[[^\]\n]*\]\(([^)\n]+)\)")
REFERENCE_LINK = re.compile(r"(?m)^\[(?!\^)[^\]\n]+\]:\s*(\S+)")
URI_SCHEME = re.compile(r"^[a-z][a-z0-9+.-]*:", re.IGNORECASE)


class NavigationError(ValueError):
    """A coordinated record or generated-index boundary is malformed."""


def header_field(source: str, name: str) -> str | None:
    """Read one conventional record field from the first twelve lines."""
    prefix = f"{name}:"
    for line in source.splitlines()[:12]:
        normalized = line.replace("**", "").strip()
        if normalized.casefold().startswith(prefix.casefold()):
            value = normalized[len(prefix) :].strip()
            return value or None
    return None


def record_metadata(path: Path) -> dict[str, str]:
    """Return the minimum author-owned metadata used by generated navigation."""
    source = path.read_text(encoding="utf-8")
    heading = HEADING.search(source)
    title = None if heading is None else heading.group(1).strip()
    status = header_field(source, "Status")
    coordinate = header_field(source, "Coordinate")
    owner = header_field(source, "Owner") or header_field(source, "Superseded by")
    missing = [
        name
        for name, value in (
            ("title", title),
            ("Status", status),
            ("Coordinate", coordinate),
        )
        if value is None
    ]
    if missing:
        raise NavigationError(f"{path}: missing {', '.join(missing)}")
    assert title is not None and status is not None
    assert coordinate is not None
    coordinate = coordinate.strip("`")
    if not coordinate.startswith("rrflow://"):
        raise NavigationError(f"{path}: Coordinate is not an rrflow URI")
    return {
        "title": title,
        "status": status,
        "coordinate": coordinate,
        "owner": owner or "",
    }


def _index_children(index: Path) -> list[tuple[str, Path]]:
    children: list[tuple[str, Path]] = []
    for child in sorted(index.parent.iterdir(), key=lambda value: value.name.encode()):
        if child == index:
            continue
        target: str | None = None
        record: Path | None = None
        if child.is_file() and child.suffix.casefold() == ".md":
            target, record = child.name, child
        elif child.is_dir() and (child / "README.md").is_file():
            target, record = f"{child.name}/", child / "README.md"
        if record is None:
            continue
        source = record.read_text(encoding="utf-8")
        if header_field(source, "Coordinate") is not None:
            assert target is not None
            children.append((target, record))
    return children


def generated_index_body(index: Path) -> str:
    """Render immediate coordinated children without copying their content."""
    rows: list[str] = []
    seen_coordinates: dict[str, Path] = {}
    for target, record in _index_children(index):
        metadata = record_metadata(record)
        coordinate = metadata["coordinate"]
        prior = seen_coordinates.get(coordinate)
        if prior is not None:
            raise NavigationError(
                f"{index}: child Coordinate {coordinate} is duplicated by "
                f"{prior} and {record}"
            )
        seen_coordinates[coordinate] = record
        title = metadata["title"].replace("[", "\\[").replace("]", "\\]")
        rows.append(
            f"- [{title}]({target}) — "
            f"[`{coordinate}`]({coordinate}) — {metadata['status']}"
        )
    if not rows:
        raise NavigationError(f"{index}: marked index has no coordinated children")
    return "\n".join(rows)


def render_index(source: str, generated: str, path: Path) -> str:
    """Replace exactly one marked region while preserving all authored prose."""
    starts = list(INDEX_START_LINE.finditer(source))
    ends = list(INDEX_END_LINE.finditer(source))
    if len(starts) != 1 or len(ends) != 1:
        raise NavigationError(f"{path}: expected exactly one generated-index region")
    start = starts[0].end()
    end = ends[0].start()
    if start > end:
        raise NavigationError(f"{path}: generated-index markers are reversed")
    return source[:start] + "\n" + generated + "\n" + source[end:]


def marked_indexes(root: Path) -> list[Path]:
    """Discover opted-in indexes; an ordinary README remains fully authored."""
    docs = root / "docs"
    if not docs.is_dir():
        return []
    result: list[Path] = []
    for path in docs.rglob("README.md"):
        source = path.read_text(encoding="utf-8")
        if INDEX_START_LINE.search(source) or INDEX_END_LINE.search(source):
            result.append(path)
    return sorted(result, key=lambda value: value.relative_to(root).as_posix().encode())


def index_drift_failures(root: Path) -> list[str]:
    """Return deterministic drift errors without mutating the checkout."""
    failures: list[str] = []
    for index in marked_indexes(root):
        relative = index.relative_to(root).as_posix()
        try:
            source = index.read_text(encoding="utf-8")
            expected = render_index(source, generated_index_body(index), index)
        except (NavigationError, OSError) as error:
            failures.append(str(error))
            continue
        if expected != source:
            failures.append(f"{relative}: generated index is stale")
    return failures


def _atomic_write(path: Path, value: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary_name: str | None = None
    try:
        with tempfile.NamedTemporaryFile(
            mode="w",
            encoding="utf-8",
            newline="",
            dir=path.parent,
            prefix=f".{path.name}.",
            delete=False,
        ) as temporary:
            temporary_name = temporary.name
            temporary.write(value)
            temporary.flush()
            os.fsync(temporary.fileno())
        os.replace(temporary_name, path)
    finally:
        if temporary_name is not None and os.path.exists(temporary_name):
            os.unlink(temporary_name)


def write_indexes(root: Path) -> list[Path]:
    """Regenerate every opted-in index and return the paths that changed."""
    changed: list[Path] = []
    for index in marked_indexes(root):
        source = index.read_text(encoding="utf-8")
        expected = render_index(source, generated_index_body(index), index)
        if expected != source:
            _atomic_write(index, expected)
            changed.append(index)
    return changed


def _raw_link_target(raw: str) -> str:
    target = raw.strip()
    if target.startswith("<") and ">" in target:
        return target[1 : target.index(">")]
    return target.split(maxsplit=1)[0]


def _fragment(raw: str) -> str | None:
    target = _raw_link_target(raw)
    if "#" not in target:
        return None
    value = unquote(target.split("#", 1)[1].split("?", 1)[0])
    return value or None


def _local_target(raw: str) -> str | None:
    target = _raw_link_target(raw)
    target = unquote(target.split("#", 1)[0].split("?", 1)[0])
    if not target or URI_SCHEME.match(target):
        return None
    return target


def _record_paths(root: Path) -> list[Path]:
    docs = root / "docs"
    if not docs.is_dir():
        return []
    return sorted(
        (path for path in docs.rglob("*.md") if path.is_file()),
        key=lambda value: value.relative_to(root).as_posix().encode(),
    )


def relationship_graph(root: Path) -> dict[str, object]:
    """Derive a replaceable graph from authored record links and ownership."""
    root = root.resolve()
    paths = _record_paths(root)
    metadata_by_path: dict[Path, dict[str, str]] = {}
    path_by_coordinate: dict[str, Path] = {}
    for path in paths:
        source = path.read_text(encoding="utf-8")
        coordinate = header_field(source, "Coordinate")
        if coordinate is None:
            continue
        metadata = record_metadata(path)
        resolved = path.resolve()
        prior = path_by_coordinate.get(metadata["coordinate"])
        if prior is not None:
            raise NavigationError(
                f"duplicate Coordinate {metadata['coordinate']}: {prior} and {path}"
            )
        metadata_by_path[resolved] = metadata
        path_by_coordinate[metadata["coordinate"]] = resolved

    nodes: list[dict[str, str]] = []
    edges: set[tuple[str, str, str, str]] = set()
    for path in sorted(
        metadata_by_path,
        key=lambda value: value.relative_to(root).as_posix().encode(),
    ):
        metadata = metadata_by_path[path]
        relative = path.relative_to(root).as_posix()
        nodes.append(
            {
                "coordinate": metadata["coordinate"],
                "source_path": relative,
                "title": metadata["title"],
                "status": metadata["status"],
                "owner": metadata["owner"],
            }
        )
        source = path.read_text(encoding="utf-8")
        for raw in (*INLINE_LINK.findall(source), *REFERENCE_LINK.findall(source)):
            target_coordinate: str | None = None
            raw_target = _raw_link_target(raw)
            if raw_target.startswith("rrflow://"):
                target_coordinate = raw_target.split("#", 1)[0].split("?", 1)[0]
            elif (local := _local_target(raw)) is not None:
                target = (path.parent / local).resolve()
                if target.is_dir():
                    target = target / "README.md"
                target_metadata = metadata_by_path.get(target)
                if target_metadata is not None:
                    target_coordinate = target_metadata["coordinate"]
            if target_coordinate is not None:
                edges.add(
                    (
                        metadata["coordinate"],
                        target_coordinate,
                        "links-to",
                        _fragment(raw) or "",
                    )
                )

    return {
        "schema_version": 1,
        "nodes": sorted(nodes, key=lambda node: node["coordinate"].encode()),
        "edges": [
            {
                "source": source,
                "target": target,
                "kind": kind,
                "fragment": fragment or None,
            }
            for source, target, kind, fragment in sorted(edges)
        ],
    }


def _arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root",
        type=Path,
        default=Path(__file__).resolve().parents[2],
        help="repository root containing docs/",
    )
    output_mode = parser.add_mutually_exclusive_group()
    output_mode.add_argument(
        "--check", action="store_true", help="report stale indexes without writing"
    )
    output_mode.add_argument(
        "--graph-output",
        type=Path,
        help="optionally write the derived coordinated-record graph as canonical JSON",
    )
    return parser.parse_args()


def main() -> int:
    arguments = _arguments()
    root = arguments.root.resolve()
    if arguments.check:
        failures = index_drift_failures(root)
        if failures:
            for failure in failures:
                print(f"navigation: {failure}")
            return 1
        changed: list[Path] = []
    else:
        changed = write_indexes(root)

    graph = relationship_graph(root)
    if arguments.graph_output is not None:
        encoded = (
            json.dumps(graph, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
            + "\n"
        )
        _atomic_write(arguments.graph_output.resolve(), encoded)

    print(
        "navigation: OK: "
        f"{len(marked_indexes(root))} generated indexes, "
        f"{len(graph['nodes'])} nodes, {len(graph['edges'])} edges, "
        f"{len(changed)} index files updated"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
