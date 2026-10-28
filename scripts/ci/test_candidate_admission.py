#!/usr/bin/env python3
"""Regressions for the candidate scheduling boundary, using the actual caller."""

import ast
import re
import unittest
from types import SimpleNamespace

from check_workflow import CALLER, PolicyError, verify_candidate_admission


class CandidateAdmissionTests(unittest.TestCase):
    def setUp(self):
        self.caller = CALLER.read_text(encoding="utf-8")

    def test_checked_in_caller_is_admitted(self):
        verify_candidate_admission(self.caller)

    def test_each_required_pr_transition_is_retained(self):
        for activity in ["opened", "synchronize", "reopened", "ready_for_review", "converted_to_draft"]:
            with self.subTest(activity=activity):
                changed = self.caller.replace(activity, "edited", 1)
                with self.assertRaisesRegex(PolicyError, "transitions"):
                    verify_candidate_admission(changed)

    def test_merge_group_and_manual_dispatch_cannot_be_omitted(self):
        for trigger in ["merge_group", "workflow_dispatch"]:
            with self.subTest(trigger=trigger):
                with self.assertRaisesRegex(PolicyError, "merge/manual"):
                    verify_candidate_admission(self.caller.replace(f"  {trigger}:\n", ""))

    def test_draft_guard_cannot_be_removed_or_reversed(self):
        condition = next(line for line in self.caller.splitlines(True) if line.startswith("    if:"))
        for replacement in ["", "    if: true\n", condition.replace("== false", "== true")]:
            with self.subTest(replacement=replacement):
                with self.assertRaisesRegex(PolicyError, "draft PRs"):
                    verify_candidate_admission(self.caller.replace(condition, replacement))

    def test_non_pr_candidates_cannot_be_skipped(self):
        with self.assertRaisesRegex(PolicyError, "merge-group and manual"):
            verify_candidate_admission(self.caller.replace("github.event_name != 'pull_request' || ", ""))

    def test_path_filters_cannot_skip_candidate_evidence(self):
        with self.assertRaisesRegex(PolicyError, "filters"):
            verify_candidate_admission(self.caller.replace("  merge_group:", "    paths: ['crates/**']\n  merge_group:"))

    def test_concurrency_cannot_cancel_another_pr_or_keep_stale_work(self):
        for old, new in [
            ("cancel-in-progress: true", "cancel-in-progress: false"),
            ("github.event.pull_request.number || github.ref", "github.workflow"),
        ]:
            with self.subTest(old=old):
                with self.assertRaisesRegex(PolicyError, "same PR or ref"):
                    verify_candidate_admission(self.caller.replace(old, new))

    def test_draft_events_have_a_distinct_concurrency_lane(self):
        expected = (
            "  group: ci-${{ github.workflow }}-"
            "${{ github.event.pull_request.number || github.ref }}-"
            "${{ github.event_name == 'pull_request' && github.event.pull_request.draft && 'draft' || 'candidate' }}"
        )
        self.assertIn(expected, self.caller)

    def test_concurrency_groups_for_pr_and_non_pr_events(self):
        # Evaluate the checked-in expressions for the exercised boolean/string
        # inputs. This model is not a live GitHub event-delivery test.
        group_line = next(line for line in self.caller.splitlines() if line.startswith("  group: "))
        template = group_line.removeprefix("  group: ")

        def group(event, number, ref, draft):
            github = SimpleNamespace(
                workflow="ci",
                event_name=event,
                ref=ref,
                event=SimpleNamespace(pull_request=SimpleNamespace(number=number, draft=draft)),
            )

            def evaluate(match):
                expression = match.group(1).replace("&&", " and ").replace("||", " or ")
                def interpret(node):
                    if isinstance(node, ast.Expression):
                        return interpret(node.body)
                    if isinstance(node, ast.Constant):
                        return node.value
                    if isinstance(node, ast.Name) and node.id == "github":
                        return github
                    if isinstance(node, ast.Attribute):
                        self.assertIn(node.attr, {"workflow", "event_name", "ref", "event", "pull_request", "number", "draft"})
                        return getattr(interpret(node.value), node.attr)
                    if isinstance(node, ast.Compare) and len(node.ops) == 1 and isinstance(node.ops[0], ast.Eq):
                        return interpret(node.left) == interpret(node.comparators[0])
                    if isinstance(node, ast.BoolOp):
                        value = interpret(node.values[0])
                        for child in node.values[1:]:
                            if isinstance(node.op, ast.And) and not value:
                                return value
                            if isinstance(node.op, ast.Or) and value:
                                return value
                            value = interpret(child)
                        return value
                    self.fail(f"unsupported concurrency expression node: {type(node).__name__}")

                return str(interpret(ast.parse(expression, mode="eval")))

            return re.sub(r"\$\{\{\s*(.*?)\s*\}\}", evaluate, template)

        draft = group("pull_request", 154, "refs/pull/154/merge", True)
        ready = group("pull_request", 154, "refs/pull/154/merge", False)
        self.assertEqual(draft, "ci-ci-154-draft")
        self.assertEqual(ready, "ci-ci-154-candidate")
        self.assertNotEqual(draft, ready)
        # Both opened/synchronize and converted_to_draft with draft=true use
        # the draft lane; a new ready event supersedes the old ready lane.
        self.assertEqual(group("pull_request", 154, "refs/pull/154/merge", True), draft)
        self.assertEqual(group("pull_request", 154, "refs/pull/154/merge", False), ready)
        self.assertNotEqual(group("pull_request", 155, "refs/pull/155/merge", False), ready)
        self.assertNotEqual(group("merge_group", None, "refs/heads/queue-a", False), ready)
        self.assertNotEqual(
            group("workflow_dispatch", None, "refs/heads/branch-a", False),
            group("workflow_dispatch", None, "refs/heads/branch-b", False),
        )

    def test_draft_lane_cannot_be_removed_or_reversed(self):
        lane = "github.event_name == 'pull_request' && github.event.pull_request.draft && 'draft' || 'candidate'"
        for replacement in [
            "'candidate'",
            "github.event_name == 'pull_request' && github.event.pull_request.draft && 'candidate' || 'draft'",
            "github.event_name == 'pull_request' && github.event.pull_request.draft && 'draft' || 'draft'",
        ]:
            with self.subTest(replacement=replacement):
                with self.assertRaisesRegex(PolicyError, "isolate draft and ready"):
                    verify_candidate_admission(self.caller.replace(lane, replacement))

    def test_another_caller_cannot_bypass_admission(self):
        with self.assertRaisesRegex(PolicyError, "one pipeline"):
            verify_candidate_admission(self.caller + "\n  bypass:\n    uses: ./.github/workflows/ci-reusable.yml\n")


if __name__ == "__main__":
    unittest.main()
