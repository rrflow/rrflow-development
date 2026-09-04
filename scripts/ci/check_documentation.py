#!/usr/bin/env python3
"""Enforce documentation ownership, status metadata, and local links."""

from __future__ import annotations

import re
import sys
from pathlib import Path
from urllib.parse import unquote

ROOT = Path(__file__).resolve().parents[2]
README = ROOT / "README.md"
DOCS_INDEX = ROOT / "docs" / "README.md"
ROADMAP = ROOT / "docs" / "roadmap" / "rrflow-1.0.md"
OBJECTIVE = ROOT / "docs" / "objectives" / "rrflow-1.0-alpha.md"
POAM = ROOT / "docs" / "poam" / "rrflow-1.0-alpha.md"
AGENT_REFERENCE = ROOT / "docs" / "reference" / "agent-bootstrap.md"
ENGINE_DATA_FLOW = ROOT / "docs" / "architecture" / "engine-data-flow.md"
RRFLOWKV_CURRENT_FORMAT = (
    ROOT / "docs" / "reference" / "storage" / "rrflowkv-current-format.md"
)
HISTORICAL_QUERY_PLAN = (
    ROOT / "docs" / "history" / "rrd-arrow-datafusion-bm25-plan.md"
)
STATUS = re.compile(r"(?im)^(?:\*\*)?Status(?:\*\*)?:\s*\S")
LEGACY_MILESTONE = re.compile(r"\b(?:F\d|G\d{2}-W\d+|M\d|Q\d)\b")
INLINE_LINK = re.compile(r"!?\[[^\]\n]*\]\(([^)\n]+)\)")
REFERENCE_LINK = re.compile(r"(?m)^\[[^\]\n]+\]:\s*(\S+)")
URI_SCHEME = re.compile(r"^[a-z][a-z0-9+.-]*:", re.IGNORECASE)
RRFLOW_COORDINATE = re.compile(
    r"^rrflow://rrflow-instance/data/[a-z0-9][a-z0-9./-]*$"
)
CANONICAL_DIRECTORIES = (
    "architecture",
    "objectives",
    "roadmap",
    "poam",
    "reference",
    "history",
)


def header_field(source: str, name: str) -> str | None:
    """Read one Markdown metadata field without coupling policy to bold style."""
    for line in source.splitlines()[:12]:
        normalized = line.replace("**", "").strip()
        prefix = f"{name}:"
        if normalized.casefold().startswith(prefix.casefold()):
            value = normalized[len(prefix) :].strip()
            return value or None
    return None


def canonical_records() -> list[Path]:
    records: list[Path] = []
    for directory in CANONICAL_DIRECTORIES:
        records.extend(sorted((ROOT / "docs" / directory).rglob("*.md")))
    return records


def index_links_record(record: Path) -> bool:
    if record.name == "README.md":
        if record.parent == ROOT / "docs":
            return True
        parent_index = record.parent.parent / "README.md"
        accepted_targets = {record.resolve(), record.parent.resolve()}
    else:
        parent_index = record.parent / "README.md"
        accepted_targets = {record.resolve()}
    if not parent_index.is_file():
        return False
    source = parent_index.read_text(encoding="utf-8")
    links = [*INLINE_LINK.findall(source), *REFERENCE_LINK.findall(source)]
    for raw in links:
        target = local_target(raw)
        if target is None:
            continue
        if (parent_index.parent / target).resolve() in accepted_targets:
            return True
    return False


def supporting_documents() -> list[Path]:
    return [ROOT / "SPEC.md", *sorted((ROOT / "docs").rglob("*.md"))]


def local_target(raw: str) -> str | None:
    target = raw.strip()
    if target.startswith("<") and ">" in target:
        target = target[1 : target.index(">")]
    else:
        target = target.split(maxsplit=1)[0]
    target = unquote(target.split("#", 1)[0].split("?", 1)[0])
    if not target or target.startswith("#") or URI_SCHEME.match(target):
        return None
    return target


