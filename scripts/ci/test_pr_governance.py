#!/usr/bin/env python3
"""Regressions for the pull-request work-in-progress limits."""

import unittest

from check_pr_governance import (
    GOVERNANCE_SINCE,
    INTEGRATION_LABEL,
    MAX_AGE_DAYS,
    MAX_LINES,
    MAX_OPEN,
    MAX_PER_AUTHOR,
    Facts,
    advisories,
    evaluate,
    generated_line_count,
)


def facts(**overrides) -> Facts:
    base = dict(number=10, author="agent", base_ref="main", labels=[], changed_files=3, changed_lines=120,
                merge_base_age_days=0.5, created_at="2026-11-11T00:00:00Z",
                open_pull_requests=[{"number": 10, "author": "agent", "created_at": "2026-11-11T00:00:00Z"}])
    base.update(overrides)
    return Facts(**base)


def opened(count: int, author: str = "other") -> list[dict]:
    return [{"number": 10, "author": "agent", "created_at": "2026-11-11T00:00:00Z"}] + [
        {"number": 100 + i, "author": author, "created_at": "2026-11-11T01:00:00Z"} for i in range(count)
    ]


class GovernanceTests(unittest.TestCase):
    def test_a_small_fresh_single_pull_request_is_admitted(self):
        self.assertEqual(evaluate(facts()), [])

    def test_an_empty_diff_is_refused(self):
        self.assertTrue(any("changes no files" in v for v in evaluate(facts(changed_files=0, changed_lines=0))))

    def test_a_stacked_pull_request_is_refused(self):
        self.assertTrue(any("no stacks" in v for v in evaluate(facts(base_ref="agent/other"))))

    def test_size_is_advisory_and_never_blocks_a_complete_package(self):
        big = facts(changed_lines=MAX_LINES + 1)
        self.assertEqual(evaluate(big), [])
        self.assertTrue(any("hand-written changed lines" in a for a in advisories(big)))
        self.assertEqual(advisories(facts(changed_lines=MAX_LINES)), [])

    def test_generated_lines_do_not_count_as_hand_written(self):
        numstat = "\n".join([
            "120\t30\tsdks/python/src/rrd_client/generated/endpoints.py",
            "10\t2\teval/schemas/storage-manifest-v1.json",
            "5\t5\tCargo.lock",
            "40\t1\tcrates/authority/rrd-engine/src/engine/context.rs",
            "-\t-\tdocs/evidence/image.png",
        ])
        self.assertEqual(generated_line_count(numstat), 172)
        mostly_generated = facts(changed_lines=MAX_LINES + 100, generated_lines=200)
        self.assertEqual(advisories(mostly_generated), [])

    def test_open_pull_request_cap_counts_every_open_pull_request(self):
        at_limit = facts(open_pull_requests=opened(MAX_OPEN - 1))
        self.assertEqual(evaluate(at_limit), [])
        over = facts(open_pull_requests=opened(MAX_OPEN))
        self.assertTrue(any(f"the limit is {MAX_OPEN}" in v for v in evaluate(over)))
        self.assertEqual(evaluate(facts(open_pull_requests=opened(MAX_OPEN), labels=[INTEGRATION_LABEL])), [])

    def test_per_author_cap(self):
        mine = facts(open_pull_requests=opened(MAX_PER_AUTHOR, author="agent"))
        self.assertTrue(any(f"the limit is {MAX_PER_AUTHOR}" in v for v in evaluate(mine)))
        theirs = facts(open_pull_requests=opened(MAX_PER_AUTHOR, author="someone-else"))
        self.assertFalse(any("agent would have" in v for v in evaluate(theirs)))

    def test_a_stale_branch_is_refused_even_when_small(self):
        stale = facts(merge_base_age_days=MAX_AGE_DAYS + 0.1)
        self.assertTrue(any("Merge main into the branch first" in v for v in evaluate(stale)))
        self.assertEqual(evaluate(facts(merge_base_age_days=MAX_AGE_DAYS)), [])

    def test_the_integration_label_does_not_excuse_emptiness_stacking_or_staleness(self):
        violations = evaluate(facts(changed_files=0, changed_lines=0, base_ref="agent/x",
                                    merge_base_age_days=30, labels=[INTEGRATION_LABEL]))
        self.assertEqual(len(violations), 3)

    def test_the_backlog_opened_before_the_limits_is_not_counted_and_not_capped(self):
        backlog = [{"number": 200 + i, "author": "agent", "created_at": "2026-11-03T00:00:00Z"} for i in range(20)]
        new = facts(open_pull_requests=opened(0) + backlog)
        self.assertEqual(evaluate(new), [], "old open pull requests do not block a new small one")
        old = facts(created_at="2026-11-03T00:00:00Z", changed_lines=40_000, open_pull_requests=opened(0) + backlog)
        self.assertEqual(evaluate(old), [], "a backlog pull request keeps its size while it is absorbed")
        stale_old = facts(created_at="2026-11-03T00:00:00Z", merge_base_age_days=40, changed_files=0, changed_lines=0)
        self.assertEqual(len(evaluate(stale_old)), 2, "but it must still be non-empty and merged with main")
        self.assertLess("2026-11-03T00:00:00Z", GOVERNANCE_SINCE)

    def test_every_violation_is_reported_at_once(self):
        violations = evaluate(facts(changed_files=0, changed_lines=MAX_LINES + 50, base_ref="dev",
                                    merge_base_age_days=10, open_pull_requests=opened(MAX_OPEN, author="agent")))
        self.assertEqual(len(violations), 5, "size is an advisory, so it is not among the violations")


class LiveFactsCases(unittest.TestCase):
    def test_facts_read_the_live_pull_request_not_the_frozen_event(self):
        import tempfile
        from pathlib import Path
        from unittest.mock import patch
        import check_pr_governance as governance

        event = {"pull_request": {"number": 7, "labels": [], "user": {"login": "stale"}}}
        live = {"number": 7, "user": {"login": "owner"}, "base": {"ref": "main", "sha": "b"},
                "head": {"sha": "h"}, "labels": [{"name": governance.INTEGRATION_LABEL}],
                "changed_files": 3, "additions": 10, "deletions": 2, "created_at": "2026-11-10T00:00:00Z"}
        calls = []

        def fake_json(url, token):
            calls.append(url)
            return live if url.endswith("/pulls/7") else []
        numstat = "8\t0\tsdks/go/generated/endpoints.go\n"
        with patch.object(governance, "github_json", side_effect=fake_json), \
                patch.object(governance, "merge_base_age_days", return_value=0.5), \
                patch.object(governance, "diff_numstat", return_value=numstat):
            facts = governance.facts_from_github(event, "rrflow/rrflow-development", None, Path(tempfile.gettempdir()))
        self.assertEqual(facts.author, "owner")
        self.assertEqual(facts.labels, [governance.INTEGRATION_LABEL])
        self.assertEqual(facts.changed_lines, 12)
        self.assertEqual(facts.generated_lines, 8)
        self.assertTrue(any(url.endswith("/pulls/7") for url in calls))


if __name__ == "__main__":
    unittest.main()
