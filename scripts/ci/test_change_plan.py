#!/usr/bin/env python3
"""Mutation tests for the committed RRFlow change-plan policy."""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from typing import Any

CHECKER = Path(__file__).resolve().with_name("check_change_plan.py")
PLAN_PATH = Path("docs/roadmap/rrflow-1.0-active-change.json")


def run_git(root: Path, *arguments: str) -> str:
    result = subprocess.run(
        ["git", *arguments],
        cwd=root,
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout.strip()


def write(root: Path, relative: str | Path, contents: str) -> None:
    path = root / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(contents, encoding="utf-8")


def digest(contents: str) -> str:
    return hashlib.sha256(contents.encode("utf-8")).hexdigest()


class ChangePlanPolicyTests(unittest.TestCase):
    def create_repository(self) -> tuple[tempfile.TemporaryDirectory[str], Path]:
        temporary = tempfile.TemporaryDirectory(prefix="rrflow-change-plan-")
        root = Path(temporary.name)
        run_git(root, "init", "--quiet", "--initial-branch=main")
        run_git(root, "config", "user.name", "RRFlow Policy Test")
        run_git(root, "config", "user.email", "rrflow-policy@example.invalid")
        write(root, "Cargo.lock", "locked\n")
        write(root, "src/existing.py", "alpha\nbeta\n")
        write(root, "evidence.md", "baseline evidence\n")
        write(root, "scripts/generate.py", "generate\n")
        run_git(root, "add", ".")
        run_git(root, "commit", "--quiet", "-m", "baseline")
        return temporary, root

    def plan_document(self, root: Path) -> dict[str, Any]:
        revision = run_git(root, "rev-parse", "HEAD")
        tree = run_git(root, "rev-parse", "HEAD^{tree}")
        return {
            "schema_version": 1,
            "package_id": "test-change",
            "status": "active",
            "package_kind": "repository-engineering",
            "baseline": {
                "revision": revision,
                "tree": tree,
                "branch": "test",
                "development_remote": "development",
                "development_url": "https://example.invalid/rrflow-development.git",
                "worktree_state": "clean",
                "cargo_lock_sha256": digest("locked\n"),
            },
            "authority": {
                "alpha_prerequisite": "Test the plan boundary.",
                "roadmap_gates": ["J-01"],
                "poam_items": ["POAM-027"],
                "architecture_owner": "test-owner",
            },
            "current_behavior": "No plan is enforced.",
            "target_behavior": "The exact plan is enforced.",
            "unchanged_behavior": ["Runtime behavior is unchanged."],
            "planning_commit": {
                "must_precede_implementation": True,
                "must_have_exactly_one_parent": True,
                "allowed_paths": [PLAN_PATH.as_posix()],
            },
            "reviewed_files": [
                {
                    "path": "src/existing.py",
                    "baseline_sha256": digest("alpha\nbeta\n"),
                    "baseline_lines": 2,
                    "reviewed_lines": "1-2",
                    "symbols": ["existing"],
                },
                {
                    "path": "evidence.md",
                    "baseline_sha256": digest("baseline evidence\n"),
                    "baseline_lines": 1,
                    "reviewed_lines": "1-1",
                    "symbols": ["evidence"],
                },
                {
                    "path": "scripts/generate.py",
                    "baseline_sha256": digest("generate\n"),
                    "baseline_lines": 1,
                    "reviewed_lines": "1-1",
                    "symbols": ["generator"],
                },
            ],
            "implementation_changes": [
                {
                    "path": "src/existing.py",
                    "baseline_state": "present",
                    "symbols": ["existing"],
                    "action": "Change the existing function.",
                    "failure_semantics": "A wrong result fails the oracle.",
                },
                {
                    "path": "src/new.py",
                    "baseline_state": "absent",
                    "symbols": ["new"],
                    "action": "Create the planned implementation.",
                    "failure_semantics": "An invalid input is rejected.",
                },
            ],
            "evidence_changes": [
                {
                    "path": "evidence.md",
                    "action": "Record exact test results.",
                }
            ],
            "research": {
                "decision": "required",
                "reason": "The policy depends on Git behavior.",
                "sources": [
                    {
                        "title": "Git documentation",
                        "url": "https://git-scm.com/docs",
                        "retained_behavior": "Use committed ancestry.",
                        "rejected_assumption": "An uncommitted prompt is authority.",
                    }
                ],
            },
            "trace_resource_debug": {
                "runtime_trace": {
                    "decision": "not_applicable",
                    "reason": "No runtime change.",
                },
                "resource_evidence": {
                    "decision": "preserve",
                    "reason": "File reads are bounded by repository content.",
                },
                "debugging_evidence": {
                    "decision": "add",
                    "reason": "Denials have stable diagnostics.",
                },
            },
            "first_failure_oracle": ["Reject an undeclared path."],
            "edit_sequence": ["Implement, then verify."],
            "acceptance_commands": ["python3 scripts/ci/check_change_plan.py"],
            "stop_conditions": ["Stop on plan drift."],
            "roadmap_status_change": "none",
        }

    def commit_plan(
        self,
        root: Path,
        plan: dict[str, Any],
        *,
        mixed_implementation: bool = False,
    ) -> None:
        write(root, PLAN_PATH, json.dumps(plan, indent=2, sort_keys=True) + "\n")
        if mixed_implementation:
            write(root, "src/existing.py", "mixed with plan\n")
        run_git(root, "add", ".")
        run_git(root, "commit", "--quiet", "-m", "plan")

    def prepare_complete_change(
        self,
        root: Path,
        *,
        include_new: bool = True,
        commit_existing: bool = False,
    ) -> None:
        write(root, "src/existing.py", "implemented\n")
        if commit_existing:
            run_git(root, "add", "src/existing.py")
            run_git(root, "commit", "--quiet", "-m", "implement declared path")
        write(root, "evidence.md", "verified evidence\n")
        run_git(root, "add", "evidence.md")
        if include_new:
            write(root, "src/new.py", "new implementation\n")

    def check(self, root: Path) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, str(CHECKER), "--root", str(root)],
            cwd=root,
            check=False,
            capture_output=True,
            text=True,
        )

    def assert_rejected(
        self, result: subprocess.CompletedProcess[str], text: str
    ) -> None:
        self.assertNotEqual(result.returncode, 0, result.stdout)
        self.assertIn(text, result.stderr)

    def test_accepts_the_separate_planning_commit(self) -> None:
        temporary, root = self.create_repository()
        with temporary:
            self.commit_plan(root, self.plan_document(root))
            result = self.check(root)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("planning-only, 0 post-plan paths", result.stdout)

    def test_accepts_declared_committed_staged_unstaged_and_untracked_union(
        self,
    ) -> None:
        temporary, root = self.create_repository()
        with temporary:
            self.commit_plan(root, self.plan_document(root))
            self.prepare_complete_change(root, commit_existing=True)
            result = self.check(root)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("implementation, 3 post-plan paths", result.stdout)

    def test_accepts_a_generated_evidence_record(self) -> None:
        temporary, root = self.create_repository()
        with temporary:
            plan = self.plan_document(root)
            plan["evidence_changes"][0].update(
                {"generated": True, "generator": "scripts/generate.py"}
            )
            plan["acceptance_commands"].append("python3 scripts/generate.py --check")
            self.commit_plan(root, plan)
            self.prepare_complete_change(root)
            result = self.check(root)
            self.assertEqual(result.returncode, 0, result.stderr)

    def test_accepts_a_non_fast_forward_merge_of_the_planned_package(self) -> None:
        temporary, root = self.create_repository()
        with temporary:
            run_git(root, "branch", "feature")
            run_git(root, "checkout", "--quiet", "feature")
            self.commit_plan(root, self.plan_document(root))
            self.prepare_complete_change(root)
            run_git(root, "add", ".")
            run_git(root, "commit", "--quiet", "-m", "implementation")
            run_git(root, "checkout", "--quiet", "main")
            run_git(root, "merge", "--quiet", "--no-ff", "feature", "-m", "merge")
            result = self.check(root)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("implementation, 3 post-plan paths", result.stdout)

    def test_rejects_code_committed_with_the_plan(self) -> None:
        temporary, root = self.create_repository()
        with temporary:
            self.commit_plan(root, self.plan_document(root), mixed_implementation=True)
            result = self.check(root)
            self.assert_rejected(result, "planning commit changed undeclared paths")

    def test_rejects_an_undeclared_untracked_path(self) -> None:
        temporary, root = self.create_repository()
        with temporary:
            self.commit_plan(root, self.plan_document(root))
            self.prepare_complete_change(root)
            write(root, "src/surprise.py", "not planned\n")
            result = self.check(root)
            self.assert_rejected(result, "implementation contains undeclared paths")
            self.assertIn("src/surprise.py", result.stderr)

    def test_rejects_an_undeclared_staged_path(self) -> None:
        temporary, root = self.create_repository()
        with temporary:
            self.commit_plan(root, self.plan_document(root))
            self.prepare_complete_change(root)
            write(root, "scripts/generate.py", "staged surprise\n")
            run_git(root, "add", "scripts/generate.py")
            result = self.check(root)
            self.assert_rejected(result, "implementation contains undeclared paths")
            self.assertIn("scripts/generate.py", result.stderr)

    def test_rejects_an_undeclared_unstaged_path(self) -> None:
        temporary, root = self.create_repository()
        with temporary:
            self.commit_plan(root, self.plan_document(root))
            self.prepare_complete_change(root)
            write(root, "scripts/generate.py", "unstaged surprise\n")
            result = self.check(root)
            self.assert_rejected(result, "implementation contains undeclared paths")
            self.assertIn("scripts/generate.py", result.stderr)

    def test_rejects_an_undeclared_path_changed_and_reverted_in_history(self) -> None:
        temporary, root = self.create_repository()
        with temporary:
            self.commit_plan(root, self.plan_document(root))
            write(root, "src/surprise.py", "not planned\n")
            run_git(root, "add", "src/surprise.py")
            run_git(root, "commit", "--quiet", "-m", "unplanned")
            run_git(root, "rm", "--quiet", "src/surprise.py")
            run_git(root, "commit", "--quiet", "-m", "revert unplanned")
            self.prepare_complete_change(root)
            result = self.check(root)
            self.assert_rejected(result, "implementation contains undeclared paths")
            self.assertIn("src/surprise.py", result.stderr)

    def test_rejects_a_stale_review_digest(self) -> None:
        temporary, root = self.create_repository()
        with temporary:
            plan = self.plan_document(root)
            plan["reviewed_files"][0]["baseline_sha256"] = "0" * 64
            self.commit_plan(root, plan)
            self.prepare_complete_change(root)
            result = self.check(root)
            self.assert_rejected(
                result, "baseline SHA-256 is stale for src/existing.py"
            )

    def test_rejects_incomplete_line_coverage(self) -> None:
        temporary, root = self.create_repository()
        with temporary:
            plan = self.plan_document(root)
            plan["reviewed_files"][0]["reviewed_lines"] = "1-1"
            self.commit_plan(root, plan)
            self.prepare_complete_change(root)
            result = self.check(root)
            self.assert_rejected(result, "reviewed line interval is incomplete")

    def test_rejects_plan_drift_after_the_planning_commit(self) -> None:
        temporary, root = self.create_repository()
        with temporary:
            self.commit_plan(root, self.plan_document(root))
            self.prepare_complete_change(root)
            with (root / PLAN_PATH).open("a", encoding="utf-8") as target:
                target.write("\n")
            result = self.check(root)
            self.assert_rejected(
                result, "active change plan drifted after planning commit"
            )

    def test_rejects_missing_required_research(self) -> None:
        temporary, root = self.create_repository()
        with temporary:
            plan = self.plan_document(root)
            plan["research"]["sources"] = []
            self.commit_plan(root, plan)
            self.prepare_complete_change(root)
            result = self.check(root)
            self.assert_rejected(result, "required research has no primary sources")

    def test_rejects_missing_required_package_evidence(self) -> None:
        mutations = [
            ("first_failure_oracle", "first_failure_oracle must be a nonempty list"),
            ("acceptance_commands", "acceptance_commands must be a nonempty list"),
            ("stop_conditions", "stop_conditions must be a nonempty list"),
        ]
        for field, diagnostic in mutations:
            with self.subTest(field=field):
                temporary, root = self.create_repository()
                with temporary:
                    plan = self.plan_document(root)
                    plan[field] = []
                    self.commit_plan(root, plan)
                    self.prepare_complete_change(root)
                    result = self.check(root)
                    self.assert_rejected(result, diagnostic)

    def test_rejects_missing_observability_decision(self) -> None:
        temporary, root = self.create_repository()
        with temporary:
            plan = self.plan_document(root)
            plan["trace_resource_debug"]["runtime_trace"]["reason"] = ""
            self.commit_plan(root, plan)
            self.prepare_complete_change(root)
            result = self.check(root)
            self.assert_rejected(
                result, "trace_resource_debug.runtime_trace.reason must be nonempty"
            )

    def test_rejects_missing_failure_semantics(self) -> None:
        temporary, root = self.create_repository()
        with temporary:
            plan = self.plan_document(root)
            plan["implementation_changes"][0]["failure_semantics"] = ""
            self.commit_plan(root, plan)
            self.prepare_complete_change(root)
            result = self.check(root)
            self.assert_rejected(
                result, "implementation_changes[0].failure_semantics must be nonempty"
            )

    def test_rejects_an_unreviewed_generated_evidence_generator(self) -> None:
        temporary, root = self.create_repository()
        with temporary:
            plan = self.plan_document(root)
            plan["reviewed_files"] = [
                record
                for record in plan["reviewed_files"]
                if record["path"] != "scripts/generate.py"
            ]
            plan["evidence_changes"][0].update(
                {"generated": True, "generator": "scripts/generate.py"}
            )
            plan["acceptance_commands"].append("python3 scripts/generate.py --check")
            self.commit_plan(root, plan)
            self.prepare_complete_change(root)
            result = self.check(root)
            self.assert_rejected(
                result,
                "generated evidence generator lacks complete baseline review",
            )

    def test_rejects_generated_evidence_without_a_check_command(self) -> None:
        temporary, root = self.create_repository()
        with temporary:
            plan = self.plan_document(root)
            plan["evidence_changes"][0].update(
                {"generated": True, "generator": "scripts/generate.py"}
            )
            self.commit_plan(root, plan)
            self.prepare_complete_change(root)
            result = self.check(root)
            self.assert_rejected(
                result,
                "generated evidence has no deterministic --check command",
            )

    def test_rejects_an_undeclared_cargo_lock_change(self) -> None:
        temporary, root = self.create_repository()
        with temporary:
            self.commit_plan(root, self.plan_document(root))
            self.prepare_complete_change(root)
            write(root, "Cargo.lock", "changed lock\n")
            result = self.check(root)
            self.assert_rejected(result, "implementation contains undeclared paths")
            self.assertIn("Cargo.lock", result.stderr)

    def test_rejects_an_omitted_declared_path(self) -> None:
        temporary, root = self.create_repository()
        with temporary:
            self.commit_plan(root, self.plan_document(root))
            self.prepare_complete_change(root, include_new=False)
            result = self.check(root)
            self.assert_rejected(result, "implementation omitted declared paths")
            self.assertIn("src/new.py", result.stderr)


if __name__ == "__main__":
    unittest.main(verbosity=2)