def main() -> int:
    failures: list[str] = []
    readme = README.read_text(encoding="utf-8")
    ownership_contract = (
        "This README is the bootstrap product entry point and knowledge map. It owns\n"
        "RRFlow's identity, non-negotiable architecture invariants, and current status;"
    )
    if ownership_contract not in readme:
        failures.append("README.md does not contain the knowledge-ownership contract")
    required_warps = (
        "docs/README.md",
        "docs/architecture/engine-data-flow.md",
        "docs/reference/storage/rrflowkv-current-format.md",
        "docs/objectives/rrflow-1.0-alpha.md",
        "docs/roadmap/rrflow-1.0.md",
        "docs/poam/rrflow-1.0-alpha.md",
        "docs/reference/agent-bootstrap.md",
    )
    if any(warp not in readme for warp in required_warps):
        failures.append("README.md is missing a required knowledge warp point")
    if "## RRFlow 1.0 execution checklist" in readme:
        failures.append("README.md duplicates the detailed RRFlow 1.0 roadmap")

    docs_index = DOCS_INDEX.read_text(encoding="utf-8")
    if "## Documentation taxonomy" not in docs_index:
        failures.append("docs/README.md does not define the documentation taxonomy")
    if "## Record header and indexing pattern" not in docs_index:
        failures.append("docs/README.md does not define the record/index pattern")
    for directory in (
        "architecture/",
        "objectives/",
        "roadmap/",
        "poam/",
        "reference/",
        "history/",
    ):
        if directory not in docs_index:
            failures.append(f"docs/README.md does not route {directory}")

    objective = OBJECTIVE.read_text(encoding="utf-8")
    if "## Required outcomes" not in objective:
        failures.append("the owning alpha objective has no measurable outcomes")
    if "rrflow://rrflow-instance/data/objective/rrflow-1.0-alpha" not in objective:
        failures.append("the owning alpha objective has no durable coordinate")

    poam = POAM.read_text(encoding="utf-8")
    if "## Open deficiencies" not in poam:
        failures.append("the owning alpha POA&M has no deficiency ledger")
    if "rrflow://rrflow-instance/data/poam/rrflow-1.0-alpha" not in poam:
        failures.append("the owning alpha POA&M has no durable coordinate")

    agent_reference = AGENT_REFERENCE.read_text(encoding="utf-8")
    if "## Installed specialization" not in agent_reference:
        failures.append("the agent bootstrap reference has no specialization contract")
    if (ROOT / "CLAUDE.md").read_text(encoding="utf-8").strip() != (
        "# Claude Code\n\n@AGENTS.md"
    ):
        failures.append("CLAUDE.md must remain a forwarding-only AGENTS.md adapter")
    if (ROOT / "GEMINI.md").read_text(encoding="utf-8").strip() != (
        "# Google Gemini CLI\n\n@./AGENTS.md"
    ):
        failures.append("GEMINI.md must remain a forwarding-only AGENTS.md adapter")

    engine_data_flow = ENGINE_DATA_FLOW.read_text(encoding="utf-8")
    if "## Write and commit flow" not in engine_data_flow:
        failures.append("the engine data-flow owner has no write path")
    if "## Read and query flow" not in engine_data_flow:
        failures.append("the engine data-flow owner has no read path")
    if "## Conditional zero-copy" not in engine_data_flow:
        failures.append("the engine data-flow owner has no physical copy boundary")
    if "rrflow://rrflow-instance/data/architecture/engine-data-flow" not in (
        engine_data_flow
    ):
        failures.append("the engine data-flow owner has no durable coordinate")
    if (ROOT / "docs" / "rrd-arrow-datafusion-bm25-plan.md").exists():
        failures.append("the superseded Q1-Q4 query plan remains active and flat")
    if (ROOT / "docs" / "rrd-lsm-format.md").exists():
        failures.append("the rrflowKV physical-format reference remains active and flat")
    historical_query_plan = HISTORICAL_QUERY_PLAN.read_text(encoding="utf-8")
    if "historical" not in "\n".join(historical_query_plan.splitlines()[:12]).lower():
        failures.append("the superseded Q1-Q4 query plan is not marked historical")
    if "../architecture/engine-data-flow.md" not in historical_query_plan:
        failures.append("the superseded Q1-Q4 query plan has no architecture successor")

    roadmap = ROADMAP.read_text(encoding="utf-8")
    if "## RRFlow 1.0 execution checklist" not in roadmap:
        failures.append("the owning RRFlow 1.0 roadmap has no execution checklist")
    if "rrflow://rrflow-instance/data/roadmap/rrflow-1.0" not in roadmap:
        failures.append("the owning RRFlow 1.0 roadmap has no durable coordinate")
    if "## Required outcomes" in roadmap or "## Open deficiencies" in roadmap:
        failures.append("the roadmap duplicates objective or POA&M ownership")

    current_format = RRFLOWKV_CURRENT_FORMAT.read_text(encoding="utf-8")
    for required_section in (
        "## Current and target boundary",
        "## Concrete fixture and failure examples",
        "## Executable proof",
    ):
        if required_section not in current_format:
            failures.append(
                f"the rrflowKV current-format reference lacks {required_section}"
            )
    if "not the accepted RRFlow 1.0 target" not in current_format:
        failures.append("the current row format is not separated from the 1.0 target")

    coordinates: dict[str, Path] = {}
    for record in canonical_records():
        source = record.read_text(encoding="utf-8")
        relative = record.relative_to(ROOT)
        status_value = header_field(source, "Status")
        coordinate = header_field(source, "Coordinate")
        owner = header_field(source, "Owner")
        active = status_value is not None and status_value.casefold().startswith(
            "active"
        )
        if active and coordinate is None:
            failures.append(f"{relative}: active record has no stable Coordinate")
        if active and owner is None:
            failures.append(f"{relative}: active record has no Owner")
        if active and not index_links_record(record):
            failures.append(f"{relative}: active record is absent from its parent index")
        if coordinate is not None:
            normalized_coordinate = coordinate.strip("`")
            if RRFLOW_COORDINATE.fullmatch(normalized_coordinate) is None:
                failures.append(
                    f"{relative}: Coordinate is not a canonical path-safe rrflow URI"
                )
            prior = coordinates.get(normalized_coordinate)
            if prior is not None:
                failures.append(
                    f"{relative}: duplicate Coordinate also owned by "
                    f"{prior.relative_to(ROOT)}"
                )
            else:
                coordinates[normalized_coordinate] = record

    documents = supporting_documents()
    for document in documents:
        source = document.read_text(encoding="utf-8")
        relative = document.relative_to(ROOT)
        header = "\n".join(source.splitlines()[:12])
        status = STATUS.search(header)
        if status is None:
            failures.append(f"{relative}: no Status declaration in the first 12 lines")
        else:
            status_block = header[status.start() :].split("\n\n", 1)[0]
            if (
                LEGACY_MILESTONE.search(status_block)
                and "historical" not in status_block.lower()
            ):
                failures.append(
                    f"{relative}: active status uses a retired milestone label"
                )

        lowered = source.lower()
        for stale in (
            "`spec.md` remains authoritative",
            "canonical platform terms and hierarchy are defined only in",
            "crates/connectome-ui",
            "-p rrd-graph",
        ):
            if stale in lowered:
                failures.append(f"{relative}: stale authority/layout claim {stale!r}")
        if re.search(r"(?im)^# .*\bcanon\b", source):
            failures.append(
                f"{relative}: supporting document title claims canon status"
            )

    linked_documents = [README, *documents]
    for document in linked_documents:
        source = document.read_text(encoding="utf-8")
        relative = document.relative_to(ROOT)
        links = [*INLINE_LINK.findall(source), *REFERENCE_LINK.findall(source)]
        for raw in links:
            target = local_target(raw)
            if target is None:
                continue
            resolved = (document.parent / target).resolve()
            if not resolved.is_relative_to(ROOT):
                failures.append(f"{relative}: local link escapes the repository: {raw}")
            elif not resolved.exists():
                failures.append(f"{relative}: broken local link: {raw}")

    if failures:
        for failure in failures:
            print(f"documentation-policy: {failure}", file=sys.stderr)
        return 1
    print(
        "documentation-policy: OK: "
        f"knowledge ownership, {len(documents)} document statuses, "
        f"{len(coordinates)} classified coordinates, parent indexes, and local links"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
