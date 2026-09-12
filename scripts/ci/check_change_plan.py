#!/usr/bin/env python3
"""Fail closed when repository work diverges from its committed change plan."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import shlex
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Any

PLAN_PATH = "docs/roadmap/rrflow-1.0-active-change.json"
HEX_OBJECT_ID = re.compile(r"[0-9a-f]{40}(?:[0-9a-f]{24})?")
SHA256 = re.compile(r"[0-9a-f]{64}")
DECISIONS = {"add", "preserve", "not_applicable"}
GIT_TIMEOUT_SECONDS = 60
MAX_GIT_OUTPUT_BYTES = 64 * 1024 * 1024
MAX_PLAN_BYTES = 1024 * 1024
MAX_REVIEWED_FILE_BYTES = 64 * 1024 * 1024
MAX_POST_PLAN_COMMITS = 256
MAX_CHANGED_PATHS = 4096


class PlanError(RuntimeError):
    """A deterministic change-plan policy violation."""


def require(condition: bool, message: str) -> None:
    if not condition:
        raise PlanError(message)


def require_object(
    value: Any,
    label: str,
    required: set[str],
    optional: set[str] | None = None,
) -> dict[str, Any]:
    require(isinstance(value, dict), f"{label} must be an object")
    optional = optional or set()
    missing = sorted(required - set(value))
    unknown = sorted(set(value) - required - optional)
    require(not missing, f"{label} is missing fields: {missing}")
    require(not unknown, f"{label} has unknown fields: {unknown}")
    return value


def require_text(value: Any, label: str) -> str:
    require(isinstance(value, str) and bool(value.strip()), f"{label} must be nonempty")
    return value


def require_text_list(value: Any, label: str) -> list[str]:
    require(isinstance(value, list) and bool(value), f"{label} must be a nonempty list")
    result: list[str] = []
    for index, item in enumerate(value):
        result.append(require_text(item, f"{label}[{index}]"))
    require(len(result) == len(set(result)), f"{label} contains duplicate values")
    return result


def require_path(value: Any, label: str) -> str:
    path = require_text(value, label)
    pure = PurePosixPath(path)
    require(
        pure.as_posix() == path
        and not pure.is_absolute()
        and path not in {".", ""}
        and ".." not in pure.parts
        and "\\" not in path
        and "\0" not in path
        and "\n" not in path
        and "\r" not in path,
        f"{label} is not a normalized repository-relative path: {path!r}",
    )
    return path


def require_path_list(value: Any, label: str) -> list[str]:
    require(isinstance(value, list) and bool(value), f"{label} must be a nonempty list")
    result = [
        require_path(item, f"{label}[{index}]") for index, item in enumerate(value)
    ]
    require(len(result) == len(set(result)), f"{label} contains duplicate paths")
    return result


class GitRepository:
    """Small byte-safe Git query boundary used by the policy and its tests."""

    def __init__(self, root: Path) -> None:
        self.root = root.resolve()
        discovered = self.run_text("rev-parse", "--show-toplevel").strip()
        require(
            Path(discovered).resolve() == self.root,
            f"--root is not the Git worktree root: {self.root}",
        )

    def invoke(self, *arguments: str) -> subprocess.CompletedProcess[bytes]:
        try:
            return subprocess.run(
                ["git", *arguments],
                cwd=self.root,
                check=False,
                capture_output=True,
                timeout=GIT_TIMEOUT_SECONDS,
            )
        except subprocess.TimeoutExpired as error:
            raise PlanError(
                f"git {' '.join(arguments)} exceeded {GIT_TIMEOUT_SECONDS} seconds"
            ) from error

    def run(self, *arguments: str, allow_failure: bool = False) -> bytes:
        result = self.invoke(*arguments)
        if result.returncode != 0 and not allow_failure:
            diagnostic = result.stderr.decode("utf-8", errors="replace").strip()
            raise PlanError(
                f"git {' '.join(arguments)} failed with {result.returncode}: {diagnostic}"
            )
        output = result.stdout if result.returncode == 0 else b""
        require(
            len(output) <= MAX_GIT_OUTPUT_BYTES,
            f"git {' '.join(arguments)} exceeded the policy output budget",
        )
        return output

    def run_text(self, *arguments: str, allow_failure: bool = False) -> str:
        output = self.run(*arguments, allow_failure=allow_failure)
        try:
            return output.decode("utf-8")
        except UnicodeDecodeError as error:
            raise PlanError(
                f"git {' '.join(arguments)} returned a non-UTF-8 value"
            ) from error

    def revision(self, expression: str) -> str:
        value = self.run_text("rev-parse", "--verify", expression).strip()
        require(
            HEX_OBJECT_ID.fullmatch(value) is not None,
            f"Git expression did not resolve to an object ID: {expression}",
        )
        return value

    def object_exists(self, revision: str, path: str) -> bool:
        result = self.invoke("cat-file", "-e", f"{revision}:{path}")
        return result.returncode == 0

    def blob(self, revision: str, path: str) -> bytes:
        require(
            self.object_exists(revision, path),
            f"baseline path is absent at {revision}: {path}",
        )
        identity = f"{revision}:{path}"
        kind = self.run_text("cat-file", "-t", identity).strip()
        require(kind == "blob", f"reviewed baseline path is not a file blob: {path}")
        size_text = self.run_text("cat-file", "-s", identity).strip()
        try:
            size = int(size_text)
        except ValueError as error:
            raise PlanError(f"cannot resolve baseline blob size for {path}") from error
        require(
            0 <= size <= MAX_REVIEWED_FILE_BYTES,
            f"reviewed baseline file exceeds {MAX_REVIEWED_FILE_BYTES} bytes: {path}",
        )
        return self.run("show", identity)

    def parents(self, revision: str) -> list[str]:
        parts = self.run_text("rev-list", "--parents", "-n", "1", revision).split()
        require(
            parts and parts[0] == revision, f"cannot resolve commit parents: {revision}"
        )
        return parts[1:]

    def is_ancestor(self, ancestor: str, descendant: str) -> bool:
        result = self.invoke("merge-base", "--is-ancestor", ancestor, descendant)
        require(
            result.returncode in {0, 1},
            "cannot resolve planning ancestry: "
            + result.stderr.decode("utf-8", errors="replace").strip(),
        )
        return result.returncode == 0

    def paths(self, *arguments: str) -> set[str]:
        raw = self.run(*arguments)
        paths: set[str] = set()
        for item in raw.split(b"\0"):
            if not item:
                continue
            try:
                path = item.decode("utf-8")
            except UnicodeDecodeError as error:
                raise PlanError(
                    "Git changed-path output contains a non-UTF-8 path"
                ) from error
            paths.add(require_path(path, "Git changed path"))
        require(
            len(paths) <= MAX_CHANGED_PATHS,
            f"Git path set exceeds the {MAX_CHANGED_PATHS}-path package budget",
        )
        return paths

    def revisions(self, *arguments: str) -> list[str]:
        revisions = self.run_text("rev-list", *arguments).splitlines()
        require(
            len(revisions) <= MAX_POST_PLAN_COMMITS,
            f"post-plan history exceeds the {MAX_POST_PLAN_COMMITS}-commit package budget",
        )
        for revision in revisions:
            require(
                HEX_OBJECT_ID.fullmatch(revision) is not None,
                f"Git history contains an invalid object ID: {revision!r}",
            )
        return revisions

    def planning_commit_paths(self, parent: str, commit: str) -> set[str]:
        return self.paths(
            "diff", "--name-only", "--no-renames", "-z", parent, commit, "--"
        )

    def changed_paths(self, plan_commit: str, head: str) -> set[str]:
        changed = self.paths(
            "diff", "--name-only", "--no-renames", "-z", plan_commit, head, "--"
        )
        changed.update(
            self.paths("diff", "--cached", "--name-only", "--no-renames", "-z", "--")
        )
        changed.update(self.paths("diff", "--name-only", "--no-renames", "-z", "--"))
        changed.update(self.paths("ls-files", "--others", "--exclude-standard", "-z"))
        return changed

    def committed_history_paths(self, plan_commit: str, head: str) -> set[str]:
        paths: set[str] = set()
        revisions = self.revisions(f"{plan_commit}..{head}")
        for revision in revisions:
            parent_paths = [
                self.planning_commit_paths(parent, revision)
                for parent in self.parents(revision)
            ]
            if len(parent_paths) == 1:
                paths.update(parent_paths[0])
            elif parent_paths:
                paths.update(
                    set.intersection(
                        *(
                            set(changed_for_parent)
                            for changed_for_parent in parent_paths
                        )
                    )
                )
        return paths

    def planning_commit(
        self,
        baseline: str,
        head: str,
        plan_bytes: bytes,
    ) -> str:
        revisions = self.revisions("--ancestry-path", f"{baseline}..{head}")
        structural_candidates: list[str] = []
        content_candidates: list[str] = []
        for revision in revisions:
            if self.parents(revision) != [baseline]:
                continue
            if not self.object_exists(revision, PLAN_PATH):
                continue
            if PLAN_PATH not in self.planning_commit_paths(baseline, revision):
                continue
            structural_candidates.append(revision)
            if self.blob(revision, PLAN_PATH) == plan_bytes:
                content_candidates.append(revision)
        if len(content_candidates) == 1:
            return content_candidates[0]
        if not content_candidates and len(structural_candidates) == 1:
            return structural_candidates[0]
        raise PlanError(
            "active change plan must resolve to one direct child of its baseline: "
            f"structural={structural_candidates}, content={content_candidates}"
        )


@dataclass(frozen=True)
class ValidatedPlan:
    document: dict[str, Any]
    baseline_revision: str
    baseline_tree: str
    cargo_lock_sha256: str
    planning_paths: frozenset[str]
    reviewed_files: dict[str, dict[str, Any]]
    implementation_paths: frozenset[str]
    evidence_paths: frozenset[str]
    generated_evidence_paths: frozenset[str]


def path_records(value: Any, label: str, required: set[str]) -> list[dict[str, Any]]:
    require(isinstance(value, list) and bool(value), f"{label} must be a nonempty list")
    records: list[dict[str, Any]] = []
    paths: list[str] = []
    for index, item in enumerate(value):
        record = require_object(item, f"{label}[{index}]", required)
        path = require_path(record["path"], f"{label}[{index}].path")
        paths.append(path)
        records.append(record)
    require(len(paths) == len(set(paths)), f"{label} contains duplicate paths")
    return records


def validate_schema(document: Any) -> ValidatedPlan:
    plan = require_object(
        document,
        "active change plan",
        {
            "schema_version",
            "package_id",
            "status",
            "package_kind",
            "baseline",
            "authority",
            "current_behavior",
            "target_behavior",
            "unchanged_behavior",
            "planning_commit",
            "reviewed_files",
            "implementation_changes",
            "evidence_changes",
            "research",
            "trace_resource_debug",
            "first_failure_oracle",
            "edit_sequence",
            "acceptance_commands",
            "stop_conditions",
            "roadmap_status_change",
        },
    )
    require(
        isinstance(plan["schema_version"], int)
        and not isinstance(plan["schema_version"], bool)
        and plan["schema_version"] == 1,
        "active change plan schema_version must be integer 1",
    )
    require(plan["status"] == "active", "active change plan status must be active")
    require_text(plan["package_id"], "package_id")
    require_text(plan["package_kind"], "package_kind")
    require_text(plan["current_behavior"], "current_behavior")
    require_text(plan["target_behavior"], "target_behavior")
    require_text_list(plan["unchanged_behavior"], "unchanged_behavior")
    require_text_list(plan["first_failure_oracle"], "first_failure_oracle")
    require_text_list(plan["edit_sequence"], "edit_sequence")
    acceptance_commands = require_text_list(
        plan["acceptance_commands"], "acceptance_commands"
    )
    for index, command in enumerate(acceptance_commands):
        try:
            require(
                bool(shlex.split(command)),
                f"acceptance_commands[{index}] has no executable token",
            )
        except ValueError as error:
            raise PlanError(
                f"acceptance_commands[{index}] is not valid shell syntax"
            ) from error
    require(
        "python3 scripts/ci/check_change_plan.py" in acceptance_commands,
        "acceptance_commands must run the canonical change-plan validator",
    )
    require_text_list(plan["stop_conditions"], "stop_conditions")
    require_text(plan["roadmap_status_change"], "roadmap_status_change")

    baseline = require_object(
        plan["baseline"],
        "baseline",
        {
            "revision",
            "tree",
            "branch",
            "development_remote",
            "development_url",
            "worktree_state",
            "cargo_lock_sha256",
        },
    )
    revision = require_text(baseline["revision"], "baseline.revision")
    tree = require_text(baseline["tree"], "baseline.tree")
    require(
        HEX_OBJECT_ID.fullmatch(revision) is not None,
        "baseline.revision must be a complete Git object ID",
    )
    require(
        HEX_OBJECT_ID.fullmatch(tree) is not None,
        "baseline.tree must be a complete Git object ID",
    )
    require_text(baseline["branch"], "baseline.branch")
    require_text(baseline["development_remote"], "baseline.development_remote")
    development_url = require_text(
        baseline["development_url"], "baseline.development_url"
    )
    require(
        development_url.startswith("https://") and development_url.endswith(".git"),
        "baseline.development_url must be an explicit HTTPS Git URL",
    )
    require(
        baseline["worktree_state"] == "clean",
        "baseline.worktree_state must record clean",
    )
    cargo_lock_sha256 = require_text(
        baseline["cargo_lock_sha256"], "baseline.cargo_lock_sha256"
    )
    require(
        SHA256.fullmatch(cargo_lock_sha256) is not None,
        "baseline.cargo_lock_sha256 must be SHA-256",
    )

    authority = require_object(
        plan["authority"],
        "authority",
        {"alpha_prerequisite", "roadmap_gates", "poam_items", "architecture_owner"},
    )
    require_text(authority["alpha_prerequisite"], "authority.alpha_prerequisite")
    require_text_list(authority["roadmap_gates"], "authority.roadmap_gates")
    require_text_list(authority["poam_items"], "authority.poam_items")
    require_text(authority["architecture_owner"], "authority.architecture_owner")

    planning = require_object(
        plan["planning_commit"],
        "planning_commit",
        {
            "must_precede_implementation",
            "must_have_exactly_one_parent",
            "allowed_paths",
        },
    )
    require(
        planning["must_precede_implementation"] is True,
        "planning_commit.must_precede_implementation must be true",
    )
    require(
        planning["must_have_exactly_one_parent"] is True,
        "planning_commit.must_have_exactly_one_parent must be true",
    )
    planning_paths = frozenset(
        require_path_list(planning["allowed_paths"], "planning_commit.allowed_paths")
    )
    require(PLAN_PATH in planning_paths, f"planning paths must include {PLAN_PATH}")

    reviewed_records = path_records(
        plan["reviewed_files"],
        "reviewed_files",
        {"path", "baseline_sha256", "baseline_lines", "reviewed_lines", "symbols"},
    )
    reviewed: dict[str, dict[str, Any]] = {}
    for index, record in enumerate(reviewed_records):
        path = str(record["path"])
        digest = require_text(
            record["baseline_sha256"], f"reviewed_files[{index}].baseline_sha256"
        )
        require(
            SHA256.fullmatch(digest) is not None,
            f"reviewed file has invalid SHA-256: {path}",
        )
        lines = record["baseline_lines"]
        require(
            isinstance(lines, int) and not isinstance(lines, bool) and lines > 0,
            f"reviewed file must have a positive baseline line count: {path}",
        )
        require(
            record["reviewed_lines"] == f"1-{lines}",
            f"reviewed line interval is incomplete for {path}: "
            f"expected 1-{lines}, observed {record['reviewed_lines']!r}",
        )
        require_text_list(record["symbols"], f"reviewed_files[{index}].symbols")
        reviewed[path] = record

    implementation_records = path_records(
        plan["implementation_changes"],
        "implementation_changes",
        {"path", "baseline_state", "symbols", "action", "failure_semantics"},
    )
    implementation_paths: set[str] = set()
    for index, record in enumerate(implementation_records):
        path = str(record["path"])
        require(
            record["baseline_state"] in {"present", "absent"},
            f"implementation baseline_state is invalid: {path}",
        )
        require_text_list(record["symbols"], f"implementation_changes[{index}].symbols")
        require_text(record["action"], f"implementation_changes[{index}].action")
        require_text(
            record["failure_semantics"],
            f"implementation_changes[{index}].failure_semantics",
        )
        implementation_paths.add(path)

    require(
        isinstance(plan["evidence_changes"], list) and bool(plan["evidence_changes"]),
        "evidence_changes must be a nonempty list",
    )
    evidence_paths: set[str] = set()
    generated_evidence_paths: set[str] = set()
    for index, raw_record in enumerate(plan["evidence_changes"]):
        record = require_object(
            raw_record,
            f"evidence_changes[{index}]",
            {"path", "action"},
            {"generated", "generator"},
        )
        path = require_path(record["path"], f"evidence_changes[{index}].path")
        require_text(record["action"], f"evidence_changes[{index}].action")
        generated = record.get("generated", False)
        require(
            isinstance(generated, bool), f"evidence generated flag is invalid: {path}"
        )
        if generated:
            require_path(
                record.get("generator"), f"evidence_changes[{index}].generator"
            )
            generated_evidence_paths.add(path)
        else:
            require(
                "generator" not in record,
                f"non-generated evidence names a generator: {path}",
            )
        require(
            path not in evidence_paths,
            f"evidence_changes contains duplicate path: {path}",
        )
        evidence_paths.add(path)

    require(
        not (implementation_paths & planning_paths),
        "implementation and planning paths must be disjoint",
    )
    require(
        not (implementation_paths & evidence_paths),
        "implementation and evidence paths must be disjoint",
    )

    research = require_object(
        plan["research"], "research", {"decision", "reason", "sources"}
    )
    require(
        research["decision"] in {"required", "not_required"},
        "research.decision must be required or not_required",
    )
    require_text(research["reason"], "research.reason")
    require(isinstance(research["sources"], list), "research.sources must be a list")
    if research["decision"] == "required":
        require(bool(research["sources"]), "required research has no primary sources")
    source_urls: list[str] = []
    for index, raw_source in enumerate(research["sources"]):
        source = require_object(
            raw_source,
            f"research.sources[{index}]",
            {"title", "url", "retained_behavior", "rejected_assumption"},
        )
        require_text(source["title"], f"research.sources[{index}].title")
        url = require_text(source["url"], f"research.sources[{index}].url")
        require(url.startswith("https://"), f"research source is not HTTPS: {url}")
        source_urls.append(url)
        require_text(
            source["retained_behavior"],
            f"research.sources[{index}].retained_behavior",
        )
        require_text(
            source["rejected_assumption"],
            f"research.sources[{index}].rejected_assumption",
        )
    require(
        len(source_urls) == len(set(source_urls)),
        "research.sources contains duplicate URLs",
    )

    evidence = require_object(
        plan["trace_resource_debug"],
        "trace_resource_debug",
        {"runtime_trace", "resource_evidence", "debugging_evidence"},
    )
    for name in ("runtime_trace", "resource_evidence", "debugging_evidence"):
        decision = require_object(
            evidence[name], f"trace_resource_debug.{name}", {"decision", "reason"}
        )
        require(
            decision["decision"] in DECISIONS,
            f"trace_resource_debug.{name}.decision is invalid",
        )
        require_text(decision["reason"], f"trace_resource_debug.{name}.reason")

    return ValidatedPlan(
        document=plan,
        baseline_revision=revision,
        baseline_tree=tree,
        cargo_lock_sha256=cargo_lock_sha256,
        planning_paths=planning_paths,
        reviewed_files=reviewed,
        implementation_paths=frozenset(implementation_paths),
        evidence_paths=frozenset(evidence_paths),
        generated_evidence_paths=frozenset(generated_evidence_paths),
    )


def validate_planning_commit(
    repository: GitRepository, plan: ValidatedPlan, plan_bytes: bytes
) -> tuple[str, str]:
    head = repository.revision("HEAD^{commit}")
    resolved_baseline = repository.revision(f"{plan.baseline_revision}^{{commit}}")
    require(
        resolved_baseline == plan.baseline_revision,
        f"declared baseline does not resolve exactly: {plan.baseline_revision}",
    )
    require(
        repository.is_ancestor(plan.baseline_revision, head),
        f"declared baseline is not an ancestor of HEAD: {plan.baseline_revision}",
    )
    plan_commit = repository.planning_commit(plan.baseline_revision, head, plan_bytes)
    require(
        repository.is_ancestor(plan_commit, head),
        f"planning commit is not an ancestor of HEAD: {plan_commit}",
    )
    committed_plan = repository.blob(plan_commit, PLAN_PATH)
    require(
        plan_bytes == committed_plan,
        f"active change plan drifted after planning commit {plan_commit}: {PLAN_PATH}",
    )
    parents = repository.parents(plan_commit)
    require(
        len(parents) == 1,
        f"planning commit must have exactly one parent: {plan_commit}",
    )
    require(
        parents[0] == plan.baseline_revision,
        "planning commit parent does not equal declared baseline: "
        f"expected {plan.baseline_revision}, observed {parents[0]}",
    )
    resolved_tree = repository.revision(f"{plan.baseline_revision}^{{tree}}")
    require(
        resolved_tree == plan.baseline_tree,
        "declared baseline tree is stale: "
        f"expected {plan.baseline_tree}, observed {resolved_tree}",
    )
    planning_paths = repository.planning_commit_paths(parents[0], plan_commit)
    undeclared = sorted(planning_paths - plan.planning_paths)
    omitted = sorted(plan.planning_paths - planning_paths)
    require(
        not undeclared,
        f"planning commit changed undeclared paths: {undeclared}",
    )
    require(
        not omitted,
        f"planning commit did not contain declared planning paths: {omitted}",
    )
    return plan_commit, head


def validate_reviewed_files(repository: GitRepository, plan: ValidatedPlan) -> None:
    for path, record in plan.reviewed_files.items():
        data = repository.blob(plan.baseline_revision, path)
        try:
            data.decode("utf-8")
        except UnicodeDecodeError as error:
            raise PlanError(
                f"reviewed line evidence requires a UTF-8 baseline file: {path}"
            ) from error
        digest = hashlib.sha256(data).hexdigest()
        require(
            digest == record["baseline_sha256"],
            f"baseline SHA-256 is stale for {path}: expected "
            f"{record['baseline_sha256']}, observed {digest}",
        )
        line_count = len(data.splitlines())
        require(
            line_count == record["baseline_lines"],
            f"baseline line count is stale for {path}: expected "
            f"{record['baseline_lines']}, observed {line_count}",
        )
        require(
            record["reviewed_lines"] == f"1-{line_count}",
            f"reviewed lines do not cover the complete baseline file {path}: "
            f"expected 1-{line_count}, observed {record['reviewed_lines']!r}",
        )

    generated = plan.generated_evidence_paths
    required_reviews = {
        path
        for path in plan.planning_paths
        | plan.implementation_paths
        | plan.evidence_paths
        if repository.object_exists(plan.baseline_revision, path)
        and path not in generated
    }
    missing_reviews = sorted(required_reviews - set(plan.reviewed_files))
    require(
        not missing_reviews,
        f"existing planned paths lack complete baseline review: {missing_reviews}",
    )

    implementation_by_path = {
        str(record["path"]): record
        for record in plan.document["implementation_changes"]
    }
    for path, record in implementation_by_path.items():
        existed = repository.object_exists(plan.baseline_revision, path)
        expected = record["baseline_state"]
        require(
            existed == (expected == "present"),
            f"implementation baseline_state is stale for {path}: "
            f"declared {expected}, observed {'present' if existed else 'absent'}",
        )

    for record in plan.document["evidence_changes"]:
        if not record.get("generated", False):
            continue
        generator = str(record["generator"])
        require(
            repository.object_exists(plan.baseline_revision, generator),
            f"generated evidence uses a generator absent at baseline: {generator}",
        )
        require(
            generator in plan.reviewed_files,
            f"generated evidence generator lacks complete baseline review: {generator}",
        )
        commands = [
            shlex.split(command) for command in plan.document["acceptance_commands"]
        ]
        require(
            any(generator in command and "--check" in command for command in commands),
            f"generated evidence has no deterministic --check command: {generator}",
        )

    for path in plan.planning_paths:
        if repository.object_exists(plan.baseline_revision, path):
            continue
        require(
            path == PLAN_PATH,
            f"planning path was unexpectedly absent at baseline: {path}",
        )


def validate_changed_paths(
    repository: GitRepository,
    plan: ValidatedPlan,
    plan_commit: str,
    head: str,
) -> tuple[set[str], bool]:
    changed = repository.changed_paths(plan_commit, head)
    history = repository.committed_history_paths(plan_commit, head)
    require(
        PLAN_PATH not in changed | history,
        f"active change plan changed alongside implementation: {PLAN_PATH}",
    )
    expected = set(plan.implementation_paths | plan.evidence_paths)
    if not changed and head == plan_commit:
        return changed, True
    undeclared = sorted((changed | history) - expected)
    omitted = sorted(expected - changed)
    require(not undeclared, f"implementation contains undeclared paths: {undeclared}")
    require(not omitted, f"implementation omitted declared paths: {omitted}")

    if "Cargo.lock" not in expected:
        lock = repository.root / "Cargo.lock"
        require(lock.is_file(), "Cargo.lock is absent but was not declared for change")
        digest = hashlib.sha256(lock.read_bytes()).hexdigest()
        require(
            digest == plan.cargo_lock_sha256,
            "Cargo.lock changed without declaration: "
            f"expected {plan.cargo_lock_sha256}, observed {digest}",
        )
    return changed, False


def load_plan(path: Path) -> tuple[dict[str, Any], bytes]:
    require(path.is_file(), f"active change plan is absent: {path}")
    data = path.read_bytes()
    require(
        len(data) <= MAX_PLAN_BYTES,
        f"active change plan exceeds {MAX_PLAN_BYTES} bytes: {path}",
    )
    try:
        decoded = data.decode("utf-8")
    except UnicodeDecodeError as error:
        raise PlanError(f"active change plan is not UTF-8: {path}") from error
    try:
        document = json.loads(decoded)
    except json.JSONDecodeError as error:
        raise PlanError(
            f"active change plan is invalid JSON at line {error.lineno}: {error.msg}"
        ) from error
    require(isinstance(document, dict), "active change plan root must be an object")
    return document, data


def check(root: Path) -> tuple[ValidatedPlan, str, set[str], bool]:
    repository = GitRepository(root)
    document, plan_bytes = load_plan(repository.root / PLAN_PATH)
    plan = validate_schema(document)
    plan_commit, head = validate_planning_commit(repository, plan, plan_bytes)
    validate_reviewed_files(repository, plan)
    changed, planning_only = validate_changed_paths(repository, plan, plan_commit, head)
    return plan, plan_commit, changed, planning_only


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--root",
        type=Path,
        default=Path(__file__).resolve().parents[2],
        help="repository root; used by isolated policy tests",
    )
    arguments = parser.parse_args()
    plan, plan_commit, changed, planning_only = check(arguments.root)
    stage = "planning-only" if planning_only else "implementation"
    print(
        "change-plan: OK: "
        f"{plan.document['package_id']} at {plan_commit}, "
        f"{stage}, {len(changed)} post-plan paths"
    )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except PlanError as error:
        print(f"change-plan: {error}", file=sys.stderr)
        raise SystemExit(1) from error
