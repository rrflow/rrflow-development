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
STATUS = re.compile(r"(?im)^(?:\*\*)?Status(?:\*\*)?:\s*\S")
LEGACY_MILESTONE = re.compile(r"\b(?:F\d|G\d{2}-W\d+|M\d|Q\d)\b")
INLINE_LINK = re.compile(r"!?\[[^\]\n]*\]\(([^)\n]+)\)")
REFERENCE_LINK = re.compile(r"(?m)^\[[^\]\n]+\]:\s*(\S+)")
URI_SCHEME = re.compile(r"^[a-z][a-z0-9+.-]*:", re.IGNORECASE)


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
    for directory in ("objectives/", "roadmap/", "poam/", "reference/"):
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

    roadmap = ROADMAP.read_text(encoding="utf-8")
    if "## RRFlow 1.0 execution checklist" not in roadmap:
        failures.append("the owning RRFlow 1.0 roadmap has no execution checklist")
    if "rrflow://rrflow-instance/data/roadmap/rrflow-1.0" not in roadmap:
        failures.append("the owning RRFlow 1.0 roadmap has no durable coordinate")
    if "## Required outcomes" in roadmap or "## Open deficiencies" in roadmap:
        failures.append("the roadmap duplicates objective or POA&M ownership")

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
        f"knowledge ownership, {len(documents)} document statuses, and local links"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
