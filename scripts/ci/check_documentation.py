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
EXECUTION_MAP = ROOT / "docs" / "roadmap" / "rrflow-1.0-execution-map.md"
EXECUTION_FILE_PLAN = ROOT / "docs" / "roadmap" / "rrflow-1.0-file-plan.jsonl"
OBJECTIVE = ROOT / "docs" / "objectives" / "rrflow-1.0-alpha.md"
POAM = ROOT / "docs" / "poam" / "rrflow-1.0-alpha.md"
AGENT_REFERENCE = ROOT / "docs" / "reference" / "agent-bootstrap.md"
SEAT_IDENTITY_REFERENCE = ROOT / "docs" / "reference" / "seat-identity.md"
SYSTEM_OVERVIEW = ROOT / "docs" / "architecture" / "system-overview.md"
ENGINE_DATA_FLOW = ROOT / "docs" / "architecture" / "engine-data-flow.md"
SINGLE_ENGINE_DECISION = ROOT / "docs" / "decisions" / "0001-single-engine-authority.md"
SYSTEM_CONVERGENCE_RESEARCH = (
    ROOT / "docs" / "research" / "rrflow-system-convergence-architecture-research.md"
)
RRFLOWKV_CURRENT_FORMAT = (
    ROOT / "docs" / "reference" / "storage" / "rrflowkv-current-format.md"
)
RRFLOWKV_BENCHMARK_HARNESS = (
    ROOT / "docs" / "reference" / "storage" / "rrflowkv-benchmark-harness.md"
)
HISTORICAL_QUERY_PLAN = ROOT / "docs" / "history" / "rrd-arrow-datafusion-bm25-plan.md"
HISTORICAL_LSM_BENCHMARK = ROOT / "docs" / "history" / "rrd-lsm-promotion-benchmark.md"
HISTORICAL_DATA_SERVICES_RESEARCH = (
    ROOT / "docs" / "history" / "rrd-data-services-architecture-research.md"
)
HISTORICAL_LSM_MIGRATION = ROOT / "docs" / "history" / "rrd-lsm-migration.md"
STATUS = re.compile(r"(?im)^(?:\*\*)?Status(?:\*\*)?:\s*\S")
LEGACY_MILESTONE = re.compile(r"\b(?:F\d|G\d{2}-W\d+|M\d|Q\d)\b")
INLINE_LINK = re.compile(r"!?\[[^\]\n]*\]\(([^)\n]+)\)")
REFERENCE_LINK = re.compile(r"(?m)^\[[^\]\n]+\]:\s*(\S+)")
MARKDOWN_HEADING = re.compile(r"(?m)^#{1,6}\s+(.+?)\s*#*\s*$")
URI_SCHEME = re.compile(r"^[a-z][a-z0-9+.-]*:", re.IGNORECASE)
RRFLOW_COORDINATE = re.compile(r"^rrflow://rrflow-instance/data/[a-z0-9][a-z0-9./-]*$")
CANONICAL_DIRECTORIES = (
    "architecture",
    "decisions",
    "objectives",
    "roadmap",
    "poam",
    "reference",
    "research",
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


def local_fragment(raw: str) -> str | None:
    """Return a local Markdown fragment without treating rrflow URIs as files."""
    target = raw.strip()
    if target.startswith("<") and ">" in target:
        target = target[1 : target.index(">")]
    else:
        target = target.split(maxsplit=1)[0]
    if URI_SCHEME.match(target) or "#" not in target:
        return None
    fragment = unquote(target.split("#", 1)[1].split("?", 1)[0])
    return fragment or None


def markdown_anchors(source: str) -> set[str]:
    """Build the GitHub-style heading anchors used by the root portal."""
    anchors: set[str] = set()
    counts: dict[str, int] = {}
    for heading in MARKDOWN_HEADING.findall(source):
        label = re.sub(r"\[([^\]]+)\]\([^)]+\)", r"\1", heading)
        label = label.replace("`", "").replace("*", "").replace("~", "")
        base = "".join(
            character
            for character in label.casefold()
            if character.isalnum() or character in {" ", "-", "_"}
        ).replace(" ", "-")
        ordinal = counts.get(base, 0)
        counts[base] = ordinal + 1
        anchors.add(base if ordinal == 0 else f"{base}-{ordinal}")
    return anchors


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
        "docs/architecture/system-overview.md",
        "docs/architecture/engine-data-flow.md",
        "docs/decisions/0001-single-engine-authority.md",
        "docs/reference/storage/rrflowkv-current-format.md",
        "docs/objectives/rrflow-1.0-alpha.md",
        "docs/roadmap/rrflow-1.0.md",
        "docs/poam/rrflow-1.0-alpha.md",
        "docs/reference/agent-bootstrap.md",
        "docs/reference/seat-identity.md",
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
        "decisions/",
        "objectives/",
        "roadmap/",
        "poam/",
        "reference/",
        "research/",
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

    system_overview = SYSTEM_OVERVIEW.read_text(encoding="utf-8")
    for required_section in (
        "## Platform map",
        "## Canonical component terminology",
        "## Security boundary",
        "## Client bootstrap boundary",
        "## Documentation and future memory",
    ):
        if required_section not in system_overview:
            failures.append(f"the system-overview owner lacks {required_section}")
    for required_term in (
        "**RRFlow**",
        "**RRD**",
        "**`RrdEngine`**",
        "**rrflowDB**",
        "**rrflowKV**",
        "**rrflowMX**",
        "**rrflowQL**",
        "**Arrow substrate**",
        "**DataFusion execution**",
        "**RRFlow vector subsystem**",
        "**RRFlow inference**",
        "**Connectome**",
    ):
        if required_term not in system_overview:
            failures.append(
                f"the system-overview owner lacks canonical term {required_term}"
            )
    if "rrflow://rrflow-instance/data/architecture/system-overview" not in (
        system_overview
    ):
        failures.append("the system-overview owner has no durable coordinate")

    seat_identity_reference = SEAT_IDENTITY_REFERENCE.read_text(encoding="utf-8")
    for required_section in (
        "## Canonical records and relations",
        "## Stable record warps",
        "## CLI operations",
        "## Executable proof and remaining boundary",
    ):
        if required_section not in seat_identity_reference:
            failures.append(f"the seat-identity reference lacks {required_section}")
    if "rrflow://rrflow-instance/data/reference/seat-identity" not in (
        seat_identity_reference
    ):
        failures.append("the seat-identity reference has no durable coordinate")

    single_engine_decision = SINGLE_ENGINE_DECISION.read_text(encoding="utf-8")
    for required_section in (
        "## Context",
        "## Decision",
        "## Consequences",
        "## Rejected alternatives",
    ):
        if required_section not in single_engine_decision:
            failures.append(f"the single-engine ADR lacks {required_section}")
    if "../architecture/system-overview.md" not in single_engine_decision:
        failures.append("the single-engine ADR has no system-overview link")

    engine_data_flow = ENGINE_DATA_FLOW.read_text(encoding="utf-8")
    if "## Write and commit flow" not in engine_data_flow:
        failures.append("the engine data-flow owner has no write path")
    if "## Read and query flow" not in engine_data_flow:
        failures.append("the engine data-flow owner has no read path")
    if "## Context assembly contract" not in engine_data_flow:
        failures.append("the engine data-flow owner has no context contract")
    if "## Conditional zero-copy" not in engine_data_flow:
        failures.append("the engine data-flow owner has no physical copy boundary")
    if "rrflow://rrflow-instance/data/architecture/engine-data-flow" not in (
        engine_data_flow
    ):
        failures.append("the engine data-flow owner has no durable coordinate")
    if (ROOT / "docs" / "rrd-arrow-datafusion-bm25-plan.md").exists():
        failures.append("the superseded Q1-Q4 query plan remains active and flat")
    if (ROOT / "docs" / "rrd-lsm-format.md").exists():
        failures.append(
            "the rrflowKV physical-format reference remains active and flat"
        )
    if (ROOT / "docs" / "rrd-lsm-benchmark.md").exists():
        failures.append(
            "the mixed-purpose rrflowKV benchmark note remains active and flat"
        )
    if (ROOT / "docs" / "rrd-data-services-architecture-research.md").exists():
        failures.append(
            "the superseded M0-M8 architecture chronology remains active and flat"
        )
    if (ROOT / "docs" / "rrd-lsm-migration.md").exists():
        failures.append("the compatibility migration contract remains active and flat")
    if (ROOT / "docs" / "platform").exists():
        failures.append("the obsolete platform documentation wrapper still exists")
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
    if "#### A-06 knowledge-bootstrap sequence" not in roadmap:
        failures.append("the roadmap has no incremental knowledge-bootstrap sequence")
    if "rrflow-1.0-execution-map.md" not in roadmap:
        failures.append("the roadmap does not link its supporting code execution map")

    execution_map = EXECUTION_MAP.read_text(encoding="utf-8")
    for required_section in (
        "## How to execute this map",
        "## Product terms versus implementation packages",
        "## Frozen target source tree",
        "## Target runtime flows",
        "## Current implementation inventory and exact disposition",
        "## Repository-wide run checklist",
        "## Global stop conditions",
    ):
        if required_section not in execution_map:
            failures.append(f"the execution map lacks {required_section}")
    if "cannot mark a release gate complete" not in execution_map:
        failures.append("the execution map does not disclaim roadmap authority")
    if "rrflow-1.0-file-plan.jsonl" not in execution_map:
        failures.append("the execution map does not link its exhaustive file plan")
    if not EXECUTION_FILE_PLAN.is_file():
        failures.append("the exhaustive RRFlow 1.0 file plan is absent")

    convergence_research = SYSTEM_CONVERGENCE_RESEARCH.read_text(encoding="utf-8")
    for required_section in (
        "## Direct answer",
        "## Evidence reconciliation",
        "## Current-code gap matrix",
        "## Decisions and exclusions",
        "## Claim-to-source ledger",
        "## Research limitations and stop condition",
    ):
        if required_section not in convergence_research:
            failures.append(f"the system-convergence research lacks {required_section}")

    terminology_owners = {
        README: readme,
        SYSTEM_OVERVIEW: system_overview,
        ENGINE_DATA_FLOW: engine_data_flow,
        OBJECTIVE: objective,
        ROADMAP: roadmap,
        POAM: poam,
    }
    for owner_path, source in terminology_owners.items():
        for retired_term in ("RRFlow database", "RRFlowQL"):
            if retired_term in source:
                failures.append(
                    f"{owner_path.relative_to(ROOT)} uses retired product term "
                    f"{retired_term!r}"
                )

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

    benchmark_harness = RRFLOWKV_BENCHMARK_HARNESS.read_text(encoding="utf-8")
    for required_section in (
        "## General comparison protocol",
        "## Output schema and provenance gap",
        "## Evidence required for release use",
    ):
        if required_section not in benchmark_harness:
            failures.append(f"the benchmark-harness reference lacks {required_section}")
    if "not RRFlow 1.0 release evidence" not in benchmark_harness:
        failures.append("the local comparator is not separated from release evidence")

    historical_lsm_benchmark = HISTORICAL_LSM_BENCHMARK.read_text(encoding="utf-8")
    historical_lsm_header = "\n".join(historical_lsm_benchmark.splitlines()[:12])
    if "historical" not in historical_lsm_header.casefold():
        failures.append("the August LSM comparison note is not marked historical")
    if "../reference/storage/rrflowkv-benchmark-harness.md" not in (
        historical_lsm_header
    ):
        failures.append("the August LSM comparison note has no current successor")

    historical_research = HISTORICAL_DATA_SERVICES_RESEARCH.read_text(encoding="utf-8")
    historical_research_header = "\n".join(historical_research.splitlines()[:12])
    if "historical" not in historical_research_header.casefold():
        failures.append("the M0-M8 data-services chronology is not marked historical")
    if "../architecture/engine-data-flow.md" not in historical_research_header:
        failures.append(
            "the M0-M8 data-services chronology has no architecture successor"
        )

    historical_migration = HISTORICAL_LSM_MIGRATION.read_text(encoding="utf-8")
    historical_migration_header = "\n".join(historical_migration.splitlines()[:12])
    if "historical" not in historical_migration_header.casefold():
        failures.append("the compatibility migration contract is not marked historical")
    if "../reference/storage/rrflowkv-current-format.md" not in (
        historical_migration_header
    ):
        failures.append(
            "the compatibility migration contract has no current-format successor"
        )
    if "`RRDMIG01`" not in historical_migration or "RRFLOWIG01" in historical_migration:
        failures.append("the historical migration archive identity is inaccurate")

    coordinates: dict[str, Path] = {}
    for record in canonical_records():
        source = record.read_text(encoding="utf-8")
        relative = record.relative_to(ROOT)
        status_value = header_field(source, "Status")
        coordinate = header_field(source, "Coordinate")
        owner = header_field(source, "Owner")
        superseded_by = header_field(source, "Superseded by")
        active = status_value is not None and status_value.casefold().startswith(
            "active"
        )
        historical = status_value is not None and status_value.casefold().startswith(
            "historical"
        )
        if (active or historical) and coordinate is None:
            failures.append(f"{relative}: classified record has no stable Coordinate")
        if active and owner is None:
            failures.append(f"{relative}: active record has no Owner")
        if historical and superseded_by is None:
            failures.append(f"{relative}: historical record has no Superseded by link")
        if (active or historical) and not index_links_record(record):
            failures.append(
                f"{relative}: classified record is absent from its parent index"
            )
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
            elif (
                document == README
                and (fragment := local_fragment(raw)) is not None
                and (
                    not resolved.is_file()
                    or fragment
                    not in markdown_anchors(resolved.read_text(encoding="utf-8"))
                )
            ):
                failures.append(f"{relative}: broken owner-section fragment: {raw}")

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
