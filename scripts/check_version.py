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


def is_cargo_fuzz_manifest(document: dict[str, object]) -> bool:
    package = document.get("package")
    if not isinstance(package, dict):
        return False
    metadata = package.get("metadata")
    return isinstance(metadata, dict) and metadata.get("cargo-fuzz") is True


def validate_cargo_fuzz_manifest(
    manifest: Path,
    document: dict[str, object],
    declared_manifests: list[Path],
    failures: list[str],
) -> None:
    relative = manifest.relative_to(ROOT)
    package = document.get("package")
    if not isinstance(package, dict):
        fail(f"{relative} cargo-fuzz manifest has no package table", failures)
        return

    if manifest in declared_manifests:
        fail(f"{relative} cargo-fuzz tooling must not be a product member", failures)
    if package.get("version") != "0.0.0":
        fail(f"{relative} cargo-fuzz tooling must use version 0.0.0", failures)
    if package.get("publish") is not False:
        fail(f"{relative} cargo-fuzz tooling must set publish = false", failures)

    nested_workspace = document.get("workspace")
    if not isinstance(nested_workspace, dict) or nested_workspace.get("members") != [
        "."
    ]:
        fail(
            f"{relative} cargo-fuzz tooling must define workspace members = ['.']",
            failures,
        )

    owner_manifest = manifest.parent.parent / "Cargo.toml"
    if manifest.parent.name != "fuzz" or owner_manifest not in declared_manifests:
        fail(
            f"{relative} cargo-fuzz tooling must live in fuzz/ below a product crate",
            failures,
        )


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
    manifest_documents = {manifest: load_toml(manifest) for manifest in manifests}
    declared_manifests = sorted(
        ROOT / member / "Cargo.toml"
        for member in workspace["workspace"]["members"]  # type: ignore[index]
    )
    cargo_fuzz_manifests = [
        manifest
        for manifest, document in manifest_documents.items()
        if is_cargo_fuzz_manifest(document)
    ]
    product_manifests = sorted(set(manifests) - set(cargo_fuzz_manifests))
    if product_manifests != declared_manifests:
        fail(
            "Cargo package manifests and workspace membership differ: "
            f"discovered={[str(path.relative_to(ROOT)) for path in product_manifests]} "
            f"declared={[str(path.relative_to(ROOT)) for path in declared_manifests]}",
            failures,
        )
    workspace_package_names: set[str] = set()
    for manifest in product_manifests:
        package = manifest_documents[manifest]["package"]  # type: ignore[index]
        workspace_package_names.add(package["name"])  # type: ignore[index]
        if package.get("version") != {"workspace": True}:  # type: ignore[union-attr]
            fail(
                f"{manifest.relative_to(ROOT)} must use `version.workspace = true`",
                failures,
            )
    for manifest in cargo_fuzz_manifests:
        validate_cargo_fuzz_manifest(
            manifest,
            manifest_documents[manifest],
            declared_manifests,
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
    version_policy_path = ROOT / "docs/reference/release/version-policy.md"
    version_policy = version_policy_path.read_text(encoding="utf-8")
    if version_policy_line not in version_policy:
        fail(
            f"{version_policy_path.relative_to(ROOT)} does not declare the canonical VERSION",
            failures,
        )

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
