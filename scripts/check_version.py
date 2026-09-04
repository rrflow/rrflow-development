#!/usr/bin/env python3
"""Fail when RRFlow release-version declarations drift from VERSION."""

from __future__ import annotations

import json
import re
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

import tomllib

ROOT = Path(__file__).resolve().parents[1]
VERSION = (ROOT / "VERSION").read_text(encoding="utf-8").strip()
SEMVER = re.compile(
    r"(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)"
    r"(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$"
)


def load_toml(path: Path) -> dict[str, object]:
    with path.open("rb") as source:
        return tomllib.load(source)


def fail(message: str, failures: list[str]) -> None:
    failures.append(message)


def main() -> int:
    failures: list[str] = []
    if not SEMVER.fullmatch(VERSION):
        fail(f"VERSION is not valid SemVer: {VERSION!r}", failures)

    workspace = load_toml(ROOT / "Cargo.toml")
    workspace_version = workspace["workspace"]["package"]["version"]  # type: ignore[index]
    if workspace_version != VERSION:
        fail(
            f"Cargo workspace version is {workspace_version!r}, expected {VERSION!r}",
            failures,
        )

    manifests = sorted((ROOT / "crates").rglob("Cargo.toml"))
    declared_manifests = sorted(
        ROOT / member / "Cargo.toml"
        for member in workspace["workspace"]["members"]  # type: ignore[index]
    )
    if manifests != declared_manifests:
        fail(
            "Cargo package manifests and workspace membership differ: "
            f"discovered={[str(path.relative_to(ROOT)) for path in manifests]} "
            f"declared={[str(path.relative_to(ROOT)) for path in declared_manifests]}",
            failures,
        )
    workspace_package_names: set[str] = set()
    for manifest in manifests:
        package = load_toml(manifest)["package"]  # type: ignore[index]
        workspace_package_names.add(package["name"])  # type: ignore[index]
        if package.get("version") != {"workspace": True}:  # type: ignore[union-attr]
            fail(
                f"{manifest.relative_to(ROOT)} must use `version.workspace = true`",
                failures,
            )

    cargo_lock = load_toml(ROOT / "Cargo.lock")
    stale_locked_packages = sorted(
        package["name"]
        for package in cargo_lock["package"]  # type: ignore[index]
        if package["name"] in workspace_package_names
        and package["version"] != VERSION
        and "source" not in package
    )
    if stale_locked_packages:
        fail(
            "Cargo.lock has stale workspace versions for "
            + ", ".join(stale_locked_packages),
            failures,
        )

    readme_version_line = f"The target release-train version is `{VERSION}`."
    readme = (ROOT / "README.md").read_text(encoding="utf-8")
    if readme_version_line not in readme:
        fail("README.md does not mirror the target VERSION", failures)

    version_policy_line = (
        f"RRFlow's canonical current release-train version is `{VERSION}`."
    )
    version_policy = (ROOT / "docs/versioning.md").read_text(encoding="utf-8")
    if version_policy_line not in version_policy:
        fail("docs/versioning.md does not declare the canonical VERSION", failures)

    if (ROOT / "apps/connectome").exists():
        fail(
            "apps/connectome is not a supported layout; Connectome is a separate repository",
            failures,
        )

    typescript = json.loads(
        (ROOT / "sdks/typescript/package.json").read_text(encoding="utf-8")
    )
    if typescript["version"] != VERSION:
        fail(
            f"TypeScript SDK version is {typescript['version']!r}, expected {VERSION!r}",
            failures,
        )

    python_project = load_toml(ROOT / "sdks/python/pyproject.toml")
    python_version = python_project["project"]["version"]  # type: ignore[index]
    if python_version != VERSION:
        fail(
            f"Python SDK version is {python_version!r}, expected {VERSION!r}", failures
        )

    python_lock = load_toml(ROOT / "sdks/python/uv.lock")
    locked_versions = [
        package["version"]
        for package in python_lock["package"]  # type: ignore[index]
        if package["name"] == "rrflow-rrd-client"
    ]
    if locked_versions != [VERSION]:
        fail(
            f"Python SDK lock versions are {locked_versions!r}, expected [{VERSION!r}]",
            failures,
        )

    java_root = ET.parse(ROOT / "sdks/java/pom.xml").getroot()
    namespace = {"m": "http://maven.apache.org/POM/4.0.0"}
    java_version = java_root.findtext("m:version", namespaces=namespace)
    if java_version != VERSION:
        fail(f"Java SDK version is {java_version!r}, expected {VERSION!r}", failures)

    dotnet_root = ET.parse(
        ROOT / "sdks/dotnet/src/Rrflow.Rrd.Client/Rrflow.Rrd.Client.csproj"
    ).getroot()
    dotnet_version = dotnet_root.findtext("./PropertyGroup/Version")
    if dotnet_version != VERSION:
        fail(f".NET SDK version is {dotnet_version!r}, expected {VERSION!r}", failures)

    contract = json.loads(
        (
            ROOT / "crates/transport/rrd-contract/fixtures/public-contract-v1.json"
        ).read_text(encoding="utf-8")
    )
    implementation_version = contract["service"]["implementation_version"]
    if implementation_version != VERSION:
        fail(
            f"public contract implementation version is {implementation_version!r}, "
            f"expected {VERSION!r}",
            failures,
        )

    if failures:
        for message in failures:
            print(f"version-policy: ERROR: {message}", file=sys.stderr)
        return 1

    print(f"version-policy: OK: RRFlow release train is {VERSION}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
